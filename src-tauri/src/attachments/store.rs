//! Attached files on disk and in the database.
//!
//! A file is copied into `<data>/attachments/<sha[0..2]>/<sha>.<ext>`, so the same content is kept
//! once however often it is attached. Its fragments go to `attachment_chunks` (with a full-text
//! index). Draft attachments belong to no run yet; they are removed when no longer used.

use std::path::{Path, PathBuf};

use base64::Engine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

use super::model::{AttachmentInfo, AttachmentKind, AttachmentStatus, ParsedFile};
use super::parse::{self, ParseError};

/// Why a file could not be attached; `code` is translated by the interface.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AttachError {
    pub file_name: String,
    pub code: String,
    pub message: String,
}

impl AttachError {
    fn new(file_name: &str, code: &str, message: impl Into<String>) -> Self {
        Self { file_name: file_name.to_string(), code: code.to_string(), message: message.into() }
    }
}

impl std::fmt::Display for AttachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file_name, self.message)
    }
}

pub struct AttachmentStore {
    pool: SqlitePool,
    dir: PathBuf,
    max_bytes: u64,
}

pub const DEFAULT_MAX_MB: u64 = 50;

const COLUMNS: &str = "id, file_name, kind, mime, size, status, page_count, sheet_count, char_count, scanned_pages, stored_name, created_at";

impl AttachmentStore {
    pub fn new(pool: SqlitePool, dir: PathBuf) -> Self {
        Self { pool, dir, max_bytes: DEFAULT_MAX_MB * 1024 * 1024 }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn with_max_mb(mut self, max_mb: u64) -> Self {
        self.max_bytes = max_mb.max(1) * 1024 * 1024;
        self
    }

    pub fn default_dir() -> PathBuf {
        crate::storage::db::Database::data_dir().join("attachments")
    }

    fn file_path(&self, stored_name: &str) -> PathBuf {
        self.dir.join(&stored_name[..2.min(stored_name.len())]).join(stored_name)
    }

    /// Attaches a file chosen or dropped by the user.
    pub async fn add_path(&self, path: &Path) -> Result<AttachmentInfo, AttachError> {
        let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "file".into());
        let size = tokio::fs::metadata(path)
            .await
            .map_err(|e| AttachError::new(&file_name, "unreadable", e.to_string()))?
            .len();
        if size > self.max_bytes {
            return Err(self.too_large(&file_name));
        }
        let bytes = tokio::fs::read(path).await.map_err(|e| AttachError::new(&file_name, "unreadable", e.to_string()))?;
        self.add_bytes(&file_name, bytes).await
    }

    fn too_large(&self, file_name: &str) -> AttachError {
        AttachError::new(file_name, "tooLarge", format!("larger than {} MB", self.max_bytes / 1024 / 1024))
    }

    /// Attaches file content (pasted or dropped from another app). The same content attached
    /// again returns the stored file.
    pub async fn add_bytes(&self, file_name: &str, bytes: Vec<u8>) -> Result<AttachmentInfo, AttachError> {
        let file_name = sanitize_name(file_name);
        if bytes.len() as u64 > self.max_bytes {
            return Err(self.too_large(&file_name));
        }
        let sha = format!("{:x}", Sha256::digest(&bytes));
        let db = |e: sqlx::Error| AttachError::new(&file_name, "storage", e.to_string());

        if let Some(existing) = self.by_sha(&sha).await.map_err(db)? {
            // A draft takes the newest name and counts as fresh again.
            if !self.is_used(&existing.id).await.map_err(db)? {
                sqlx::query("UPDATE attachments SET file_name = ?, created_at = ? WHERE id = ?")
                    .bind(&file_name)
                    .bind(now())
                    .bind(&existing.id)
                    .execute(&self.pool)
                    .await
                    .map_err(db)?;
            }
            return self.get_one(&existing.id).await.map_err(db)?.ok_or_else(|| AttachError::new(&file_name, "storage", "missing"));
        }

        let name_for_parse = file_name.clone();
        let parse_bytes = bytes.clone();
        let parsed = tokio::task::spawn_blocking(move || parse::parse(&name_for_parse, &parse_bytes))
            .await
            .map_err(|e| AttachError::new(&file_name, "damaged", e.to_string()))?
            .map_err(|e| parse_error(&file_name, e))?;

        let ext = file_name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
        let stored_name = if ext.is_empty() { sha.clone() } else { format!("{sha}.{ext}") };
        self.write_files(&stored_name, &bytes, &parsed).await.map_err(|e| AttachError::new(&file_name, "storage", e.to_string()))?;

        let id = crate::utils::id::new_id();
        let char_count: usize = parsed.fragments.iter().map(|f| f.text.chars().count()).sum();
        let status = if parsed.scanned_pages.is_empty() { AttachmentStatus::Ready } else { AttachmentStatus::NeedsVision };
        let mut tx = self.pool.begin().await.map_err(db)?;
        let inserted = sqlx::query(
            "INSERT INTO attachments (id, sha256, file_name, mime, kind, size, stored_name, status, page_count, sheet_count, char_count, scanned_pages, outline, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&sha)
        .bind(&file_name)
        .bind(parsed.mime)
        .bind(parsed.kind.as_str())
        .bind(bytes.len() as i64)
        .bind(&stored_name)
        .bind(status_str(status))
        .bind(parsed.page_count.map(i64::from))
        .bind((!parsed.sheets.is_empty()).then_some(parsed.sheets.len() as i64))
        .bind(char_count as i64)
        .bind(serde_json::to_string(&parsed.scanned_pages).unwrap_or_else(|_| "[]".into()))
        .bind(outline(&file_name, &parsed))
        .bind(now())
        .execute(&mut *tx)
        .await;
        if let Err(e) = inserted {
            // The same file attached twice at once: the other request stored it.
            drop(tx);
            if let Some(existing) = self.by_sha(&sha).await.map_err(db)? {
                return Ok(existing);
            }
            return Err(db(e));
        }
        for (ord, fragment) in parsed.fragments.iter().enumerate() {
            sqlx::query("INSERT INTO attachment_chunks (attachment_id, ord, locator, text) VALUES (?, ?, ?, ?)")
                .bind(&id)
                .bind(ord as i64)
                .bind(serde_json::to_string(&fragment.locator).unwrap_or_else(|_| "{}".into()))
                .bind(&fragment.text)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        self.get_one(&id).await.map_err(db)?.ok_or_else(|| AttachError::new(&file_name, "storage", "missing"))
    }

    async fn write_files(&self, stored_name: &str, bytes: &[u8], parsed: &ParsedFile) -> std::io::Result<()> {
        let path = self.file_path(stored_name);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, bytes).await?;
        if let Some(image) = &parsed.image {
            tokio::fs::write(self.model_image_path(stored_name, image.media_type), &image.bytes).await?;
            tokio::fs::write(self.thumbnail_path(stored_name), &image.thumbnail_jpeg).await?;
        }
        Ok(())
    }

    fn base_name(stored_name: &str) -> &str {
        stored_name.split('.').next().unwrap_or(stored_name)
    }

    fn thumbnail_path(&self, stored_name: &str) -> PathBuf {
        self.file_path(&format!("{}.thumb.jpg", Self::base_name(stored_name)))
    }

    fn model_image_path(&self, stored_name: &str, media_type: &str) -> PathBuf {
        let ext = if media_type == "image/png" { "png" } else { "jpg" };
        self.file_path(&format!("{}.model.{ext}", Self::base_name(stored_name)))
    }

    /// The image as prepared for a model (downscaled), with its media type.
    pub async fn model_image(&self, id: &str) -> Result<Option<(String, Vec<u8>)>, sqlx::Error> {
        let Some(stored) = self.stored_name(id).await? else { return Ok(None) };
        for media_type in ["image/jpeg", "image/png"] {
            if let Ok(bytes) = tokio::fs::read(self.model_image_path(&stored, media_type)).await {
                return Ok(Some((media_type.to_string(), bytes)));
            }
        }
        Ok(None)
    }

    async fn stored_name(&self, id: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT stored_name FROM attachments WHERE id = ?").bind(id).fetch_optional(&self.pool).await
    }

    /// Pages of a document that are scans not yet read.
    pub async fn unread_scans(&self, id: &str) -> Result<Vec<u32>, sqlx::Error> {
        let json: Option<String> = sqlx::query_scalar("SELECT scanned_pages FROM attachments WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(json.and_then(|j| serde_json::from_str(&j).ok()).unwrap_or_default())
    }

    /// The original bytes of a file.
    pub async fn original(&self, id: &str) -> Result<Option<Vec<u8>>, sqlx::Error> {
        let Some((_, path)) = self.stored_path(id).await? else { return Ok(None) };
        Ok(tokio::fs::read(path).await.ok())
    }

    /// Stores text read from a scanned page; the page no longer counts as unread and the file
    /// is ready once every scanned page has text.
    pub async fn add_page_text(&self, id: &str, page: u32, text: &str) -> Result<(), sqlx::Error> {
        let fragments = super::chunk::fragments(vec![(super::model::Locator::page(page), text.to_string())], super::chunk::FRAGMENT_CHARS);
        let remaining: Vec<u32> = self.unread_scans(id).await?.into_iter().filter(|p| *p != page).collect();
        self.add_fragments(id, &fragments).await?;
        sqlx::query("UPDATE attachments SET scanned_pages = ?, status = ? WHERE id = ?")
            .bind(serde_json::to_string(&remaining).unwrap_or_else(|_| "[]".into()))
            .bind(if remaining.is_empty() { "ready" } else { "needs_vision" })
            .bind(id)
            .execute(&self.pool)
            .await?;
        if remaining.is_empty() {
            // The outline no longer needs to warn about unread scans.
            let outline: String = sqlx::query_scalar("SELECT outline FROM attachments WHERE id = ?").bind(id).fetch_one(&self.pool).await?;
            let updated: Vec<&str> = outline.lines().collect();
            let updated: Vec<String> = updated
                .iter()
                .map(|line| match line.split_once(" (scanned, no text yet") {
                    Some((head, _)) if line.starts_with("Pages:") => format!("{head} (scanned pages read from images)"),
                    _ => line.to_string(),
                })
                .collect();
            sqlx::query("UPDATE attachments SET outline = ? WHERE id = ?").bind(updated.join("\n")).bind(id).execute(&self.pool).await?;
        }
        Ok(())
    }

    /// The stored description of an image, if it has been described.
    pub async fn image_description(&self, id: &str) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar("SELECT text FROM attachment_chunks WHERE attachment_id = ? ORDER BY ord LIMIT 1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    /// Keeps a text description of an image: used by models that cannot see it and by search.
    pub async fn set_image_description(&self, id: &str, description: &str) -> Result<(), sqlx::Error> {
        let text = format!("Image description: {description}");
        self.add_fragments(id, &[super::model::Fragment { locator: super::model::Locator::default(), text }]).await?;
        let outline: String = sqlx::query_scalar("SELECT outline FROM attachments WHERE id = ?").bind(id).fetch_one(&self.pool).await?;
        let summary: String = description.chars().take(600).collect();
        sqlx::query("UPDATE attachments SET outline = ? WHERE id = ?")
            .bind(format!("{outline}\nShows: {}", summary.replace('\n', " ")))
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn add_fragments(&self, id: &str, fragments: &[super::model::Fragment]) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let next: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(ord) + 1, 0) FROM attachment_chunks WHERE attachment_id = ?")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        let mut added = 0i64;
        for (i, fragment) in fragments.iter().enumerate() {
            sqlx::query("INSERT INTO attachment_chunks (attachment_id, ord, locator, text) VALUES (?, ?, ?, ?)")
                .bind(id)
                .bind(next + i as i64)
                .bind(serde_json::to_string(&fragment.locator).unwrap_or_else(|_| "{}".into()))
                .bind(&fragment.text)
                .execute(&mut *tx)
                .await?;
            added += fragment.text.chars().count() as i64;
        }
        sqlx::query("UPDATE attachments SET char_count = char_count + ? WHERE id = ?").bind(added).bind(id).execute(&mut *tx).await?;
        tx.commit().await
    }

    /// Path of the stored copy of a file.
    pub async fn stored_path(&self, id: &str) -> Result<Option<(String, PathBuf)>, sqlx::Error> {
        let row = sqlx::query("SELECT file_name, stored_name FROM attachments WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| (r.get::<String, _>("file_name"), self.file_path(&r.get::<String, _>("stored_name")))))
    }

    async fn by_sha(&self, sha: &str) -> Result<Option<AttachmentInfo>, sqlx::Error> {
        let row = sqlx::query(&format!("SELECT {COLUMNS} FROM attachments WHERE sha256 = ?"))
            .bind(sha)
            .fetch_optional(&self.pool)
            .await?;
        Ok(match row {
            Some(row) => Some(self.info(row).await),
            None => None,
        })
    }

    async fn get_one(&self, id: &str) -> Result<Option<AttachmentInfo>, sqlx::Error> {
        Ok(self.get(&[id.to_string()]).await?.into_iter().next())
    }

    /// Attachments by id, in the given order; unknown ids are skipped.
    pub async fn get(&self, ids: &[String]) -> Result<Vec<AttachmentInfo>, sqlx::Error> {
        let mut out = Vec::new();
        for id in ids {
            let row = sqlx::query(&format!("SELECT {COLUMNS} FROM attachments WHERE id = ?"))
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
            if let Some(row) = row {
                out.push(self.info(row).await);
            }
        }
        Ok(out)
    }

    async fn info(&self, row: sqlx::sqlite::SqliteRow) -> AttachmentInfo {
        let kind = AttachmentKind::from_str(&row.get::<String, _>("kind"));
        let stored_name: String = row.get("stored_name");
        let thumbnail = if kind == AttachmentKind::Image {
            tokio::fs::read(self.thumbnail_path(&stored_name))
                .await
                .ok()
                .map(|bytes| format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
        } else {
            None
        };
        let scanned: Vec<u32> = serde_json::from_str(&row.get::<String, _>("scanned_pages")).unwrap_or_default();
        AttachmentInfo {
            id: row.get("id"),
            file_name: row.get("file_name"),
            kind,
            mime: row.get("mime"),
            size: row.get("size"),
            status: if row.get::<String, _>("status") == "needs_vision" { AttachmentStatus::NeedsVision } else { AttachmentStatus::Ready },
            page_count: row.get("page_count"),
            sheet_count: row.get("sheet_count"),
            char_count: row.get("char_count"),
            scanned_pages: scanned.len() as i64,
            thumbnail,
            created_at: row.get("created_at"),
        }
    }

    async fn is_used(&self, id: &str) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM run_attachments WHERE attachment_id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(count > 0)
    }

    /// Records which files a run (or one turn of a conversation) was started with.
    pub async fn link_to_run(&self, run_id: &str, ids: &[String], turn_index: i64) -> Result<(), sqlx::Error> {
        link_to_run(&self.pool, run_id, ids, turn_index).await
    }

    /// Files of a run with the conversation turn each was added in.
    pub async fn for_run(&self, run_id: &str) -> Result<Vec<(i64, AttachmentInfo)>, sqlx::Error> {
        let rows = sqlx::query("SELECT attachment_id, turn_index FROM run_attachments WHERE run_id = ? ORDER BY turn_index, ord")
            .bind(run_id)
            .fetch_all(&self.pool)
            .await?;
        let mut out = Vec::new();
        for row in rows {
            if let Some(info) = self.get_one(&row.get::<String, _>("attachment_id")).await? {
                out.push((row.get("turn_index"), info));
            }
        }
        Ok(out)
    }

    /// Marks drafts as just used, so a draft kept in the query form is not cleaned up.
    pub async fn touch_drafts(&self, ids: &[String]) -> Result<(), sqlx::Error> {
        for id in ids {
            sqlx::query("UPDATE attachments SET created_at = ? WHERE id = ? AND NOT EXISTS (SELECT 1 FROM run_attachments r WHERE r.attachment_id = attachments.id)")
                .bind(now())
                .bind(id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// Removes a draft attachment; files used by a run stay.
    pub async fn remove_draft(&self, id: &str) -> Result<(), sqlx::Error> {
        if self.is_used(id).await? {
            return Ok(());
        }
        self.delete(id).await
    }

    async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        let Some(stored) = self.stored_name(id).await? else { return Ok(()) };
        sqlx::query("DELETE FROM attachments WHERE id = ?").bind(id).execute(&self.pool).await?;
        for path in [
            self.file_path(&stored),
            self.thumbnail_path(&stored),
            self.model_image_path(&stored, "image/jpeg"),
            self.model_image_path(&stored, "image/png"),
        ] {
            let _ = tokio::fs::remove_file(path).await;
        }
        Ok(())
    }

    /// Deletes files no run uses that were attached more than `min_age_secs` ago (drafts that
    /// were never sent, and files of deleted runs). Returns how many were removed.
    pub async fn cleanup(&self, min_age_secs: i64) -> Result<usize, sqlx::Error> {
        let ids: Vec<String> = sqlx::query_scalar(
            "SELECT id FROM attachments a WHERE created_at <= ? AND NOT EXISTS (SELECT 1 FROM run_attachments r WHERE r.attachment_id = a.id)",
        )
        .bind(now() - min_age_secs)
        .fetch_all(&self.pool)
        .await?;
        for id in &ids {
            self.delete(id).await?;
        }
        Ok(ids.len())
    }
}

/// Records which files a run (or one turn of a conversation) was started with; unknown ids are skipped.
pub async fn link_to_run(pool: &SqlitePool, run_id: &str, ids: &[String], turn_index: i64) -> Result<(), sqlx::Error> {
    for (ord, id) in ids.iter().enumerate() {
        sqlx::query("INSERT OR IGNORE INTO run_attachments (run_id, attachment_id, turn_index, ord) SELECT ?, id, ?, ? FROM attachments WHERE id = ?")
            .bind(run_id)
            .bind(turn_index)
            .bind(ord as i64)
            .bind(id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

fn status_str(status: AttachmentStatus) -> &'static str {
    match status {
        AttachmentStatus::Ready => "ready",
        AttachmentStatus::NeedsVision => "needs_vision",
    }
}

fn parse_error(file_name: &str, error: ParseError) -> AttachError {
    AttachError::new(file_name, error.code(), error.to_string())
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Keeps the visible name, without folders or control characters.
fn sanitize_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let clean: String = base.chars().filter(|c| !c.is_control()).collect::<String>().trim().chars().take(200).collect();
    if clean.is_empty() { "file".into() } else { clean }
}

/// What a model is told about a file before reading any of it.
fn outline(file_name: &str, parsed: &ParsedFile) -> String {
    let mut lines = vec![format!("File: {file_name}")];
    if let Some(pages) = parsed.page_count {
        let mut line = format!("Pages: {pages}");
        if !parsed.scanned_pages.is_empty() {
            line.push_str(&format!(" (scanned, no text yet: {})", page_list(&parsed.scanned_pages)));
        }
        lines.push(line);
    }
    for sheet in &parsed.sheets {
        let name = if sheet.name.is_empty() { "Table".to_string() } else { format!("Sheet \"{}\"", sheet.name) };
        lines.push(format!("{name}: {} rows; columns: {}", sheet.rows, sheet.columns.join(" | ")));
    }
    if !parsed.headings.is_empty() {
        let headings: Vec<&str> = parsed.headings.iter().take(30).map(String::as_str).collect();
        lines.push(format!("Headings: {}", headings.join(" · ")));
    }
    if let Some(image) = &parsed.image {
        lines.push(format!("Image: {}×{} px", image.width, image.height));
    }
    if parsed.sheets.is_empty() {
        if let Some(first) = parsed.fragments.first() {
            let start: String = first.text.chars().take(600).collect();
            lines.push(format!("Begins: {}", start.replace('\n', " ")));
        }
    }
    lines.join("\n")
}

fn page_list(pages: &[u32]) -> String {
    let shown: Vec<String> = pages.iter().take(20).map(u32::to_string).collect();
    if pages.len() > 20 { format!("{}…", shown.join(", ")) } else { shown.join(", ") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;
    use crate::storage::db::Database;

    async fn store() -> (AttachmentStore, tempfile::TempDir) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        let db = Database::with_pool(pool.clone()).await;
        db.migrate().await.unwrap();
        let dir = tempfile::tempdir().unwrap();
        (AttachmentStore::new(pool, dir.path().to_path_buf()), dir)
    }

    async fn run(store: &AttachmentStore, id: &str) {
        sqlx::query("INSERT INTO runs (id, query, status, config, created_at, updated_at) VALUES (?, 'q', 'completed', '{}', 0, 0)")
            .bind(id)
            .execute(&store.pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn files_are_parsed_stored_once_and_described() {
        let (store, dir) = store().await;
        let pdf = test_files::pdf(&["Annual report of Acme Corporation", "", "Revenue grew to 12 million"]);
        let info = store.add_bytes("report.pdf", pdf.clone()).await.unwrap();
        assert_eq!((info.kind, info.status, info.page_count, info.scanned_pages), (AttachmentKind::Document, AttachmentStatus::NeedsVision, Some(3), 1));
        assert!(info.char_count > 20);

        // The same content again is the same attachment, now under the newer name.
        let again = store.add_bytes("copy of report.pdf", pdf).await.unwrap();
        assert_eq!(again.id, info.id);
        assert_eq!(again.file_name, "copy of report.pdf");
        let files: Vec<_> = walkdir(dir.path());
        assert_eq!(files.len(), 1, "{files:?}");

        let outline: String = sqlx::query_scalar("SELECT outline FROM attachments WHERE id = ?").bind(&info.id).fetch_one(&store.pool).await.unwrap();
        assert!(outline.contains("Pages: 3 (scanned, no text yet: 2)"), "{outline}");
        assert!(outline.contains("Begins: Annual report"), "{outline}");
    }

    #[tokio::test]
    async fn images_keep_a_model_copy_and_a_thumbnail() {
        let (store, _dir) = store().await;
        let info = store.add_bytes("../../photo.png", test_files::png(3000, 2000, false)).await.unwrap();
        assert_eq!(info.file_name, "photo.png");
        assert_eq!(info.kind, AttachmentKind::Image);
        assert!(info.thumbnail.as_deref().unwrap().starts_with("data:image/jpeg;base64,"));
        let (media_type, bytes) = store.model_image(&info.id).await.unwrap().unwrap();
        assert_eq!(media_type, "image/jpeg");
        assert!(bytes.len() > 100);
    }

    #[tokio::test]
    async fn bad_and_oversized_files_are_refused_with_a_code() {
        let (store, _dir) = store().await;
        let error = store.add_bytes("virus.exe", b"MZ".to_vec()).await.unwrap_err();
        assert_eq!((error.file_name.as_str(), error.code.as_str()), ("virus.exe", "unsupported"));
        let small = AttachmentStore { max_bytes: 10, ..store };
        assert_eq!(small.add_bytes("notes.txt", vec![b'a'; 11]).await.unwrap_err().code, "tooLarge");
    }

    #[tokio::test]
    async fn drafts_are_removed_but_files_of_runs_stay_until_the_run_is_gone() {
        let (store, dir) = store().await;
        let draft = store.add_bytes("draft.txt", b"just a draft".to_vec()).await.unwrap();
        let used = store.add_bytes("used.txt", b"used by a run".to_vec()).await.unwrap();
        run(&store, "r1").await;
        store.link_to_run("r1", &[used.id.clone(), "unknown".into()], 0).await.unwrap();
        assert_eq!(store.for_run("r1").await.unwrap().iter().map(|(_, a)| a.id.clone()).collect::<Vec<_>>(), vec![used.id.clone()]);

        store.remove_draft(&used.id).await.unwrap();
        assert_eq!(store.get(&[used.id.clone()]).await.unwrap().len(), 1);
        store.remove_draft(&draft.id).await.unwrap();
        assert!(store.get(&[draft.id.clone()]).await.unwrap().is_empty());
        let chunks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM attachment_chunks WHERE attachment_id = ?").bind(&draft.id).fetch_one(&store.pool).await.unwrap();
        assert_eq!(chunks, 0);

        // Deleting the run frees its file for cleanup.
        assert_eq!(store.cleanup(0).await.unwrap(), 0);
        sqlx::query("DELETE FROM runs WHERE id = 'r1'").execute(&store.pool).await.unwrap();
        assert_eq!(store.cleanup(3600).await.unwrap(), 0, "recent files wait");
        sqlx::query("UPDATE attachments SET created_at = 0").execute(&store.pool).await.unwrap();
        store.touch_drafts(&[used.id.clone()]).await.unwrap();
        assert_eq!(store.cleanup(3600).await.unwrap(), 0, "a draft still in the form is kept");
        assert_eq!(store.cleanup(-10).await.unwrap(), 1);
        assert!(walkdir(dir.path()).is_empty());
    }

    fn walkdir(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            if entry.path().is_dir() {
                out.extend(walkdir(&entry.path()));
            } else {
                out.push(entry.path());
            }
        }
        out
    }
}
