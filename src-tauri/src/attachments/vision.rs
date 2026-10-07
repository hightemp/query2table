//! Turns what only a vision model can read into text: scanned PDF pages and pictures.
//! Results are stored with the file, so each page or picture is read once for all runs.

use super::model::AttachmentKind;
use super::render::render_pages;
use super::store::AttachmentStore;
use crate::providers::llm::ImageInput;
use crate::roles::vision_reader::VisionReader;

/// What preparing the files took, for the run's activity log.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct VisionReport {
    pub pages_read: u32,
    pub pages_failed: u32,
    /// Scanned pages left unread because the run's page limit was reached.
    pub pages_skipped: u32,
    pub images_described: u32,
    pub images_failed: u32,
}

/// How many pages are rendered and sent at once.
const PARALLEL_PAGES: usize = 3;

/// Reads unread scanned pages of the given files, at most `max_pages` in total.
pub async fn read_scans(store: &AttachmentStore, reader: &VisionReader<'_>, ids: &[String], max_pages: u32) -> VisionReport {
    let mut report = VisionReport::default();
    let mut budget = max_pages;
    for id in ids {
        let pages = store.unread_scans(id).await.unwrap_or_default();
        if pages.is_empty() {
            continue;
        }
        let take = (budget as usize).min(pages.len());
        report.pages_skipped += (pages.len() - take) as u32;
        budget -= take as u32;
        if take == 0 {
            continue;
        }
        let Ok(Some(bytes)) = store.original(id).await else {
            report.pages_failed += take as u32;
            continue;
        };
        for batch in pages[..take].chunks(PARALLEL_PAGES) {
            let (pdf, wanted) = (bytes.clone(), batch.to_vec());
            let rendered = match tokio::task::spawn_blocking(move || render_pages(&pdf, &wanted)).await {
                Ok(Ok(rendered)) => rendered,
                _ => {
                    report.pages_failed += batch.len() as u32;
                    continue;
                }
            };
            report.pages_failed += (batch.len() - rendered.len()) as u32;
            let reads = rendered.into_iter().map(|(page, image)| async move {
                (page, reader.transcribe_page(ImageInput::from_bytes(image.media_type, &image.bytes)).await)
            });
            for (page, result) in futures::future::join_all(reads).await {
                match result {
                    Ok(text) => {
                        // A blank page still counts as read.
                        let text = if text.is_empty() { "[No text on this page]".to_string() } else { text };
                        if store.add_page_text(id, page, &text).await.is_ok() {
                            report.pages_read += 1;
                        } else {
                            report.pages_failed += 1;
                        }
                    }
                    Err(_) => report.pages_failed += 1,
                }
            }
        }
    }
    report
}

/// Describes pictures that have no description yet.
pub async fn describe_images(store: &AttachmentStore, reader: &VisionReader<'_>, ids: &[String]) -> VisionReport {
    let mut report = VisionReport::default();
    for info in store.get(ids).await.unwrap_or_default() {
        if info.kind != AttachmentKind::Image || store.image_description(&info.id).await.ok().flatten().is_some() {
            continue;
        }
        let Ok(Some((media_type, bytes))) = store.model_image(&info.id).await else {
            report.images_failed += 1;
            continue;
        };
        match reader.describe_image(ImageInput::from_bytes(media_type, &bytes)).await {
            Ok(text) if !text.is_empty() && store.set_image_description(&info.id, &text).await.is_ok() => report.images_described += 1,
            _ => report.images_failed += 1,
        }
    }
    report
}

/// Prepares a run's files: reads scanned pages with the model for images, and describes
/// pictures when the main model cannot see them. Without any model for images, scans stay
/// unread and are counted as skipped.
pub async fn prepare(
    store: &AttachmentStore,
    manager: &crate::providers::llm::LlmManager,
    plan: &crate::providers::llm::capabilities::VisionPlan,
    ids: &[String],
    max_pages: u32,
) -> VisionReport {
    let Some(model) = plan.reader.clone() else {
        let mut report = VisionReport::default();
        for id in ids {
            report.pages_skipped += store.unread_scans(id).await.unwrap_or_default().len() as u32;
        }
        return report;
    };
    let reader = VisionReader { manager, model };
    let mut report = read_scans(store, &reader, ids, max_pages).await;
    if !plan.main_sees {
        let described = describe_images(store, &reader, ids).await;
        report.images_described = described.images_described;
        report.images_failed = described.images_failed;
    }
    report
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use super::*;
    use crate::attachments::model::AttachmentStatus;
    use crate::attachments::{retrieve, test_files, Locator};
    use crate::providers::llm::manager::{LlmConfig, LlmManager};
    use crate::providers::llm::{CompletionRequest, CompletionResponse, LlmError, LlmProvider};
    use crate::storage::db::Database;

    /// Answers like a vision model and records what it was sent.
    struct FakeVision {
        seen: Mutex<Vec<(String, usize, String)>>,
        fail_page_text: Option<&'static str>,
    }

    #[async_trait]
    impl LlmProvider for FakeVision {
        async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
            let user = request.messages.last().unwrap();
            let call = self.seen.lock().unwrap().len();
            self.seen.lock().unwrap().push((request.model.clone(), user.images.len(), user.content.clone()));
            if self.fail_page_text.is_some() && call == 1 {
                return Err(LlmError::ParseError("bad".into()));
            }
            let content = if user.content.contains("Transcribe") { format!("Scanned contract clause {call}") } else { "A red tractor in a field".into() };
            Ok(CompletionResponse { content, model: request.model, prompt_tokens: 1, completion_tokens: 1, total_tokens: 2, usage: Default::default() })
        }
        fn provider_name(&self) -> &str {
            "fake"
        }
        async fn health_check(&self) -> Result<(), LlmError> {
            Ok(())
        }
    }

    async fn setup(fail: Option<&'static str>) -> (AttachmentStore, Arc<FakeVision>, LlmManager, tempfile::TempDir) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        Database::with_pool(pool.clone()).await.migrate().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        let fake = Arc::new(FakeVision { seen: Mutex::new(Vec::new()), fail_page_text: fail });
        let manager = LlmManager::with_provider(fake.clone(), LlmConfig::default());
        (AttachmentStore::new(pool, dir.path().to_path_buf()), fake, manager, dir)
    }

    #[tokio::test]
    async fn scanned_pages_are_read_once_by_the_model_for_images() {
        let (store, fake, manager, _dir) = setup(None).await;
        let info = store.add_bytes("contract.pdf", test_files::pdf(&["", "Typed second page of the contract", ""])).await.unwrap();
        assert_eq!(info.status, AttachmentStatus::NeedsVision);
        let reader = VisionReader { manager: &manager, model: "llava".into() };

        let report = read_scans(&store, &reader, &[info.id.clone()], 50).await;
        assert_eq!(report, VisionReport { pages_read: 2, ..Default::default() });
        let seen = fake.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 2);
        assert!(seen.iter().all(|(model, images, _)| model == "llava" && *images == 1));

        let after = store.get(&[info.id.clone()]).await.unwrap().remove(0);
        assert_eq!((after.status, after.scanned_pages), (AttachmentStatus::Ready, 0));
        assert!(after.char_count > info.char_count);
        let page = retrieve::at(store_pool(&store), &info.id, &Locator::page(3), 3).await.unwrap();
        assert!(page[0].text.starts_with("Scanned contract clause"));
        let hits = retrieve::search(store_pool(&store), &[info.id.clone()], "clause", 5).await.unwrap();
        assert_eq!(hits.len(), 2);

        // Already read: a second run sends nothing.
        assert_eq!(read_scans(&store, &reader, &[info.id.clone()], 50).await, VisionReport::default());
        assert_eq!(fake.seen.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn the_page_limit_and_failures_are_reported() {
        let (store, _fake, manager, _dir) = setup(Some("second")).await;
        let info = store.add_bytes("scan.pdf", test_files::pdf(&["", "", "", ""])).await.unwrap();
        let reader = VisionReader { manager: &manager, model: "llava".into() };
        let report = read_scans(&store, &reader, &[info.id.clone()], 3).await;
        assert_eq!((report.pages_read, report.pages_failed, report.pages_skipped), (2, 1, 1));
        let after = store.get(&[info.id.clone()]).await.unwrap().remove(0);
        assert_eq!((after.status, after.scanned_pages), (AttachmentStatus::NeedsVision, 2));
    }

    #[tokio::test]
    async fn pictures_are_described_once() {
        let (store, fake, manager, _dir) = setup(None).await;
        let photo = store.add_bytes("photo.png", test_files::png(64, 48, false)).await.unwrap();
        let doc = store.add_bytes("notes.txt", b"plain notes".to_vec()).await.unwrap();
        let reader = VisionReader { manager: &manager, model: "llava".into() };
        let ids = [photo.id.clone(), doc.id.clone()];
        assert_eq!(describe_images(&store, &reader, &ids).await.images_described, 1);
        assert_eq!(describe_images(&store, &reader, &ids).await.images_described, 0);
        assert_eq!(fake.seen.lock().unwrap().len(), 1);
        assert_eq!(store.image_description(&photo.id).await.unwrap().as_deref(), Some("Image description: A red tractor in a field"));
        let hits = retrieve::search(store_pool(&store), &[photo.id.clone()], "tractor", 5).await.unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[tokio::test]
    async fn preparing_follows_the_vision_plan() {
        use crate::providers::llm::capabilities::VisionPlan;
        let (store, fake, manager, _dir) = setup(None).await;
        let scan = store.add_bytes("scan.pdf", test_files::pdf(&["", ""])).await.unwrap();
        let photo = store.add_bytes("photo.png", test_files::png(20, 20, false)).await.unwrap();
        let ids = [scan.id.clone(), photo.id.clone()];

        let nobody = VisionPlan { main_sees: false, detected: Some(false), reader: None };
        assert_eq!(prepare(&store, &manager, &nobody, &ids, 50).await, VisionReport { pages_skipped: 2, ..Default::default() });
        assert!(fake.seen.lock().unwrap().is_empty());

        // The main model sees images: it gets pictures directly, so only scans are read.
        let main = VisionPlan { main_sees: true, detected: Some(true), reader: Some("gpt-4o".into()) };
        let report = prepare(&store, &manager, &main, &ids, 50).await;
        assert_eq!((report.pages_read, report.images_described), (2, 0));

        let helper = VisionPlan { main_sees: false, detected: Some(false), reader: Some("llava".into()) };
        assert_eq!(prepare(&store, &manager, &helper, &ids, 50).await.images_described, 1);
        assert_eq!(fake.seen.lock().unwrap().last().unwrap().0, "llava");
    }

    fn store_pool(store: &AttachmentStore) -> &sqlx::SqlitePool {
        store.pool()
    }
}
