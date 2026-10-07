//! Which models can see images, and who reads images in a run.

use std::collections::HashMap;
use std::sync::Mutex;

use super::manager::{LlmBackend, LlmConfig, LlmManager};
use super::ollama::OllamaProvider;
use super::openai_compatible::OpenAiCompatibleProvider;
use super::openrouter::OpenRouterProvider;

/// Answers already learned from catalogs, by provider address and model.
static KNOWN: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);

fn cache_key(config: &LlmConfig, model: &str) -> String {
    let address = match config.backend {
        LlmBackend::OpenRouter => "openrouter",
        LlmBackend::Ollama => config.ollama_url.as_str(),
        LlmBackend::OllamaCloud => config.ollama_cloud_url.as_str(),
        LlmBackend::OpenAiCompatible => config.openai_base_url.as_str(),
    };
    format!("{:?}|{address}|{model}", config.backend)
}

/// Whether `model` of the configured provider sees images, from the provider's catalog.
/// None when the catalog does not say (many local OpenAI-compatible servers).
pub async fn vision_support(config: &LlmConfig, model: &str) -> Option<bool> {
    let model = model.trim();
    if model.is_empty() {
        return None;
    }
    let key = cache_key(config, model);
    if let Some(known) = KNOWN.lock().unwrap().as_ref().and_then(|m| m.get(&key).copied()) {
        return Some(known);
    }
    let answer = match config.backend {
        LlmBackend::OpenRouter => OpenRouterProvider::sees_images(config.openrouter_api_key.clone(), model).await,
        LlmBackend::Ollama => OllamaProvider::new(config.ollama_url.clone()).ok()?.sees_images(model).await,
        LlmBackend::OllamaCloud => {
            OllamaProvider::cloud(config.ollama_cloud_url.clone(), config.ollama_cloud_api_key.clone()).ok()?.sees_images(model).await
        }
        LlmBackend::OpenAiCompatible => {
            OpenAiCompatibleProvider::new(config.openai_base_url.clone(), config.openai_api_key.clone(), true).ok()?.sees_images(model).await
        }
    };
    if let Some(answer) = answer {
        KNOWN.lock().unwrap().get_or_insert_with(HashMap::new).insert(key, answer);
    }
    answer
}

/// The model of the configured provider used for the main work.
pub fn main_model(config: &LlmConfig) -> &str {
    match config.backend {
        LlmBackend::OpenRouter => &config.openrouter_model,
        LlmBackend::Ollama => &config.ollama_model,
        LlmBackend::OllamaCloud => &config.ollama_cloud_model,
        LlmBackend::OpenAiCompatible => &config.openai_model,
    }
}

/// Who handles images in a run.
#[derive(Debug, Clone, PartialEq)]
pub struct VisionPlan {
    /// The main model sees images: they go to it directly.
    pub main_sees: bool,
    /// What the catalog says about the main model (None: unknown).
    pub detected: Option<bool>,
    /// The model that reads images and scanned pages (the main model, or the model for
    /// images when the main one is blind); None when no model can.
    pub reader: Option<String>,
}

impl VisionPlan {
    /// From the settings `llm_vision` (`auto` | `on` | `off`) and `vision_model`.
    pub async fn resolve(settings: &HashMap<String, String>) -> Self {
        let config = LlmManager::config_from_settings(settings);
        let main = main_model(&config).trim().to_string();
        let detected = vision_support(&config, &main).await;
        let main_sees = match settings.get("llm_vision").map(|s| s.as_str()) {
            Some("on") => true,
            Some("off") => false,
            _ => detected.unwrap_or(false),
        };
        let separate = settings.get("vision_model").map(|m| m.trim().to_string()).filter(|m| !m.is_empty());
        let reader = if main_sees && !main.is_empty() { Some(main) } else { separate };
        Self { main_sees, detected, reader }
    }
}
