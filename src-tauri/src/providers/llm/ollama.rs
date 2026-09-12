use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::endpoint::{normalize_base_url, status_error};
use super::types::*;
use crate::providers::http::apply_proxy;

/// Native Ollama API, used for local servers and direct Ollama Cloud access.
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    api_key: String,
    cloud: bool,
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Result<Self, LlmError> {
        Self::build(base_url, String::new(), false)
    }

    pub fn cloud(base_url: String, api_key: String) -> Result<Self, LlmError> {
        if api_key.trim().is_empty() {
            return Err(LlmError::NotConfigured(
                "Ollama Cloud API key is required".into(),
            ));
        }
        Self::build(base_url, api_key, true)
    }

    fn build(base_url: String, api_key: String, cloud: bool) -> Result<Self, LlmError> {
        let base_url = normalize_base_url(&base_url, "")?;
        let builder = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(10));
        let client = apply_proxy(builder)
            .build()
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;
        Ok(Self {
            client,
            base_url: base_url.trim_end_matches("/api").to_string(),
            api_key: api_key.trim().to_string(),
            cloud,
        })
    }

    fn authenticate(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.api_key.is_empty() {
            request
        } else {
            request.bearer_auth(&self.api_key)
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
    async fn chat_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let mut messages: Vec<OllamaMessage> = request
            .messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        // Cloud does not support the native format parameter; keep the JSON contract in the prompt.
        // https://docs.ollama.com/capabilities/structured-outputs
        let cloud_model =
            self.cloud || request.model.ends_with(":cloud") || request.model.ends_with("-cloud");
        if request.json_mode && cloud_model {
            messages.insert(0, OllamaMessage {
                role: "system".into(),
                content: "Return only valid JSON matching the requested structure, without Markdown fences or commentary.".into(),
            });
        }
        let format = if request.json_mode && !cloud_model {
            Some("json".to_string())
        } else {
            None
        };

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

        debug!(provider = self.provider_name(), model = %request.model, json_mode = request.json_mode, "Ollama chat_completion");

        let resp = self
            .authenticate(self.client.post(format!("{}/api/chat", self.base_url)))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;

        let status = resp.status();
        if let Some(error) = status_error(&resp) {
            return Err(error);
        }

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
            return Err(LlmError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let chat_resp: OllamaChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = chat_resp
            .message
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
        if self.cloud {
            "ollama_cloud"
        } else {
            "ollama"
        }
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        let resp = self
            .authenticate(self.client.get(format!("{}/api/tags", self.base_url)))
            .send()
            .await
            .map_err(|e| {
                LlmError::ConnectionError(format!(
                    "Cannot connect to Ollama at {}: {}",
                    self.base_url, e
                ))
            })?;

        if let Some(error) = status_error(&resp) {
            return Err(error);
        }
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
        let provider = OllamaProvider::new("http://localhost:11434".to_string()).unwrap();
        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.base_url, "http://localhost:11434");
    }

    #[test]
    fn test_ollama_chat_request_serialization() {
        let body = OllamaChatRequest {
            model: "llama3".to_string(),
            messages: vec![OllamaMessage {
                role: "user".to_string(),
                content: "hi".to_string(),
            }],
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: 4096,
            },
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
            options: OllamaOptions {
                temperature: 0.0,
                num_predict: 100,
            },
            format: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("format"));
    }
}
