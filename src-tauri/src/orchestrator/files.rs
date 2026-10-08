//! Attached files during a run: what the model is told about them up front, the fragments that
//! match the task, pictures for models that see them, and reading a place the model asks for.

use crate::attachments::model::{AttachmentKind, Locator};
use crate::attachments::retrieve::{self, FragmentHit};
use crate::attachments::store::AttachmentStore;
use crate::attachments::vision::{self, VisionReport};
use crate::providers::llm::capabilities::VisionPlan;
use crate::providers::llm::{ImageInput, LlmManager};

/// A file as the model refers to it: `F1`, `F2`, … in attach order.
#[derive(Debug, Clone, PartialEq)]
pub struct FileRef {
    pub label: String,
    pub id: String,
    pub file_name: String,
    pub kind: AttachmentKind,
}

/// Text read from a file for the model, with the place to cite.
#[derive(Debug, Clone, PartialEq)]
pub struct FileRead {
    /// "report.pdf, page 3".
    pub label: String,
    /// `attachment://…` of the place read.
    pub url: String,
    pub text: String,
}

pub struct RunFiles {
    store: AttachmentStore,
    pub files: Vec<FileRef>,
    pub plan: VisionPlan,
}

/// Characters of overview per file in the first message.
const OVERVIEW_CHARS: usize = 900;
/// Most pictures put in one message for a model that sees images.
pub const MAX_INLINE_IMAGES: usize = 8;

impl RunFiles {
    /// Like [`Self::load`], working out the vision plan only when the run has files.
    pub async fn load_with<F, Fut>(store: AttachmentStore, run_id: &str, plan: F) -> Result<Option<Self>, String>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = VisionPlan>,
    {
        let Some(mut files) = Self::load(store, run_id, VisionPlan { main_sees: false, detected: None, reader: None }).await? else {
            return Ok(None);
        };
        files.plan = plan().await;
        Ok(Some(files))
    }

    /// The files of a run (all conversation turns so far), or None when it has none.
    pub async fn load(store: AttachmentStore, run_id: &str, plan: VisionPlan) -> Result<Option<Self>, String> {
        let mut files: Vec<FileRef> = Vec::new();
        for (_, info) in store.for_run(run_id).await.map_err(|e| e.to_string())? {
            if files.iter().any(|f| f.id == info.id) {
                continue;
            }
            files.push(FileRef { label: format!("F{}", files.len() + 1), id: info.id, file_name: info.file_name, kind: info.kind });
        }
        Ok((!files.is_empty()).then_some(Self { store, files, plan }))
    }

    pub fn ids(&self) -> Vec<String> {
        self.files.iter().map(|f| f.id.clone()).collect()
    }

    /// Reads scanned pages and describes pictures as the vision plan allows.
    pub async fn prepare(&self, llm: &LlmManager, max_pages: u32) -> VisionReport {
        vision::prepare(&self.store, llm, &self.plan, &self.ids(), max_pages).await
    }

    /// The first message about the files: each file's overview with its cite address, then the
    /// fragments that best match `question`, within `budget` characters of fragment text.
    pub async fn context(&self, question: &str, budget: usize) -> String {
        let mut out = String::from(
            "The user attached these files. They are part of the request: use them, and cite them as Markdown links \
             to their attachment addresses with the place, e.g. [report.pdf, p. 3](attachment://ID?page=3).\n",
        );
        let outlines = self.outlines().await;
        for file in &self.files {
            let outline = outlines.iter().find(|(id, _)| *id == file.id).map(|(_, o)| o.as_str()).unwrap_or("");
            let outline: String = outline.chars().take(OVERVIEW_CHARS).collect();
            out.push_str(&format!("\n[{}] {} — attachment://{}\n{}\n", file.label, file.file_name, file.id, outline));
        }
        let hits = retrieve::search(self.store.pool(), &self.ids(), question, 12).await.unwrap_or_default();
        let mut used = 0;
        let mut shown = Vec::new();
        for hit in hits {
            let size = hit.text.chars().count();
            if used + size > budget && !shown.is_empty() {
                break;
            }
            used += size;
            shown.push(hit);
        }
        if !shown.is_empty() {
            out.push_str("\nFragments that match the request:\n");
            for hit in &shown {
                out.push_str(&self.fragment_block(hit));
            }
        }
        out
    }

    fn fragment_block(&self, hit: &FragmentHit) -> String {
        let label = self.files.iter().find(|f| f.id == hit.attachment_id).map(|f| f.label.as_str()).unwrap_or("");
        format!("\n--- [{label}] {} ({})\n{}\n", hit.locator.label(&hit.file_name), hit.url, hit.text)
    }

    async fn outlines(&self) -> Vec<(String, String)> {
        let mut out = Vec::new();
        for file in &self.files {
            let outline: String = sqlx::query_scalar("SELECT outline FROM attachments WHERE id = ?")
                .bind(&file.id)
                .fetch_optional(self.store.pool())
                .await
                .ok()
                .flatten()
                .unwrap_or_default();
            out.push((file.id.clone(), outline));
        }
        out
    }

    /// Pictures to show a model that sees images; none for a blind model (it gets descriptions).
    pub async fn images(&self) -> Vec<ImageInput> {
        if !self.plan.main_sees {
            return Vec::new();
        }
        let mut out = Vec::new();
        for file in self.files.iter().filter(|f| f.kind == AttachmentKind::Image).take(MAX_INLINE_IMAGES) {
            if let Ok(Some((media_type, bytes))) = self.store.model_image(&file.id).await {
                out.push(ImageInput::from_bytes(media_type, &bytes));
            }
        }
        out
    }

    /// A file by its label (`F2`), name, id or `attachment://` address.
    pub fn find(&self, reference: &str) -> Option<&FileRef> {
        let reference = reference.trim();
        let id = Locator::from_url(reference).map(|(id, _)| id);
        self.files.iter().find(|f| {
            f.label.eq_ignore_ascii_case(reference)
                || f.file_name.eq_ignore_ascii_case(reference)
                || f.id == reference
                || id.as_deref() == Some(f.id.as_str())
        })
    }

    /// Reads a place in a file (`place`), the fragments matching `query`, or its beginning,
    /// up to `limit` characters.
    pub async fn read(&self, reference: &str, place: Locator, query: Option<&str>, limit: usize) -> Result<FileRead, String> {
        let file = self.find(reference).ok_or_else(|| {
            let known: Vec<String> = self.files.iter().map(|f| format!("{} ({})", f.label, f.file_name)).collect();
            format!("No attached file \"{reference}\". Files: {}.", known.join(", "))
        })?;
        // An address given as the file reference carries the place.
        let place = match Locator::from_url(reference.trim()) {
            Some((_, from_url)) if place == Locator::default() => from_url,
            _ => place,
        };
        let pool = self.store.pool();
        let (hits, label, url) = if place != Locator::default() {
            let hits = retrieve::at(pool, &file.id, &place, 4).await.map_err(|e| e.to_string())?;
            if hits.is_empty() {
                return Err(format!("{} has nothing at {}.", file.file_name, place.label(&file.file_name)));
            }
            (hits, place.label(&file.file_name), place.to_url(&file.id))
        } else if let Some(query) = query.filter(|q| !q.trim().is_empty()) {
            let hits = retrieve::search(pool, &[file.id.clone()], query, 4).await.map_err(|e| e.to_string())?;
            if hits.is_empty() {
                return Err(format!("Nothing in {} matches \"{query}\".", file.file_name));
            }
            let (label, url) = if hits.len() == 1 {
                (hits[0].locator.label(&file.file_name), hits[0].url.clone())
            } else {
                (format!("{}, matches for \"{}\"", file.file_name, query.trim()), format!("attachment://{}", file.id))
            };
            (hits, label, url)
        } else {
            let hits = retrieve::at(pool, &file.id, &Locator::default(), 3).await.map_err(|e| e.to_string())?;
            if hits.is_empty() {
                return Err(format!("{} has no text to read.", file.file_name));
            }
            (hits, file.file_name.clone(), format!("attachment://{}", file.id))
        };
        let mut text = String::new();
        for hit in &hits {
            if !text.is_empty() && text.chars().count() + hit.text.chars().count() > limit {
                break;
            }
            if hits.len() > 1 {
                text.push_str(&format!("[{} — {}]\n", hit.locator.label(&file.file_name), hit.url));
            }
            text.push_str(&hit.text);
            text.push_str("\n\n");
        }
        let text = crate::utils::text::truncate_chars(text.trim_end(), limit).to_string();
        Ok(FileRead { label, url, text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;
    use crate::storage::db::Database;

    async fn files(add: &[(&str, Vec<u8>)]) -> (RunFiles, tempfile::TempDir) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        Database::with_pool(pool.clone()).await.migrate().await.unwrap();
        sqlx::query("INSERT INTO runs (id, query, status, config, created_at, updated_at) VALUES ('r', 'q', 'running', '{}', 0, 0)")
            .execute(&pool)
            .await
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let store = AttachmentStore::new(pool.clone(), dir.path().to_path_buf());
        let mut ids = Vec::new();
        for (name, bytes) in add {
            ids.push(store.add_bytes(name, bytes.clone()).await.unwrap().id);
        }
        store.link_to_run("r", &ids, 0).await.unwrap();
        let plan = VisionPlan { main_sees: false, detected: None, reader: None };
        (RunFiles::load(store, "r", plan).await.unwrap().unwrap(), dir)
    }

    #[tokio::test]
    async fn files_are_labelled_and_found_by_label_name_or_address() {
        let (files, _dir) = files(&[("a.txt", b"alpha".to_vec()), ("b.txt", b"beta".to_vec())]).await;
        assert_eq!(files.files.iter().map(|f| f.label.as_str()).collect::<Vec<_>>(), vec!["F1", "F2"]);
        let b = files.files[1].clone();
        assert_eq!(files.find("f2"), Some(&b));
        assert_eq!(files.find("B.TXT"), Some(&b));
        assert_eq!(files.find(&format!("attachment://{}?page=1", b.id)), Some(&b));
        assert_eq!(files.find("c.txt"), None);
    }

    #[tokio::test]
    async fn reading_a_page_rows_a_query_or_the_beginning() {
        let mut rows: Vec<Vec<String>> = vec![vec!["City".into(), "People".into()]];
        rows.extend((1..=150).map(|i| vec![format!("Town number {i} with a long name"), format!("{i}000")]));
        let (files, _dir) = files(&[
            ("report.pdf", test_files::pdf(&["Intro to the report", "Results show growth"])),
            ("cities.xlsx", test_files::xlsx(&[("Data", rows)])),
        ])
        .await;
        let page = files.read("F1", Locator::page(2), None, 4000).await.unwrap();
        assert_eq!(page.label, "report.pdf, page 2");
        assert!(page.text.contains("Results show growth"));
        assert_eq!(page.url, format!("attachment://{}?page=2", files.files[0].id));

        let from_url = files.read(&page.url, Locator::default(), None, 4000).await.unwrap();
        assert_eq!(from_url.label, "report.pdf, page 2");

        let found = files.read("cities.xlsx", Locator::default(), Some("Town number 120"), 4000).await.unwrap();
        assert!(found.text.contains("Row 121: Town number 120"), "{}", found.text);

        let start = files.read("F1", Locator::default(), None, 4000).await.unwrap();
        assert!(start.text.contains("Intro to the report"));

        assert!(files.read("F1", Locator::page(9), None, 4000).await.unwrap_err().contains("nothing at report.pdf, page 9"));
        assert!(files.read("F9", Locator::default(), None, 4000).await.unwrap_err().contains("F1 (report.pdf)"));
    }

    #[tokio::test]
    async fn the_first_message_lists_files_and_matching_fragments() {
        let (files, _dir) = files(&[("report.pdf", test_files::pdf(&["Weather was calm", "Revenue grew to 12 million"]))]).await;
        let context = files.context("What was the revenue?", 5000).await;
        assert!(context.contains("[F1] report.pdf — attachment://"));
        assert!(context.contains("Pages: 2"));
        assert!(context.contains("report.pdf, page 2"));
        assert!(context.contains("Revenue grew"));
        assert!(!context.contains("--- [F1] report.pdf, page 1"), "only matching fragments: {context}");
    }
}
