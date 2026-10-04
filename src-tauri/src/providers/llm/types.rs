use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Role of a message in a chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// A single message in a chat completion conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Provider-neutral reasoning preference; Auto keeps Query2Table's JSON defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    #[default]
    Auto,
    #[serde(rename = "default")]
    ProviderDefault,
    Off,
    On,
    Low,
    Medium,
    High,
    Max,
}

impl ReasoningEffort {
    pub fn from_setting(value: &str) -> Self {
        match value {
            "default" => Self::ProviderDefault,
            "off" => Self::Off,
            "on" => Self::On,
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "max" => Self::Max,
            _ => Self::Auto,
        }
    }

    pub fn level(self) -> Option<&'static str> {
        match self {
            Self::Low => Some("low"),
            Self::Medium => Some("medium"),
            Self::High => Some("high"),
            Self::Max => Some("max"),
            _ => None,
        }
    }
}

/// Configuration for a chat completion request.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub json_mode: bool,
    pub reasoning_effort: ReasoningEffort,
}

#[derive(Debug, Clone, Default)]
pub struct LlmUsage {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
    pub cached_prompt_tokens: Option<u32>,
    pub cache_write_tokens: Option<u32>,
    pub cost_usd: Option<f64>,
    pub model: Option<String>,
    pub remote_model: Option<String>,
    pub remote_host: Option<String>,
}

impl LlmUsage {
    pub fn from_json(value: &serde_json::Value, openrouter_cost: bool) -> Self {
        let count = |pointer: &str| {
            value
                .pointer(pointer)
                .and_then(|v| v.as_u64())
                .and_then(|v| u32::try_from(v).ok())
        };
        let cost = value
            .pointer("/usage/cost_usd")
            .or_else(|| {
                openrouter_cost
                    .then(|| value.pointer("/usage/cost"))
                    .flatten()
            })
            .and_then(|value| value.as_f64())
            .filter(|cost| cost.is_finite() && *cost >= 0.0);
        Self {
            prompt_tokens: count("/usage/prompt_tokens").or_else(|| count("/prompt_eval_count")),
            completion_tokens: count("/usage/completion_tokens").or_else(|| count("/eval_count")),
            reasoning_tokens: count("/usage/completion_tokens_details/reasoning_tokens"),
            cached_prompt_tokens: count("/usage/prompt_tokens_details/cached_tokens")
                .or_else(|| count("/prompt_eval_cached_count")),
            cache_write_tokens: count("/usage/prompt_tokens_details/cache_write_tokens"),
            cost_usd: cost,
            model: value
                .get("model")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            remote_model: value
                .get("remote_model")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
            remote_host: value
                .get("remote_host")
                .and_then(|v| v.as_str())
                .map(str::to_owned),
        }
    }
    pub fn has_measurements(&self) -> bool {
        self.prompt_tokens.is_some()
            || self.completion_tokens.is_some()
            || self.cost_usd.is_some()
            || self.reasoning_tokens.is_some()
            || self.cached_prompt_tokens.is_some()
            || self.cache_write_tokens.is_some()
    }
}

/// What a failed request meant for the run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueOutcome {
    /// Another attempt follows.
    Retrying,
    /// A later attempt of the same request succeeded.
    Recovered,
    /// The step worked around it without losing anything.
    Continued,
    /// A simpler method replaced the model's work.
    Fallback,
    /// Part of the result was dropped.
    Skipped,
    /// The run ended at this step.
    Stopped,
}

impl IssueOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retrying => "retrying",
            Self::Recovered => "recovered",
            Self::Continued => "continued",
            Self::Fallback => "fallback",
            Self::Skipped => "skipped",
            Self::Stopped => "stopped",
        }
    }

    /// What a stage does once a request has finally failed.
    pub fn after_failure(stage: Option<&str>, code: &str) -> Option<Self> {
        if code == "budget_limit" {
            return Some(Self::Stopped);
        }
        match stage? {
            "interpreter" | "planner" | "schema_planner" | "search_planner" | "query_expander"
            | "setup" => Some(Self::Stopped),
            "extractor" | "link_ranker" => Some(Self::Skipped),
            "image_ranker" | "image_search_planner" | "link_search_planner" => Some(Self::Fallback),
            "research" => Some(Self::Continued),
            _ => None,
        }
    }
}

/// A bounded, secret-free diagnostic emitted for every failed provider attempt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmIssue {
    pub code: String,
    pub provider: String,
    pub model: String,
    pub stage: Option<String>,
    pub message: String,
    pub max_tokens: u32,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub reasoning_tokens: Option<u32>,
    pub retry_after_ms: Option<u64>,
    pub attempt: u32,
    pub max_attempts: u32,
    pub will_retry: bool,
    /// See [`IssueOutcome`]; missing in issues recorded before outcomes existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Shared by the attempts of one request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    /// Research conversation turn the request belonged to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_index: Option<u32>,
    /// Unix time in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<i64>,
}

/// Response from a chat completion.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub usage: LlmUsage,
}

/// Error type for LLM provider operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum LlmError {
    #[error("Run spending limit reached. Increase the run's Max Cost to send more requests.")]
    BudgetExceeded,

    #[error("{error}")]
    WithUsage {
        error: Box<LlmError>,
        usage: Box<LlmUsage>,
    },
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid API key or authentication failed")]
    AuthError,

    #[error("Rate limited, retry after {retry_after_ms}ms")]
    RateLimited { retry_after_ms: u64 },

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Response parse error: {0}")]
    ParseError(String),

    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Access denied by the provider. Check model access, account permissions, and regional or network restrictions.")]
    AccessDenied,

    #[error("Provider quota or credit limit reached. Check your account balance and usage limits before retrying.")]
    QuotaExceeded,

    #[error("The request exceeds the model context limit. Shorten the input or choose a model with a larger context window.")]
    ContextLimit,

    #[error("Unsupported model setting: {0}")]
    UnsupportedSetting(String),

    #[error("The model reached the output token limit ({limit}) before finishing its answer. Increase Max Tokens, reduce reasoning effort, or request a smaller output.")]
    OutputLimit { limit: u32, usage: Box<LlmUsage> },

    #[error("The model returned an empty answer. Try a lower reasoning effort or a larger Max Tokens limit.")]
    EmptyResponse { usage: Box<LlmUsage> },

    #[error("The provider request timed out. Check the connection and model availability, or try a faster model.")]
    Timeout,

    #[error("Provider error: {message}")]
    ProviderError { message: String, retryable: bool },
}

impl LlmError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BudgetExceeded => "budget_limit",
            Self::WithUsage { error, .. } => error.code(),
            Self::OutputLimit { .. } => "output_limit",
            Self::ContextLimit => "context_limit",
            Self::RateLimited { .. } => "rate_limit",
            Self::QuotaExceeded => "quota",
            Self::AuthError => "auth",
            Self::AccessDenied => "access_denied",
            Self::Timeout => "timeout",
            Self::ConnectionError(_) => "connection",
            Self::ParseError(_) => "invalid_response",
            Self::EmptyResponse { .. } => "empty_response",
            Self::NotConfigured(_) => "not_configured",
            Self::UnsupportedSetting(_) => "unsupported_setting",
            Self::ModelNotFound(_) => "model_not_found",
            Self::RequestFailed(_) | Self::ProviderError { .. } => "provider_error",
        }
    }

    pub fn retryable(&self) -> bool {
        if let Self::WithUsage { error, .. } = self {
            return error.retryable();
        }
        matches!(
            self,
            Self::RateLimited { .. }
                | Self::ConnectionError(_)
                | Self::Timeout
                | Self::RequestFailed(_)
                | Self::ProviderError {
                    retryable: true,
                    ..
                }
        )
    }

    pub fn usage(&self) -> LlmUsage {
        match self {
            Self::WithUsage { usage, .. }
            | Self::OutputLimit { usage, .. }
            | Self::EmptyResponse { usage } => usage.as_ref().clone(),
            _ => LlmUsage::default(),
        }
    }

    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            Self::WithUsage { error, .. } => error.retry_after_ms(),
            Self::RateLimited { retry_after_ms } => Some(*retry_after_ms),
            _ => None,
        }
    }

    pub fn with_usage(self, usage: LlmUsage) -> Self {
        if usage.has_measurements() {
            Self::WithUsage {
                error: Box::new(self),
                usage: Box::new(usage),
            }
        } else {
            self
        }
    }
}

/// Trait that all LLM providers must implement.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn pricing(
        &self,
        _model: &str,
        _usage: &LlmUsage,
        _at: chrono::DateTime<chrono::Utc>,
    ) -> Option<crate::providers::accounting::PriceQuote> {
        None
    }
    /// Send a chat completion request and return the response.
    async fn chat_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError>;

    /// Return the provider name for logging.
    fn provider_name(&self) -> &str;

    /// Check if the provider is configured and ready to use.
    async fn health_check(&self) -> Result<(), LlmError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_constructors() {
        let sys = Message::system("You are a helpful assistant");
        assert!(matches!(sys.role, MessageRole::System));

        let usr = Message::user("Hello");
        assert!(matches!(usr.role, MessageRole::User));

        let ast = Message::assistant("Hi there");
        assert!(matches!(ast.role, MessageRole::Assistant));
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"test\""));
    }

    #[test]
    fn test_completion_request() {
        let req = CompletionRequest {
            messages: vec![Message::system("sys"), Message::user("hi")],
            model: "gpt-4.1-mini".to_string(),
            temperature: 0.7,
            max_tokens: 4096,
            json_mode: true,
            reasoning_effort: ReasoningEffort::Auto,
        };
        assert_eq!(req.messages.len(), 2);
        assert!(req.json_mode);
    }
}
