use crate::providers::accounting::{
    PriceQuote, RequestInfo, RequestKind, UsageGuard, UsageObserver,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{info, warn};

use super::ollama::OllamaProvider;
use super::openai_compatible::OpenAiCompatibleProvider;
use super::openrouter::OpenRouterProvider;
use super::types::*;
use crate::utils::retry::{retry_with_backoff, RetryAction, RetryConfig};

/// Which LLM provider backend to use.
#[derive(Debug, Clone, PartialEq)]
pub enum LlmBackend {
    OpenRouter,
    Ollama,
    OllamaCloud,
    OpenAiCompatible,
}

/// Configuration for the LLM manager.
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub backend: LlmBackend,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_cloud_url: String,
    pub ollama_cloud_api_key: String,
    pub ollama_cloud_model: String,
    pub openai_base_url: String,
    pub openai_api_key: String,
    pub openai_model: String,
    pub openai_json_mode: bool,
    pub temperature: f32,
    pub max_tokens: u32,
    pub reasoning_effort: ReasoningEffort,
    pub pricing_overrides: String,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            backend: LlmBackend::OpenRouter,
            openrouter_api_key: String::new(),
            openrouter_model: "openai/gpt-4.1-mini".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            ollama_cloud_url: "https://ollama.com".to_string(),
            ollama_cloud_api_key: String::new(),
            ollama_cloud_model: "gpt-oss:120b".to_string(),
            openai_base_url: "http://localhost:8080/v1".to_string(),
            openai_api_key: String::new(),
            openai_model: String::new(),
            openai_json_mode: true,
            temperature: 0.2,
            max_tokens: 4096,
            reasoning_effort: ReasoningEffort::Auto,
            pricing_overrides: "{}".into(),
        }
    }
}

/// Manages LLM providers and routes requests to the configured backend.
#[derive(Clone)]
pub struct LlmManager {
    provider: Arc<dyn LlmProvider>,
    config: LlmConfig,
    diagnostics: Option<UnboundedSender<LlmIssue>>,
    accounting: Option<Arc<dyn UsageObserver>>,
    /// Research conversation turn stamped on diagnostics.
    turn_index: Option<u32>,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

impl LlmManager {
    pub fn from_config_with_diagnostics(
        config: LlmConfig,
        sender: UnboundedSender<LlmIssue>,
    ) -> Result<Self, LlmError> {
        match Self::from_config(config.clone()) {
            Ok(manager) => Ok(manager.with_diagnostics(sender)),
            Err(error) => {
                let (provider, model) = match config.backend {
                    LlmBackend::OpenRouter => ("openrouter", &config.openrouter_model),
                    LlmBackend::Ollama => ("ollama", &config.ollama_model),
                    LlmBackend::OllamaCloud => ("ollama_cloud", &config.ollama_cloud_model),
                    LlmBackend::OpenAiCompatible => ("openai_compatible", &config.openai_model),
                };
                let _ = sender.send(LlmIssue {
                    code: error.code().into(),
                    provider: provider.into(),
                    model: safe_diagnostic_text(&config, model, 200),
                    stage: Some("setup".into()),
                    message: safe_diagnostic_text(&config, &error.to_string(), 1200),
                    max_tokens: config.max_tokens,
                    prompt_tokens: None,
                    completion_tokens: None,
                    reasoning_tokens: None,
                    retry_after_ms: None,
                    attempt: 1,
                    max_attempts: 1,
                    will_retry: false,
                    outcome: Some(IssueOutcome::Stopped.as_str().into()),
                    call_id: Some(crate::utils::id::new_id()),
                    turn_index: None,
                    at: Some(now_ms()),
                });
                Err(error)
            }
        }
    }

    /// Create a new LLM manager from configuration.
    pub fn from_config(config: LlmConfig) -> Result<Self, LlmError> {
        let provider: Arc<dyn LlmProvider> = match config.backend {
            LlmBackend::OpenRouter => {
                Arc::new(OpenRouterProvider::new(config.openrouter_api_key.clone())?)
            }
            LlmBackend::Ollama => Arc::new(OllamaProvider::new(config.ollama_url.clone())?),
            LlmBackend::OllamaCloud => Arc::new(OllamaProvider::cloud(
                config.ollama_cloud_url.clone(),
                config.ollama_cloud_api_key.clone(),
            )?),
            LlmBackend::OpenAiCompatible => Arc::new(OpenAiCompatibleProvider::new(
                config.openai_base_url.clone(),
                config.openai_api_key.clone(),
                config.openai_json_mode,
            )?),
        };

        let model = match config.backend {
            LlmBackend::OpenRouter => &config.openrouter_model,
            LlmBackend::Ollama => &config.ollama_model,
            LlmBackend::OllamaCloud => &config.ollama_cloud_model,
            LlmBackend::OpenAiCompatible => &config.openai_model,
        };
        if model.trim().is_empty() {
            return Err(LlmError::NotConfigured(format!(
                "{} model is required",
                provider.provider_name()
            )));
        }
        if matches!(config.backend, LlmBackend::Ollama | LlmBackend::OllamaCloud) {
            super::ollama::validate_reasoning(model, config.reasoning_effort)?;
        }
        info!(backend = ?config.backend, "LLM manager initialized");

        Ok(Self {
            provider,
            config,
            diagnostics: None,
            accounting: None,
            turn_index: None,
        })
    }

    /// Send a chat completion using the configured model and settings.
    pub async fn complete(
        &self,
        messages: Vec<Message>,
        json_mode: bool,
    ) -> Result<CompletionResponse, LlmError> {
        self.complete_with_stage(None, messages, json_mode).await
    }

    /// Attach a run-scoped diagnostic sink. The manager never owns run/UI state.
    pub fn with_diagnostics(mut self, sender: UnboundedSender<LlmIssue>) -> Self {
        self.diagnostics = Some(sender);
        self
    }

    /// Marks diagnostics as belonging to one turn of a research conversation.
    pub fn with_turn(mut self, turn_index: u32) -> Self {
        self.turn_index = Some(turn_index);
        self
    }

    fn send_issue(&self, mut issue: LlmIssue) {
        issue.turn_index = issue.turn_index.or(self.turn_index);
        issue.at = issue.at.or_else(|| Some(now_ms()));
        if let Some(sender) = &self.diagnostics {
            let _ = sender.send(issue);
        }
    }

    pub fn with_accounting(mut self, observer: Arc<dyn UsageObserver>) -> Self {
        self.accounting = Some(observer);
        self
    }

    fn manual_pricing(&self, model: &str) -> Option<PriceQuote> {
        let (provider, endpoint) = match self.config.backend {
            LlmBackend::OpenRouter => ("openrouter", "https://openrouter.ai/api/v1"),
            LlmBackend::Ollama => ("ollama", self.config.ollama_url.as_str()),
            LlmBackend::OllamaCloud => ("ollama_cloud", self.config.ollama_cloud_url.as_str()),
            LlmBackend::OpenAiCompatible => {
                ("openai_compatible", self.config.openai_base_url.as_str())
            }
        };
        let key = serde_json::to_string(&(
            provider,
            endpoint.trim().trim_end_matches('/'),
            model.trim(),
        ))
        .ok()?;
        let values: std::collections::HashMap<String, PriceQuote> =
            serde_json::from_str(&self.config.pricing_overrides).ok()?;
        let mut quote = values.get(&key)?.clone();
        if !quote.valid() {
            return None;
        }
        quote.source = "Manual model rates".into();
        Some(quote)
    }

    async fn accounted_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let requested_model = request.model.clone();
        let started = chrono::Utc::now();
        let guard = UsageGuard::begin(
            self.accounting.clone(),
            RequestInfo {
                kind: RequestKind::Llm,
                provider: self.safe_diagnostic_text(self.provider_name(), 64),
                model: self.safe_diagnostic_text(&requested_model, 200),
            },
        )
        .map_err(|_| LlmError::BudgetExceeded)?;
        let result = self.provider.chat_completion(request).await;
        let usage = match &result {
            Ok(response) => response.usage.clone(),
            Err(error) => error.usage(),
        };
        let mut recorded_usage = usage.clone();
        recorded_usage.model = usage
            .remote_model
            .as_deref()
            .or(usage.model.as_deref())
            .map(|model| self.safe_diagnostic_text(model, 200));
        guard.capture(recorded_usage);
        let pricing = if self.accounting.is_some() && usage.cost_usd.is_none() {
            match self.manual_pricing(&requested_model) {
                Some(quote) => Some(quote),
                None if usage.has_measurements() => {
                    self.provider
                        .pricing(&requested_model, &usage, started)
                        .await
                }
                None => None,
            }
        } else {
            None
        };
        guard.finish(pricing);
        result
    }

    /// Report a role's parse/shape failure without moving its fallback policy
    /// into the transport client. Callers must not include raw model content.
    pub fn report_invalid_response(
        &self,
        stage: &str,
        message: &str,
        response: &CompletionResponse,
        outcome: IssueOutcome,
    ) {
        let issue = LlmIssue {
            code: "invalid_response".into(),
            provider: self.safe_diagnostic_text(self.provider_name(), 64),
            model: self.safe_diagnostic_text(&response.model, 200),
            stage: Some(self.safe_diagnostic_text(stage, 80)),
            message: self.safe_diagnostic_text(message, 1200),
            max_tokens: self.config.max_tokens,
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            reasoning_tokens: response.usage.reasoning_tokens,
            retry_after_ms: None,
            attempt: 1,
            max_attempts: 1,
            will_retry: false,
            outcome: Some(outcome.as_str().into()),
            call_id: Some(crate::utils::id::new_id()),
            ..Default::default()
        };
        warn!(provider = %issue.provider, model = %issue.model, stage = ?issue.stage,
            "[FIX:llm-diagnostics] Role could not parse the model response");
        self.send_issue(issue);
    }

    pub async fn complete_for_stage(
        &self,
        stage: &str,
        messages: Vec<Message>,
        json_mode: bool,
    ) -> Result<CompletionResponse, LlmError> {
        self.complete_with_stage(Some(stage), messages, json_mode)
            .await
    }

    async fn complete_with_stage(
        &self,
        stage: Option<&str>,
        messages: Vec<Message>,
        json_mode: bool,
    ) -> Result<CompletionResponse, LlmError> {
        let model = match self.config.backend {
            LlmBackend::OpenRouter => &self.config.openrouter_model,
            LlmBackend::Ollama => &self.config.ollama_model,
            LlmBackend::OllamaCloud => &self.config.ollama_cloud_model,
            LlmBackend::OpenAiCompatible => &self.config.openai_model,
        };

        let request = CompletionRequest {
            messages,
            model: model.trim().to_string(),
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
            json_mode,
            reasoning_effort: self.config.reasoning_effort,
        };

        self.complete_with_retry(request, stage).await
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
            reasoning_effort: self.config.reasoning_effort,
        };

        self.complete_with_retry(request, None).await
    }

    /// Internal: execute a completion request with retry on transient errors.
    async fn complete_with_retry(
        &self,
        request: CompletionRequest,
        stage: Option<&str>,
    ) -> Result<CompletionResponse, LlmError> {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
        };
        let max_attempts = config.max_retries + 1;
        let mut attempt = 0;
        let call_id = crate::utils::id::new_id();
        // The last failed attempt, repeated as "recovered" if a retry succeeds.
        let last_failure = std::sync::Mutex::new(None::<LlmIssue>);
        retry_with_backoff(&config, "llm_complete", || {
            attempt += 1;
            let current_attempt = attempt;
            let req = request.clone();
            let request = &request;
            let call_id = &call_id;
            let last_failure = &last_failure;
            async move {
                match self.accounted_completion(req).await {
                    Ok(resp) => {
                        if let Some(failure) = last_failure.lock().unwrap().take() {
                            self.send_issue(LlmIssue {
                                attempt: current_attempt,
                                will_retry: false,
                                retry_after_ms: None,
                                outcome: Some(IssueOutcome::Recovered.as_str().into()),
                                at: None,
                                ..failure
                            });
                        }
                        (Ok(resp), RetryAction::Success, None)
                    }
                    Err(error) => {
                        let will_retry = error.retryable() && current_attempt < max_attempts;
                        let retry_after_ms = error.retry_after_ms().map(|ms| ms.min(60_000));
                        let usage = error.usage();
                        let outcome = if will_retry {
                            Some(IssueOutcome::Retrying)
                        } else {
                            IssueOutcome::after_failure(stage, error.code())
                        };
                        let issue = LlmIssue {
                            code: error.code().into(),
                            provider: self.safe_diagnostic_text(self.provider_name(), 64),
                            model: self.safe_diagnostic_text(&request.model, 200),
                            stage: stage.map(|s| self.safe_diagnostic_text(s, 80)),
                            message: self.safe_diagnostic_text(&error.to_string(), 1200),
                            max_tokens: request.max_tokens,
                            prompt_tokens: usage.prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                            reasoning_tokens: usage.reasoning_tokens,
                            retry_after_ms,
                            attempt: current_attempt,
                            max_attempts,
                            will_retry,
                            outcome: outcome.map(|o| o.as_str().into()),
                            call_id: Some(call_id.clone()),
                            ..Default::default()
                        };
                        warn!(code = %issue.code, provider = %issue.provider, model = %issue.model,
                            stage = ?issue.stage, attempt = current_attempt, max_attempts, will_retry,
                            "[FIX:llm-diagnostics] Provider attempt failed");
                        if will_retry {
                            *last_failure.lock().unwrap() = Some(issue.clone());
                        }
                        self.send_issue(issue);
                        let action = if will_retry { RetryAction::Retry } else { RetryAction::Fail };
                        (Err(error), action, retry_after_ms.map(Duration::from_millis))
                    }
                }
            }
        }).await
    }

    fn safe_diagnostic_text(&self, value: &str, max_chars: usize) -> String {
        safe_diagnostic_text(&self.config, value, max_chars)
    }

    /// Check if the configured provider is healthy.
    pub async fn health_check(&self) -> Result<(), LlmError> {
        self.provider.health_check().await
    }

    /// Get the currently active provider name.
    pub fn spending_limit_reached(&self) -> bool {
        self.accounting
            .as_ref()
            .is_some_and(|observer| observer.exceeded())
    }

    pub fn provider_name(&self) -> &str {
        self.provider.provider_name()
    }

    /// Get the current configuration.
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }

    /// Create an LlmManager with a custom provider (for testing).
    pub fn with_provider(provider: Arc<dyn LlmProvider>, config: LlmConfig) -> Self {
        Self {
            provider,
            config,
            diagnostics: None,
            accounting: None,
            turn_index: None,
        }
    }

    /// Build LlmConfig from settings stored in the database.
    pub fn config_from_settings(settings: &std::collections::HashMap<String, String>) -> LlmConfig {
        let backend = match settings.get("llm_provider").map(|s| s.as_str()) {
            Some("ollama") => LlmBackend::Ollama,
            Some("ollama_cloud") => LlmBackend::OllamaCloud,
            Some("openai_compatible") => LlmBackend::OpenAiCompatible,
            _ => LlmBackend::OpenRouter,
        };

        let defaults = LlmConfig::default();
        LlmConfig {
            backend,
            pricing_overrides: settings
                .get("llm_pricing_overrides")
                .cloned()
                .unwrap_or_else(|| "{}".into()),
            openrouter_api_key: settings
                .get("openrouter_api_key")
                .cloned()
                .unwrap_or_default(),
            openrouter_model: settings
                .get("openrouter_model")
                .cloned()
                .unwrap_or_else(|| "openai/gpt-4.1-mini".to_string()),
            ollama_url: settings
                .get("ollama_url")
                .cloned()
                .unwrap_or_else(|| "http://localhost:11434".to_string()),
            ollama_model: settings
                .get("ollama_model")
                .cloned()
                .unwrap_or_else(|| "llama3".to_string()),
            ollama_cloud_url: settings
                .get("ollama_cloud_url")
                .cloned()
                .unwrap_or(defaults.ollama_cloud_url),
            ollama_cloud_api_key: settings
                .get("ollama_cloud_api_key")
                .cloned()
                .unwrap_or_default(),
            ollama_cloud_model: settings
                .get("ollama_cloud_model")
                .cloned()
                .unwrap_or(defaults.ollama_cloud_model),
            openai_base_url: settings
                .get("openai_base_url")
                .cloned()
                .unwrap_or(defaults.openai_base_url),
            openai_api_key: settings.get("openai_api_key").cloned().unwrap_or_default(),
            openai_model: settings.get("openai_model").cloned().unwrap_or_default(),
            openai_json_mode: settings
                .get("openai_json_mode")
                .map(|v| v != "false")
                .unwrap_or(true),
            temperature: settings
                .get("llm_temperature")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.2),
            reasoning_effort: settings
                .get("llm_reasoning_effort")
                .map(|value| ReasoningEffort::from_setting(value))
                .unwrap_or_default(),
            max_tokens: settings
                .get("llm_max_tokens")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
        }
    }
}

fn safe_diagnostic_text(config: &LlmConfig, value: &str, max_chars: usize) -> String {
    let mut sanitized = value.to_string();
    for secret in [
        &config.openrouter_api_key,
        &config.ollama_cloud_api_key,
        &config.openai_api_key,
    ] {
        if !secret.trim().is_empty() {
            sanitized = sanitized.replace(secret.trim(), "[REDACTED]");
        }
    }
    sanitized
        .chars()
        .filter(|ch| !ch.is_control() || *ch == '\n')
        .take(max_chars)
        .collect()
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
        settings.insert(
            "openrouter_model".to_string(),
            "anthropic/claude-3.5-sonnet".to_string(),
        );
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
