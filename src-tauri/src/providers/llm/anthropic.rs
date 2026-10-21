//! Claude through Anthropic's own Messages API: prompt caching for long conversations (the
//! research agent resends its growing transcript), pictures and PDF documents, and exact usage
//! of cache reads and writes. https://platform.claude.com/docs/en/api/messages

use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;

use super::endpoint::{json_body, normalize_base_url, response_json, sized_response_json, transport_error};
use super::types::*;
use crate::providers::http::apply_proxy;

pub const BASE_URL: &str = "https://api.anthropic.com/v1";
const API_VERSION: &str = "2023-06-01";
/// Anthropic has no JSON mode without a schema; the contract goes into the system prompt.
const JSON_RULE: &str = "Return only valid JSON matching the requested structure, without Markdown fences or commentary.";

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(base_url: String, api_key: String) -> Result<Self, LlmError> {
        let base_url = normalize_base_url(&base_url, "/v1")?;
        let builder = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(15));
        let client = apply_proxy(builder).build().map_err(transport_error)?;
        Ok(Self { client, api_key: api_key.trim().to_string(), base_url })
    }

    fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client
            .get(format!("{}/{path}", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .timeout(std::time::Duration::from_secs(20))
    }

    /// Model ids from `/v1/models`, sorted.
    pub async fn list_models(&self) -> Result<Vec<String>, LlmError> {
        let response = self.get("models?limit=1000").send().await.map_err(transport_error)?;
        let catalog = response_json(response, "").await?;
        let mut models: Vec<String> = catalog
            .get("data")
            .and_then(Value::as_array)
            .ok_or_else(|| LlmError::ParseError("Anthropic returned an invalid model list.".into()))?
            .iter()
            .filter_map(|model| model.get("id")?.as_str().map(str::to_owned))
            .filter(|id| !id.trim().is_empty())
            .collect();
        models.sort_unstable();
        models.dedup();
        Ok(models)
    }
}

/// Claude 4.5 and earlier: thinking only with a token budget, and sampling settings accepted.
/// Later models take adaptive thinking with an effort level and reject other temperatures.
fn budget_thinking(model: &str) -> bool {
    let model = model.trim().to_ascii_lowercase();
    if model.starts_with("claude-3") {
        return true;
    }
    let Some(rest) = ["claude-opus-4", "claude-sonnet-4", "claude-haiku-4"].iter().find_map(|p| model.strip_prefix(p)) else {
        return false;
    };
    // `claude-opus-4`, `claude-opus-4-20250514`, `claude-opus-4-1`, `claude-sonnet-4-5-…`.
    let minor = rest.strip_prefix('-').and_then(|r| r.split('-').next()).unwrap_or("");
    rest.is_empty() || minor.len() == 8 || matches!(minor, "0" | "1" | "2" | "3" | "4" | "5")
}

/// `thinking`, `output_config` and `temperature` for the reasoning setting and model.
fn reasoning_fields(model: &str, effort: ReasoningEffort, max_tokens: u32, temperature: f32, body: &mut serde_json::Map<String, Value>) {
    if budget_thinking(model) {
        let budget = match effort {
            ReasoningEffort::Low => Some(2048),
            ReasoningEffort::On | ReasoningEffort::Medium => Some(8192),
            ReasoningEffort::High => Some(16384),
            ReasoningEffort::Max => Some(32000),
            _ => None,
        }
        // The budget must be at least 1024 and below max_tokens.
        .map(|budget: u32| budget.min(max_tokens.saturating_sub(1024)))
        .filter(|budget| *budget >= 1024);
        match budget {
            Some(budget) => {
                body.insert("thinking".into(), json!({"type": "enabled", "budget_tokens": budget}));
            }
            None => {
                body.insert("temperature".into(), json!(temperature));
            }
        }
        return;
    }
    match effort {
        ReasoningEffort::Auto | ReasoningEffort::ProviderDefault => {}
        // Some models think always; the least effort is the closest to "off" on every model.
        ReasoningEffort::Off => {
            body.insert("output_config".into(), json!({"effort": "low"}));
        }
        ReasoningEffort::On => {
            body.insert("thinking".into(), json!({"type": "adaptive"}));
        }
        level => {
            body.insert("thinking".into(), json!({"type": "adaptive"}));
            if let Some(level) = level.level() {
                body.insert("output_config".into(), json!({"effort": level}));
            }
        }
    }
}

fn content(message: &Message) -> Value {
    if message.images.is_empty() && message.documents.is_empty() {
        return Value::String(message.content.clone());
    }
    // Files first, then the question about them.
    let mut blocks: Vec<Value> = message
        .documents
        .iter()
        .map(|d| json!({"type": "document", "source": {"type": "base64", "media_type": d.media_type, "data": d.data_base64}}))
        .collect();
    blocks.extend(
        message
            .images
            .iter()
            .map(|i| json!({"type": "image", "source": {"type": "base64", "media_type": i.media_type, "data": i.data_base64}})),
    );
    if !message.content.trim().is_empty() {
        blocks.push(json!({"type": "text", "text": message.content}));
    }
    Value::Array(blocks)
}

/// Text inside a ```json fence, for answers that should have been bare JSON.
fn without_fence(text: &str) -> &str {
    let trimmed = text.trim();
    let Some(inner) = trimmed.strip_prefix("```") else {
        return text;
    };
    let inner = inner.strip_prefix("json").unwrap_or(inner);
    inner.strip_suffix("```").map(str::trim).unwrap_or(text)
}

fn usage(value: &Value) -> LlmUsage {
    let count = |name: &str| value.pointer(&format!("/usage/{name}")).and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok());
    let input = count("input_tokens");
    let read = count("cache_read_input_tokens");
    let written = count("cache_creation_input_tokens");
    LlmUsage {
        // `input_tokens` counts only what was neither read from nor written to the cache.
        prompt_tokens: input.map(|input| input.saturating_add(read.unwrap_or(0)).saturating_add(written.unwrap_or(0))),
        completion_tokens: count("output_tokens"),
        reasoning_tokens: value.pointer("/usage/output_tokens_details/thinking_tokens").and_then(Value::as_u64).and_then(|v| u32::try_from(v).ok()),
        cached_prompt_tokens: read,
        cache_write_tokens: written,
        model: value.get("model").and_then(Value::as_str).map(str::to_owned),
        ..Default::default()
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let mut system: Vec<&str> = request
            .messages
            .iter()
            .filter(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.as_str())
            .collect();
        if request.json_mode {
            system.push(JSON_RULE);
        }
        let messages: Vec<Value> = request
            .messages
            .iter()
            .filter(|m| !matches!(m.role, MessageRole::System))
            .map(|m| {
                let role = if matches!(m.role, MessageRole::Assistant) { "assistant" } else { "user" };
                json!({"role": role, "content": content(m)})
            })
            .collect();

        let mut body = serde_json::Map::new();
        body.insert("model".into(), json!(request.model));
        body.insert("max_tokens".into(), json!(request.max_tokens));
        if !system.is_empty() {
            body.insert("system".into(), json!(system.join("\n\n")));
        }
        body.insert("messages".into(), Value::Array(messages));
        // Caches the prompt up to its last block; later requests that extend it read it back.
        body.insert("cache_control".into(), json!({"type": "ephemeral"}));
        reasoning_fields(&request.model, request.reasoning_effort, request.max_tokens, request.temperature, &mut body);

        debug!(model = %request.model, json_mode = request.json_mode, "Anthropic messages request");
        let (http_request, sent) = json_body(
            self.client
                .post(format!("{}/messages", self.base_url))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION),
            &body,
        )?;
        let response = http_request.send().await.map_err(transport_error)?;
        let (value, received) = sized_response_json(response, &request.model, false).await?;
        let counts = LlmUsage { bytes_sent: Some(sent), bytes_received: Some(received), ..usage(&value) };
        let text: String = value
            .get("content")
            .and_then(Value::as_array)
            .map(|blocks| {
                blocks
                    .iter()
                    .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|b| b.get("text").and_then(Value::as_str))
                    .collect()
            })
            .unwrap_or_default();
        if value.get("stop_reason").and_then(Value::as_str) == Some("max_tokens") {
            return Err(LlmError::OutputLimit { limit: request.max_tokens, usage: Box::new(counts) });
        }
        if text.trim().is_empty() {
            return Err(LlmError::EmptyResponse { usage: Box::new(counts) });
        }
        let content = if request.json_mode { without_fence(&text).to_string() } else { text };
        Ok(CompletionResponse {
            content,
            model: counts.model.clone().unwrap_or(request.model),
            prompt_tokens: counts.prompt_tokens.unwrap_or(0),
            completion_tokens: counts.completion_tokens.unwrap_or(0),
            total_tokens: counts.prompt_tokens.unwrap_or(0).saturating_add(counts.completion_tokens.unwrap_or(0)),
            usage: counts,
        })
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        let response = self.get("models?limit=1").send().await.map_err(transport_error)?;
        response_json(response, "").await.map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_generations_are_told_apart_by_id() {
        for model in ["claude-3-5-haiku-20241022", "claude-opus-4-20250514", "claude-opus-4-1", "claude-sonnet-4-5", "claude-haiku-4-5-20251001", "claude-opus-4"] {
            assert!(budget_thinking(model), "{model}");
        }
        for model in ["claude-sonnet-4-6", "claude-opus-4-7", "claude-opus-5-5", "claude-sonnet-5", "claude-haiku-5-5", "claude-fable-5-1"] {
            assert!(!budget_thinking(model), "{model}");
        }
    }

    #[test]
    fn json_fences_are_removed_only_around_the_whole_answer() {
        assert_eq!(without_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
        assert_eq!(without_fence("```\n[1]\n```"), "[1]");
        assert_eq!(without_fence("{\"a\":\"```\"}"), "{\"a\":\"```\"}");
    }
}
