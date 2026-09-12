use crate::providers::llm::ollama::OllamaProvider;
use crate::providers::llm::openrouter::OpenRouterProvider;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

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
