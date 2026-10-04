use async_trait::async_trait;

use super::openai_compatible::OpenAiCompatibleProvider;
use super::types::*;

const BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenRouter configuration of the shared OpenAI-compatible client.
pub struct OpenRouterProvider {
    inner: OpenAiCompatibleProvider,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Result<Self, LlmError> {
        if api_key.trim().is_empty() {
            return Err(LlmError::NotConfigured(
                "OpenRouter API key is required".into(),
            ));
        }
        Ok(Self {
            inner: OpenAiCompatibleProvider::new(BASE_URL.into(), api_key, true)?.for_openrouter(),
        })
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.inner = self.inner.with_base_url(url);
        self
    }

    /// Checks the API key without generating anything.
    pub async fn check_key(&self) -> Result<(), LlmError> {
        self.inner.get_authenticated("key").await.map(|_| ())
    }

    /// Model IDs from the catalog this provider talks to.
    pub async fn models(&self) -> Result<Vec<String>, LlmError> {
        self.inner.list_models().await
    }

    /// Load the public catalog, including before the user has entered an API key.
    pub async fn list_models(api_key: String) -> Result<Vec<String>, LlmError> {
        OpenAiCompatibleProvider::new(BASE_URL.into(), api_key, true)?
            .for_openrouter()
            .list_models()
            .await
    }
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn chat_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        self.inner.chat_completion(request).await
    }

    async fn pricing(
        &self,
        model: &str,
        usage: &LlmUsage,
        at: chrono::DateTime<chrono::Utc>,
    ) -> Option<crate::providers::accounting::PriceQuote> {
        self.inner.pricing(model, usage, at).await
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        self.inner.health_check().await
    }
}
