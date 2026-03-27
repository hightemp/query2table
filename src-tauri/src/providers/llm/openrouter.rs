use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::types::*;

/// OpenRouter.ai LLM provider (OpenAI-compatible API).
pub struct OpenRouterProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to build OpenRouter HTTP client");
        Self {
            client,
            api_key,
            base_url: "https://openrouter.ai/api/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }
}

// --- OpenAI-compatible request/response types ---

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
    message: Option<String>,
    #[allow(dead_code)]
    code: Option<serde_json::Value>,
}

#[async_trait]
impl LlmProvider for OpenRouterProvider {
    async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let messages: Vec<ChatMessage> = request.messages.iter().map(|m| ChatMessage {
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: m.content.clone(),
        }).collect();

        let response_format = if request.json_mode {
            Some(ResponseFormat { format_type: "json_object".to_string() })
        } else {
            None
        };

        let body = ChatRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            response_format,
        };

        debug!(model = %request.model, json_mode = request.json_mode, "OpenRouter chat_completion");

        let resp = self.client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/query2table")
            .header("X-Title", "Query2Table")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;

        let status = resp.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(LlmError::AuthError);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(5000);
            return Err(LlmError::RateLimited { retry_after_ms: retry_after * 1000 });
        }

        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            if let Ok(err_resp) = serde_json::from_str::<ErrorResponse>(&error_text) {
                if let Some(err) = err_resp.error {
                    let msg = err.message.unwrap_or_else(|| "Unknown error".to_string());
                    if msg.contains("not found") || msg.contains("No model") {
                        return Err(LlmError::ModelNotFound(request.model));
                    }
                    return Err(LlmError::RequestFailed(msg));
                }
            }
            return Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, error_text)));
        }

        let chat_resp: ChatResponse = resp.json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = chat_resp.choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::ParseError("No choices in response".to_string()))?;

        let usage = chat_resp.usage.unwrap_or(Usage {
            prompt_tokens: Some(0),
            completion_tokens: Some(0),
            total_tokens: Some(0),
        });

        Ok(CompletionResponse {
            content,
            model: chat_resp.model.unwrap_or(request.model),
            prompt_tokens: usage.prompt_tokens.unwrap_or(0),
            completion_tokens: usage.completion_tokens.unwrap_or(0),
            total_tokens: usage.total_tokens.unwrap_or(0),
        })
    }

    fn provider_name(&self) -> &str {
        "openrouter"
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::NotConfigured("OpenRouter API key not set".to_string()));
        }

        let req = CompletionRequest {
            messages: vec![Message::user("ping")],
            model: "openai/gpt-4.1-mini".to_string(),
            temperature: 0.0,
            max_tokens: 1,
            json_mode: false,
        };

        match self.chat_completion(req).await {
            Ok(_) => Ok(()),
            Err(LlmError::AuthError) => Err(LlmError::AuthError),
            Err(LlmError::ConnectionError(e)) => Err(LlmError::ConnectionError(e)),
            Err(_) => {
                warn!("OpenRouter health check returned non-critical error, treating as OK");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_creation() {
        let provider = OpenRouterProvider::new("test-key".to_string());
        assert_eq!(provider.provider_name(), "openrouter");
        assert_eq!(provider.base_url, "https://openrouter.ai/api/v1");
    }

    #[test]
    fn test_openrouter_custom_base_url() {
        let provider = OpenRouterProvider::new("key".to_string())
            .with_base_url("http://localhost:8080".to_string());
        assert_eq!(provider.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_chat_request_serialization() {
        let body = ChatRequest {
            model: "openai/gpt-4.1-mini".to_string(),
            messages: vec![ChatMessage { role: "user".to_string(), content: "hi".to_string() }],
            temperature: 0.7,
            max_tokens: 4096,
            response_format: Some(ResponseFormat { format_type: "json_object".to_string() }),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("json_object"));
        assert!(json.contains("gpt-4.1-mini"));
    }

    #[test]
    fn test_chat_request_no_json_mode() {
        let body = ChatRequest {
            model: "test".to_string(),
            messages: vec![],
            temperature: 0.0,
            max_tokens: 100,
            response_format: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("response_format"));
    }
}
