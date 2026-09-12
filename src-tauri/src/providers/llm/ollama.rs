use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::endpoint::{normalize_base_url, response_json, transport_error};
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
        let client = apply_proxy(builder).build().map_err(transport_error)?;
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

    /// Fetch model IDs from the server's native catalog.
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        debug!(
            provider = self.provider_name(),
            "Loading Ollama model catalog"
        );
        let response = self
            .authenticate(self.client.get(format!("{}/api/tags", self.base_url)))
            .timeout(std::time::Duration::from_secs(20))
            .send()
            .await
            .map_err(transport_error)?;
        let catalog: OllamaModelList = serde_json::from_value(response_json(response, "").await?)
            .map_err(|_| {
            LlmError::ParseError("Ollama returned an invalid model catalog.".into())
        })?;
        let mut models: Vec<String> = catalog
            .models
            .into_iter()
            .map(|model| model.name)
            .filter(|name| !name.trim().is_empty())
            .collect();
        models.sort_unstable();
        models.dedup();
        debug!(count = models.len(), "Ollama model catalog loaded");
        Ok(models)
    }
}

// --- Ollama API types ---

#[derive(Deserialize)]
struct OllamaModelList {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    think: Option<OllamaThinking>,
}

#[derive(Serialize)]
#[serde(untagged)]
enum OllamaThinking {
    Enabled(bool),
    Level(&'static str),
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
    done_reason: Option<String>,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: Option<String>,
    thinking: Option<String>,
}

pub(super) fn validate_reasoning(model: &str, effort: ReasoningEffort) -> Result<(), LlmError> {
    let gpt_oss = model
        .rsplit('/')
        .next()
        .unwrap_or(model)
        .starts_with("gpt-oss");
    if gpt_oss && matches!(effort, ReasoningEffort::Off | ReasoningEffort::Max) {
        return Err(LlmError::UnsupportedSetting(
            "Ollama GPT-OSS supports Low, Medium, and High reasoning. Off and Max are not supported; choose a supported level or Provider default.".into()
        ));
    }
    Ok(())
}

fn reasoning_setting(request: &CompletionRequest) -> Result<Option<OllamaThinking>, LlmError> {
    validate_reasoning(&request.model, request.reasoning_effort)?;
    let gpt_oss = request
        .model
        .rsplit('/')
        .next()
        .unwrap_or(&request.model)
        .starts_with("gpt-oss");
    match request.reasoning_effort {
        ReasoningEffort::Auto if request.json_mode => Ok(Some(if gpt_oss {
            OllamaThinking::Level("low")
        } else {
            OllamaThinking::Enabled(false)
        })),
        ReasoningEffort::Auto | ReasoningEffort::ProviderDefault => Ok(None),
        ReasoningEffort::Off => Ok(Some(OllamaThinking::Enabled(false))),
        ReasoningEffort::On if gpt_oss => Ok(Some(OllamaThinking::Level("medium"))),
        ReasoningEffort::On => Ok(Some(OllamaThinking::Enabled(true))),
        level => Ok(level.level().map(OllamaThinking::Level)),
    }
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

        // Explicit user settings take precedence over automatic JSON defaults.
        let think = reasoning_setting(&request)?;

        let body = OllamaChatRequest {
            model: request.model.clone(),
            messages,
            stream: false,
            options: OllamaOptions {
                temperature: request.temperature,
                num_predict: request.max_tokens,
            },
            format,
            think,
        };

        debug!(provider = self.provider_name(), model = %request.model, json_mode = request.json_mode, "Ollama chat_completion");

        let started = std::time::Instant::now();

        let resp = self
            .authenticate(self.client.post(format!("{}/api/chat", self.base_url)))
            .json(&body)
            .send()
            .await
            .map_err(transport_error)?;

        let chat_resp: OllamaChatResponse =
            serde_json::from_value(response_json(resp, &request.model).await?).map_err(|_| {
                LlmError::ParseError("Ollama returned an invalid completion response.".into())
            })?;

        let prompt_tokens = chat_resp.prompt_eval_count.unwrap_or(0);
        let completion_tokens = chat_resp.eval_count.unwrap_or(0);
        let done_reason = chat_resp.done_reason.as_deref().unwrap_or("unknown");
        let (content, thinking_chars) = match chat_resp.message {
            Some(message) => (
                message.content.unwrap_or_default(),
                message.thinking.as_deref().map(str::len).unwrap_or(0),
            ),
            None if done_reason == "length" => (String::new(), 0),
            None => {
                return Err(LlmError::ParseError(
                    "Ollama response contains no message".into(),
                ))
            }
        };
        debug!(model = %request.model, elapsed_ms = started.elapsed().as_millis(),
            prompt_tokens, completion_tokens, content_chars = content.len(), thinking_chars, done_reason,
            "[FIX:ollama-json] Completion received");
        if done_reason == "length" {
            warn!(model = %request.model, max_tokens = request.max_tokens, completion_tokens,
                thinking_chars, "[FIX:ollama-json] Token limit reached before a complete answer");
            return Err(LlmError::OutputLimit {
                limit: request.max_tokens,
                usage: LlmUsage {
                    prompt_tokens: chat_resp.prompt_eval_count,
                    completion_tokens: chat_resp.eval_count,
                    reasoning_tokens: None,
                },
            });
        }
        if content.trim().is_empty() {
            warn!(model = %request.model, completion_tokens, thinking_chars, done_reason,
                "[FIX:ollama-json] Model returned an empty answer");
            return Err(LlmError::EmptyResponse {
                usage: LlmUsage {
                    prompt_tokens: chat_resp.prompt_eval_count,
                    completion_tokens: chat_resp.eval_count,
                    reasoning_tokens: None,
                },
            });
        }

        Ok(CompletionResponse {
            content,
            model: chat_resp.model.unwrap_or(request.model),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
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
            .map_err(transport_error)?;
        response_json(resp, "").await?;

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
            think: Some(OllamaThinking::Enabled(false)),
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"stream\":false"));
        assert!(json.contains("\"format\":\"json\""));
        assert!(json.contains("\"num_predict\":4096"));
        assert!(json.contains("\"think\":false"));
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
            think: None,
        };
        let json = serde_json::to_string(&body).unwrap();
        assert!(!json.contains("format"));
    }
}
