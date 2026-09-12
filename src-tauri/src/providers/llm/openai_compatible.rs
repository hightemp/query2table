use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::endpoint::{normalize_base_url, status_error};

use super::types::*;
use crate::providers::http::apply_proxy;

/// Shared Chat Completions client for OpenRouter and OpenAI-compatible servers.
pub struct OpenAiCompatibleProvider {
    client: Client,
    api_key: String,
    base_url: String,
    name: &'static str,
    json_mode_enabled: bool,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        base_url: String,
        api_key: String,
        json_mode_enabled: bool,
    ) -> Result<Self, LlmError> {
        let base_url = normalize_base_url(&base_url, "/v1")?;
        let builder = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(15));
        let client = apply_proxy(builder)
            .build()
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;
        Ok(Self {
            client,
            api_key: api_key.trim().to_string(),
            base_url,
            name: "openai_compatible",
            json_mode_enabled,
        })
    }

    pub(super) fn for_openrouter(mut self) -> Self {
        self.name = "openrouter";
        self
    }

    pub(super) fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url.trim_end_matches('/').to_string();
        self
    }

    /// Fetch model IDs from an OpenAI-compatible catalog.
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        debug!(provider = self.name, "Loading model catalog");
        let mut request = self
            .client
            .get(format!("{}/models", self.base_url))
            .timeout(std::time::Duration::from_secs(20));
        if !self.api_key.is_empty() {
            request = request.bearer_auth(&self.api_key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;
        if let Some(error) = status_error(&response) {
            return Err(error);
        }
        if !response.status().is_success() {
            return Err(LlmError::RequestFailed(format!(
                "Model list returned HTTP {}",
                response.status()
            )));
        }
        let catalog: ModelList = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;
        let mut models: Vec<String> = catalog
            .data
            .into_iter()
            .map(|model| model.id)
            .filter(|id| !id.trim().is_empty())
            .collect();
        models.sort_unstable();
        models.dedup();
        debug!(
            provider = self.name,
            count = models.len(),
            "Model catalog loaded"
        );
        Ok(models)
    }
}

// --- OpenAI-compatible request/response types ---

#[derive(Deserialize)]
struct ModelList {
    data: Vec<CatalogModel>,
}

#[derive(Deserialize)]
struct CatalogModel {
    id: String,
}

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
impl LlmProvider for OpenAiCompatibleProvider {
    async fn chat_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let messages: Vec<ChatMessage> = request
            .messages
            .iter()
            .map(|m| ChatMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let response_format = if request.json_mode && self.json_mode_enabled {
            Some(ResponseFormat {
                format_type: "json_object".to_string(),
            })
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

        debug!(provider = self.name, model = %request.model, json_mode = request.json_mode, "LLM chat_completion");

        let mut http_request = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .json(&body);
        if !self.api_key.is_empty() {
            http_request = http_request.bearer_auth(&self.api_key);
        }
        if self.name == "openrouter" {
            http_request = http_request
                .header("HTTP-Referer", "https://github.com/query2table")
                .header("X-Title", "Query2Table");
        }
        let resp = http_request
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;
        let status = resp.status();
        if let Some(error) = status_error(&resp) {
            return Err(error);
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
            return Err(LlmError::RequestFailed(format!(
                "HTTP {}: {}",
                status, error_text
            )));
        }

        let chat_resp: ChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let content = chat_resp
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
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
            total_tokens: usage.total_tokens.unwrap_or_else(|| {
                usage
                    .prompt_tokens
                    .unwrap_or(0)
                    .saturating_add(usage.completion_tokens.unwrap_or(0))
            }),
        })
    }

    fn provider_name(&self) -> &str {
        self.name
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        let mut request = self.client.get(format!("{}/models", self.base_url));
        if !self.api_key.is_empty() {
            request = request.bearer_auth(&self.api_key);
        }
        let response = request
            .send()
            .await
            .map_err(|e| LlmError::ConnectionError(e.to_string()))?;
        if let Some(error) = status_error(&response) {
            return Err(error);
        }
        if !response.status().is_success() {
            return Err(LlmError::RequestFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }
        Ok(())
    }
}
