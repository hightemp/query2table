use crate::providers::llm::ollama::OllamaProvider;
use crate::providers::llm::openrouter::OpenRouterProvider;
use crate::storage::db::Database;
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Serialize)]
pub struct AppPaths {
    pub data_dir: String,
    pub database_file: String,
    pub log_dir: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppLocation {
    Data,
    Database,
    Logs,
}

impl AppLocation {
    fn path(self) -> Result<PathBuf, String> {
        let path = match self {
            Self::Data => Database::data_dir(),
            Self::Database => Database::db_path(),
            Self::Logs => crate::utils::logging::log_dir(),
        };
        if path.is_absolute() {
            Ok(path)
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .map_err(|e| e.to_string())
        }
    }

    fn folder(self) -> Result<PathBuf, String> {
        match self {
            Self::Database => Self::Data.path(),
            _ => self.path(),
        }
    }
}

#[tauri::command]
pub fn get_app_paths() -> Result<AppPaths, String> {
    Ok(AppPaths {
        data_dir: AppLocation::Data.path()?.to_string_lossy().into_owned(),
        database_file: AppLocation::Database.path()?.to_string_lossy().into_owned(),
        log_dir: AppLocation::Logs.path()?.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn copy_app_path(app: tauri::AppHandle, location: AppLocation) -> Result<(), String> {
    app.clipboard()
        .write_text(location.path()?.to_string_lossy().into_owned())
        .map_err(|e| format!("Could not copy the application path: {e}"))
}

/// Copy a user-selected result value without exposing clipboard read access.
#[tauri::command]
pub fn copy_text(app: tauri::AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|e| format!("Could not copy the selected text: {e}"))
}

#[tauri::command]
pub async fn open_app_folder(app: tauri::AppHandle, location: AppLocation) -> Result<(), String> {
    let folder = location.folder()?;
    tokio::fs::create_dir_all(&folder)
        .await
        .map_err(|e| format!("Could not access the application folder: {e}"))?;
    app.opener()
        .open_path(folder.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|e| format!("Could not open the application folder: {e}"))
}

#[tauri::command]
pub async fn list_openrouter_models(api_key: String) -> Result<Vec<String>, String> {
    OpenRouterProvider::list_models(api_key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_ollama_cloud_models(
    base_url: String,
    api_key: String,
) -> Result<Vec<String>, String> {
    // The public cloud catalog works without a key. Use current form credentials when provided.
    let provider = if api_key.trim().is_empty() {
        OllamaProvider::new(base_url)
    } else {
        OllamaProvider::cloud(base_url, api_key)
    }
    .map_err(|e| e.to_string())?;
    provider.list_models().await.map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Vec<Setting>, String> {
    state
        .db
        .get_all_settings()
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|(key, value)| Setting { key, value })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    state.db.get_setting(&key).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state
        .db
        .set_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    // Apply the selected proxy immediately so subsequent requests use it.
    if key == "active_proxy_url" {
        let url = if value.trim().is_empty() {
            None
        } else {
            Some(value)
        };
        crate::providers::http::set_runtime_proxy(url);
    }

    Ok(())
}

#[cfg(test)]
mod file_location_tests {
    use super::*;

    #[test]
    fn displayed_locations_match_storage_and_logging_paths() {
        let paths = get_app_paths().unwrap();
        assert_eq!(
            PathBuf::from(&paths.database_file),
            PathBuf::from(&paths.data_dir).join("data.db")
        );
        assert_eq!(
            PathBuf::from(paths.log_dir),
            AppLocation::Logs.folder().unwrap()
        );
        assert!(PathBuf::from(paths.data_dir).is_absolute());
    }

    #[test]
    fn database_open_action_targets_its_folder_and_rejects_arbitrary_paths() {
        assert_eq!(
            AppLocation::Database.folder().unwrap(),
            AppLocation::Data.path().unwrap()
        );
        assert!(serde_json::from_str::<AppLocation>("\"/tmp/arbitrary\"").is_err());
    }
}
