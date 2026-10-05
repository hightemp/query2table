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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    crate::providers::accounting::validate_pricing_setting(&key, &value)?;
    state
        .db
        .set_setting(&key, &value)
        .await
        .map_err(|e| e.to_string())?;

    if key == "ui_language" {
        crate::apply_tray_language(&app, &value);
    }
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

/// Saves the edited settings in one transaction, after checking every value.
#[tauri::command]
pub async fn update_settings(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    values: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    validate_settings(&values)?;
    state.db.set_settings(&values).await.map_err(|e| e.to_string())?;
    if let Some(language) = values.get("ui_language") {
        crate::apply_tray_language(&app, language);
    }
    if let Some(url) = values.get("active_proxy_url") {
        crate::providers::http::set_runtime_proxy((!url.trim().is_empty()).then(|| url.clone()));
    }
    Ok(())
}

fn validate_settings(values: &std::collections::HashMap<String, String>) -> Result<(), String> {
    for (key, value) in values {
        crate::providers::accounting::validate_pricing_setting(key, value)?;
    }
    Ok(())
}

/// Settings that are never written to or read from a settings file.
fn private_setting(key: &str) -> bool {
    key.ends_with("_api_key") || key == "proxy_list" || key == "active_proxy_url"
}

fn settings_export_json(settings: &std::collections::HashMap<String, String>) -> String {
    let shared: std::collections::BTreeMap<_, _> =
        settings.iter().filter(|(key, _)| !private_setting(key)).collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "app": "query2table",
        "version": 1,
        "settings": shared,
    }))
    .unwrap_or_default()
}

/// Settings from a file written by `export_settings`; API keys and proxies are ignored.
fn parse_settings_import(text: &str) -> Result<std::collections::HashMap<String, String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|_| "This is not a settings file exported by Query2Table.".to_string())?;
    let settings = value
        .get("settings")
        .and_then(|s| s.as_object())
        .ok_or_else(|| "This is not a settings file exported by Query2Table.".to_string())?;
    let mut imported = std::collections::HashMap::new();
    for (key, value) in settings {
        if private_setting(key) {
            continue;
        }
        let text = match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => continue,
        };
        imported.insert(key.clone(), text);
    }
    validate_settings(&imported)?;
    Ok(imported)
}

/// Writes the shareable settings (no API keys or proxies) to a JSON file.
#[tauri::command]
pub async fn export_settings(state: State<'_, AppState>, path: String) -> Result<(), String> {
    let settings: std::collections::HashMap<String, String> =
        state.db.get_all_settings().await.map_err(|e| e.to_string())?.into_iter().collect();
    std::fs::write(&path, settings_export_json(&settings)).map_err(|e| format!("Failed to write file: {e}"))
}

/// Reads a settings file; the values are applied as unsaved changes in the form.
#[tauri::command]
pub async fn read_settings_file(path: String) -> Result<std::collections::HashMap<String, String>, String> {
    let text = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file: {e}"))?;
    parse_settings_import(&text)
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

#[cfg(test)]
mod batch_tests {
    use super::*;
    use std::collections::HashMap;
    use crate::storage::db::Database;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn settings_are_saved_together_after_all_of_them_are_checked() {
        let pool = SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        let db = Database::with_pool(pool).await;
        db.migrate().await.unwrap();
        let values = HashMap::from([
            ("llm_temperature".to_string(), "0.3".to_string()),
            ("brave_price_per_1000".to_string(), "-1".to_string()),
        ]);
        assert!(validate_settings(&values).is_err());
        assert_eq!(db.get_setting("llm_temperature").await.unwrap().as_deref(), Some("0.7"));
        let values = HashMap::from([
            ("llm_temperature".to_string(), "0.3".to_string()),
            ("max_parallel_fetches".to_string(), "4".to_string()),
        ]);
        validate_settings(&values).unwrap();
        db.set_settings(&values).await.unwrap();
        assert_eq!(db.get_setting("llm_temperature").await.unwrap().as_deref(), Some("0.3"));
        assert_eq!(db.get_setting("max_parallel_fetches").await.unwrap().as_deref(), Some("4"));
    }

    #[test]
    fn exported_settings_leave_out_keys_and_proxies() {
        let settings = HashMap::from([
            ("llm_provider".to_string(), "ollama_cloud".to_string()),
            ("ollama_cloud_api_key".to_string(), "secret".to_string()),
            ("brave_api_key".to_string(), "secret".to_string()),
            ("proxy_list".to_string(), r#"[{"name":"p","url":"http://u:pw@h:1"}]"#.to_string()),
            ("active_proxy_url".to_string(), "http://u:pw@h:1".to_string()),
            ("llm_temperature".to_string(), "0.2".to_string()),
        ]);
        let json = settings_export_json(&settings);
        assert!(!json.contains("secret") && !json.contains("pw"), "{json}");
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value["app"], "query2table");
        assert_eq!(value["settings"]["llm_temperature"], "0.2");
    }

    #[test]
    fn imported_settings_are_checked_and_never_replace_keys() {
        let text = r#"{"app":"query2table","version":1,"settings":{"llm_temperature":0.4,"notifications_enabled":false,"brave_api_key":"x","proxy_list":"[]","max_parallel_fetches":"6"}}"#;
        let imported = parse_settings_import(text).unwrap();
        assert_eq!(imported.get("llm_temperature").map(String::as_str), Some("0.4"));
        assert_eq!(imported.get("notifications_enabled").map(String::as_str), Some("false"));
        assert_eq!(imported.get("max_parallel_fetches").map(String::as_str), Some("6"));
        assert!(!imported.contains_key("brave_api_key") && !imported.contains_key("proxy_list"));
        assert!(parse_settings_import("[1,2]").is_err());
        assert!(parse_settings_import(r#"{"settings":{"brave_price_per_1000":"-5"}}"#).is_err());
        assert!(parse_settings_import("not json").unwrap_err().contains("not a settings file"));
    }
}
