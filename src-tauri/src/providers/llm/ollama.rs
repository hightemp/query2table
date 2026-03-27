use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::types::*;

/// Ollama local LLM provider.
pub struct OllamaProvider {
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build Ollama HTTP client");
        Self {
            client,
            base_url,
        }
    }
}

// --- Ollama API types ---

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: Option<OllamaResponseMessage>,
    model: Option<String>,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct OllamaErrorResponse {
    error: Option<String>,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let messages: Vec<OllamaMessage> = request.messages.iter().map(|m| OllamaMessage {
            role: match m.role {
                MessageRole::System => "system".to_string(),
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: m.content.clone(),
        }).collect();

        let format = if request.json_mode { Some("json".to_string()) } else { None };

        let body = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            options: OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            },
            format,
        };

        debug!(model = %request.model, json_mode = request.json_mode, "Ollama chat_completion");

        let resp = self.client
            .post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;

        let status = resp.status();

        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            if let Ok(err_resp) = serde_json::from_str::<OllamaErrorResponse>(&error_text) {
                if let Some(err) = err_resp.error {
                    if err.contains("not found") || err.contains("no such model") {
                        return Err(LlmError::ModelNotFound(request.model));
                    }
                    return Err(LlmError::RequestFailed(err));
                }
            }
            return Err(LlmError::RequestFailed(format!("HTTP {}: {}", status, error_text)));
        }

        let chat_resp: OllamaChatResponse = resp.json().await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = chat_resp.message
            .and_then(|m| m.content)
            .ok_or_else(|| LlmError::ParseError("No message content in response".to_string()))?;

        let prompt_tokens = chat_resp.prompt_eval_count.unwrap_or(0);
        let completion_tokens = chat_resp.eval_count.unwrap_or(0);

        Ok(CompletionResponse {
            content,
            model: chat_resp.model.unwrap_or(request.model),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        })
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        let resp = self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(format!(
                "Cannot connect to Ollama at {}: {}",
                self.base_url, e
            )))?;

        if !resp.status().is_success() {
            return Err(LlmError::ConnectionError(format!(
                "Ollama returned HTTP {}",
                resp.status()
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_creation() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string());
        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_chat_request_serialization() {
        let body = OllamaChatRequest {
            model: "llama3".to_string(),
            messages: vec![OllamaMessage { role: "user".to_string(), content: "hi".to_string() }],
            stream: false,
            options: OllamaOptions { temperature: 0.7, num_predict: 4096 },
            format: Some("json".to_string()),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"stream\":false"));
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"num_predict\":4096"));
    }

    #[test]
    fn test_ollama_no_json_format() {
        let body = OllamaChatRequest {
            model: "llama3".to_string(),
            messages: vec![],
            stream: false,
            options: OllamaOptions { temperature: 0.0, num_predict: 100 },
            format: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("format"));
    }
}
