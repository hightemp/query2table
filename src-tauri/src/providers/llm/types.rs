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
        Self { role: MessageRole::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: MessageRole::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: MessageRole::Assistant, content: content.into() }
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
}

/// Response from a chat completion.
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Error type for LLM provider operations.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
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
}

/// Trait that all LLM providers must implement.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a chat completion request and return the response.
    async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError>;

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
        };
        assert_eq!(req.messages.len(), 2);
        assert!(req.json_mode);
    }
}
