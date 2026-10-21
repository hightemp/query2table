//! The Statistics section: what the app spent by period, from the usage journal.

use tauri::State;

use crate::storage::repository::Repository;
use crate::storage::usage::{UsageReport, UsageRequest};
use crate::AppState;

/// The report of a period (also used by tests without the app).
pub async fn get_usage_report(repo: &Repository, request: &UsageRequest) -> Result<UsageReport, String> {
    repo.usage_report(request).await
}

#[tauri::command]
pub async fn get_usage(state: State<'_, AppState>, request: UsageRequest) -> Result<UsageReport, String> {
    let repo = Repository::new(state.db.pool().clone());
    get_usage_report(&repo, &request).await
}

/// Disk space the app's data takes now.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub struct StorageUsage {
    /// The database with its write-ahead log.
    pub database_bytes: u64,
    pub attachments_bytes: u64,
    /// Pictures kept with research answers (inside the database).
    pub answer_images_bytes: u64,
}

fn folder_size(path: &std::path::Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.metadata() {
            Ok(meta) if meta.is_dir() => folder_size(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        })
        .sum()
}

pub async fn storage_usage(repo: &Repository, database: &std::path::Path, attachments: &std::path::Path) -> Result<StorageUsage, String> {
    let file_size = |suffix: &str| {
        let mut name = database.as_os_str().to_owned();
        name.push(suffix);
        std::fs::metadata(std::path::PathBuf::from(name)).map(|m| m.len()).unwrap_or(0)
    };
    let images: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(length(data)), 0) FROM research_images")
        .fetch_one(repo.pool())
        .await
        .map_err(|e| e.to_string())?;
    let attachments = attachments.to_path_buf();
    let attachments_bytes = tokio::task::spawn_blocking(move || folder_size(&attachments)).await.unwrap_or(0);
    Ok(StorageUsage {
        database_bytes: file_size("") + file_size("-wal"),
        attachments_bytes,
        answer_images_bytes: images.max(0) as u64,
    })
}

#[tauri::command]
pub async fn get_storage_usage(state: State<'_, AppState>) -> Result<StorageUsage, String> {
    let repo = Repository::new(state.db.pool().clone());
    storage_usage(&repo, &crate::storage::db::Database::db_path(), &crate::attachments::store::AttachmentStore::default_dir()).await
}

/// Prices the calls recorded without a price with today's rates; returns how many got one.
#[tauri::command]
pub async fn reprice_usage(state: State<'_, AppState>) -> Result<i64, String> {
    let settings: std::collections::HashMap<String, String> = state.db.get_all_settings().await.map_err(|e| e.to_string())?.into_iter().collect();
    Repository::new(state.db.pool().clone()).reprice_usage(&settings).await
}

#[tauri::command]
pub async fn clear_usage(state: State<'_, AppState>) -> Result<(), String> {
    Repository::new(state.db.pool().clone()).clear_usage().await.map_err(|e| e.to_string())
}

/// This month's spending against the monthly budget (local calendar).
#[tauri::command]
pub async fn get_monthly_budget(state: State<'_, AppState>) -> Result<crate::storage::usage::MonthlyBudget, String> {
    Repository::new(state.db.pool().clone()).monthly_budget(&chrono::Local::now()).await
}

/// When the first recorded call happened (None before any).
#[tauri::command]
pub async fn get_usage_first_at(state: State<'_, AppState>) -> Result<Option<i64>, String> {
    Repository::new(state.db.pool().clone()).usage_first_at().await.map_err(|e| e.to_string())
}
