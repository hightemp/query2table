use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::endpoint::{normalize_base_url, response_json, transport_error};

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
        let client = apply_proxy(builder).build().map_err(transport_error)?;
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
        let response = request.send().await.map_err(transport_error)?;
        let catalog: ModelList = serde_json::from_value(response_json(response, "").await?)
            .map_err(|_| {
                LlmError::ParseError("The provider returned an invalid model catalog.".into())
            })?;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'static str>,
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
    message: Option<ChoiceMessage>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Default, Deserialize)]
struct Usage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
    total_tokens: Option<u32>,
    completion_tokens_details: Option<CompletionTokenDetails>,
}

#[derive(Deserialize)]
struct CompletionTokenDetails {
    reasoning_tokens: Option<u32>,
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

        let (reasoning, reasoning_effort) = if self.name == "openrouter" {
            let reasoning = match request.reasoning_effort {
                ReasoningEffort::Auto | ReasoningEffort::ProviderDefault => None,
                ReasoningEffort::Off => Some(serde_json::json!({"enabled": false})),
                ReasoningEffort::On => Some(serde_json::json!({"enabled": true})),
                effort => effort
                    .level()
                    .map(|level| serde_json::json!({"effort": level})),
            };
            (reasoning, None)
        } else {
            let level = match request.reasoning_effort {
                ReasoningEffort::Auto | ReasoningEffort::ProviderDefault => None,
                ReasoningEffort::Off => Some("none"),
                ReasoningEffort::On => Some("medium"),
                effort => effort.level(),
            };
            (None, level)
        };
        let body = ChatRequest {
            model: request.model.clone(),
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            response_format,
            reasoning,
            reasoning_effort,
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
        let resp = http_request.send().await.map_err(transport_error)?;
        let chat_resp: ChatResponse =
            serde_json::from_value(response_json(resp, &request.model).await?).map_err(|_| {
                LlmError::ParseError("The provider returned an invalid completion response.".into())
            })?;
        let usage = chat_resp.usage.unwrap_or_default();
        let counts = LlmUsage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            reasoning_tokens: usage
                .completion_tokens_details
                .as_ref()
                .and_then(|details| details.reasoning_tokens),
        };
        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::ParseError("The provider returned no choices.".into()))?;
        if choice.finish_reason.as_deref() == Some("length") {
            return Err(LlmError::OutputLimit {
                limit: request.max_tokens,
                usage: counts,
            });
        }
        let content = choice
            .message
            .and_then(|message| message.content)
            .unwrap_or_default();
        if content.trim().is_empty() {
            return Err(LlmError::EmptyResponse { usage: counts });
        }

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
        let response = request.send().await.map_err(transport_error)?;
        response_json(response, "").await?;
        Ok(())
    }
}
