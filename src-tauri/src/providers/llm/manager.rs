use std::sync::Arc;
use tracing::info;

use super::types::*;
use super::openrouter::OpenRouterProvider;
use super::ollama::OllamaProvider;

/// Which LLM provider backend to use.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmBackend {
    OpenRouter,
    Ollama,
}

/// Configuration for the LLM manager.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub backend: LlmBackend,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend: LlmBackend::OpenRouter,
            openrouter_api_key: String::new(),
            openrouter_model: "openai/gpt-4.1-mini".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            temperature: 0.2,
            max_tokens: 4096,
        }
    }
}

/// Manages LLM providers and routes requests to the configured backend.
pub struct LlmManager {
    provider: Arc<dyn LlmProvider>,
    config: LlmConfig,
}

impl LlmManager {
    /// Create a new LLM manager from configuration.
    pub fn from_config(config: LlmConfig) -> Result<Self, LlmError> {
        let provider: Arc<dyn LlmProvider> = match config.backend {
            LlmBackend::OpenRouter => {
                if config.openrouter_api_key.is_empty() {
                    return Err(LlmError::NotConfigured(
                        "OpenRouter API key is required".to_string()
                    ));
                }
                Arc::new(OpenRouterProvider::new(config.openrouter_api_key.clone()))
            }
            LlmBackend::Ollama => {
                Arc::new(OllamaProvider::new(config.ollama_url.clone()))
            }
        };

        info!(backend = ?config.backend, "LLM manager initialized");

        Ok(Self { provider, config })
    }

    /// Send a chat completion using the configured model and settings.
    pub async fn complete(
        &self,
        messages: Vec<Message>,
        json_mode: bool,
    ) -> Result<CompletionResponse, LlmError> {
        let model = match self.config.backend {
            LlmBackend::OpenRouter => &self.config.openrouter_model,
            LlmBackend::Ollama => &self.config.ollama_model,
        };

        let request = CompletionRequest {
            messages,
            model: model.clone(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            json_mode,
        };

        self.provider.chat_completion(request).await
    }

    /// Send a chat completion with a specific model override.
    pub async fn complete_with_model(
        &self,
        messages: Vec<Message>,
        model: &str,
        json_mode: bool,
    ) -> Result<CompletionResponse, LlmError> {
        let request = CompletionRequest {
            messages,
            model: model.to_string(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            json_mode,
        };

        self.provider.chat_completion(request).await
    }

    /// Check if the configured provider is healthy.
    pub async fn health_check(&self) -> Result<(), LlmError> {
        self.provider.health_check().await
    }

    /// Get the currently active provider name.
    pub fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Build LlmConfig from settings stored in the database.
    pub fn config_from_settings(settings: &std::collections::HashMap<String, String>) -> LlmConfig {
        let backend = match settings.get("llm_provider").map(|s| s.as_str()) {
            Some("ollama") => LlmBackend::Ollama,
            _ => LlmBackend::OpenRouter,
        };

        LlmConfig {
            backend,
            openrouter_api_key: settings.get("openrouter_api_key").cloned().unwrap_or_default(),
            openrouter_model: settings.get("openrouter_model")
                .cloned()
                .unwrap_or_else(|| "openai/gpt-4.1-mini".to_string()),
            ollama_url: settings.get("ollama_url")
                .cloned()
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            ollama_model: settings.get("ollama_model")
                .cloned()
                .unwrap_or_else(|| "llama3".to_string()),
            temperature: settings.get("llm_temperature")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.2),
            max_tokens: settings.get("llm_max_tokens")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_config() {
        let config = LlmConfig::default();
        assert_eq!(config.backend, LlmBackend::OpenRouter);
        assert_eq!(config.temperature, 0.2);
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_config_from_settings_openrouter() {
        let mut settings = HashMap::new();
        settings.insert("llm_provider".to_string(), "openrouter".to_string());
        settings.insert("openrouter_api_key".to_string(), "sk-test".to_string());
        settings.insert("openrouter_model".to_string(), "anthropic/claude-3.5-sonnet".to_string());
        settings.insert("llm_temperature".to_string(), "0.5".to_string());

        let config = LlmManager::config_from_settings(&settings);
        assert_eq!(config.backend, LlmBackend::OpenRouter);
        assert_eq!(config.openrouter_api_key, "sk-test");
        assert_eq!(config.openrouter_model, "anthropic/claude-3.5-sonnet");
        assert_eq!(config.temperature, 0.5);
    }

    #[test]
    fn test_config_from_settings_ollama() {
        let mut settings = HashMap::new();
        settings.insert("llm_provider".to_string(), "ollama".to_string());
        settings.insert("ollama_url".to_string(), "http://gpu:11434".to_string());
        settings.insert("ollama_model".to_string(), "mistral".to_string());

        let config = LlmManager::config_from_settings(&settings);
        assert_eq!(config.backend, LlmBackend::Ollama);
        assert_eq!(config.ollama_url, "http://gpu:11434");
        assert_eq!(config.ollama_model, "mistral");
    }

    #[test]
    fn test_config_from_empty_settings() {
        let settings = HashMap::new();
        let config = LlmManager::config_from_settings(&settings);
        assert_eq!(config.backend, LlmBackend::OpenRouter);
        assert_eq!(config.openrouter_model, "openai/gpt-4.1-mini");
    }

    #[test]
    fn test_manager_requires_api_key_for_openrouter() {
        let config = LlmConfig {
            backend: LlmBackend::OpenRouter,
            openrouter_api_key: String::new(),
            ..Default::default()
        };
        let result = LlmManager::from_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn test_manager_ollama_no_key_required() {
        let config = LlmConfig {
            backend: LlmBackend::Ollama,
            ..Default::default()
        };
        let result = LlmManager::from_config(config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "ollama");
    }
}
