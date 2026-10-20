//! The native Anthropic client (Messages API): request shape, prompt caching, thinking settings
//! by model generation, usage with cache reads and writes, errors, images and PDF documents.

use std::collections::HashMap;

use query2table_lib::commands::connection::test_llm_connection;
use query2table_lib::orchestrator::budget_tracker::BudgetTracker;
use query2table_lib::providers::accounting::validate_pricing_setting;
use query2table_lib::providers::llm::{
    anthropic::AnthropicProvider,
    capabilities::vision_support,
    manager::{LlmBackend, LlmConfig, LlmManager},
    CompletionRequest, DocumentInput, ImageInput, LlmError, LlmProvider, Message, ReasoningEffort,
};
use serde_json::{json, Value};
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

fn settings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn answer(text: &str) -> Value {
    json!({
        "id": "msg_1", "type": "message", "role": "assistant", "model": "claude-sonnet-5-5",
        "content": [{"type": "thinking", "thinking": "Let me see.", "signature": "sig"}, {"type": "text", "text": text}],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 100, "output_tokens": 20, "cache_read_input_tokens": 900, "cache_creation_input_tokens": 50}
    })
}

async fn server_answering(body: Value) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;
    server
}

fn provider(server: &MockServer) -> AnthropicProvider {
    AnthropicProvider::new(format!("{}/v1", server.uri()), "ant-key".into()).unwrap()
}

fn request(model: &str, effort: ReasoningEffort, json_mode: bool) -> CompletionRequest {
    CompletionRequest {
        messages: vec![Message::system("You extract data."), Message::user("hello")],
        model: model.into(),
        temperature: 0.25,
        max_tokens: 4096,
        json_mode,
        reasoning_effort: effort,
    }
}

async fn sent(server: &MockServer) -> Value {
    let requests = server.received_requests().await.unwrap();
    serde_json::from_slice(&requests.last().expect("a request was sent").body).unwrap()
}

#[tokio::test]
async fn messages_carry_the_key_version_system_prompt_and_cache_control() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "ant-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(answer("```json\n{\"ok\":true}\n```")))
        .expect(1)
        .mount(&server)
        .await;
    let response = provider(&server).chat_completion(request("claude-sonnet-5-5", ReasoningEffort::Auto, true)).await.unwrap();
    // Thinking blocks are not part of the answer; JSON fences are removed in JSON mode.
    assert_eq!(response.content, "{\"ok\":true}");

    let body = sent(&server).await;
    assert_eq!(body["model"], "claude-sonnet-5-5");
    assert_eq!(body["max_tokens"], 4096);
    assert_eq!(body["cache_control"], json!({"type": "ephemeral"}));
    let system = body["system"].as_str().unwrap();
    assert!(system.starts_with("You extract data.") && system.contains("only valid JSON"), "{system}");
    assert_eq!(body["messages"], json!([{"role": "user", "content": "hello"}]));
    // Newer models reject sampling settings; thinking is left to the model with Auto.
    for field in ["temperature", "thinking", "output_config"] {
        assert!(body.get(field).is_none(), "{field}: {body}");
    }
}

#[tokio::test]
async fn usage_counts_cache_reads_and_writes_as_input() {
    let server = server_answering(answer("done")).await;
    let response = provider(&server).chat_completion(request("claude-sonnet-5-5", ReasoningEffort::Auto, false)).await.unwrap();
    let usage = &response.usage;
    assert_eq!(
        (usage.prompt_tokens, usage.cached_prompt_tokens, usage.cache_write_tokens, usage.completion_tokens),
        (Some(1050), Some(900), Some(50), Some(20))
    );
    assert_eq!(usage.model.as_deref(), Some("claude-sonnet-5-5"));
    assert_eq!(response.content, "done", "text is kept as written outside JSON mode");
}

#[tokio::test]
async fn reasoning_settings_follow_the_model_generation() {
    let server = server_answering(answer("ok")).await;
    let client = provider(&server);
    let body_for = |model: &'static str, effort: ReasoningEffort| {
        let client = &client;
        let server = &server;
        async move {
            client.chat_completion(request(model, effort, false)).await.unwrap();
            sent(server).await
        }
    };

    // Adaptive thinking with effort on current models; no way to switch thinking off on all of
    // them, so Off asks for the least effort.
    let body = body_for("claude-opus-5-5", ReasoningEffort::High).await;
    assert_eq!((body["thinking"].clone(), body["output_config"].clone()), (json!({"type": "adaptive"}), json!({"effort": "high"})));
    let body = body_for("claude-sonnet-5-5", ReasoningEffort::Off).await;
    assert_eq!((body.get("thinking"), body["output_config"].clone()), (None, json!({"effort": "low"})));
    let body = body_for("claude-sonnet-4-6", ReasoningEffort::On).await;
    assert_eq!((body["thinking"].clone(), body.get("output_config")), (json!({"type": "adaptive"}), None));

    // Earlier models: a thinking budget below max_tokens, and temperature only without thinking.
    let body = body_for("claude-haiku-4-5", ReasoningEffort::High).await;
    assert_eq!(body["thinking"], json!({"type": "enabled", "budget_tokens": 3072}));
    assert!(body.get("temperature").is_none() && body.get("output_config").is_none(), "{body}");
    let body = body_for("claude-3-5-haiku-20241022", ReasoningEffort::Auto).await;
    assert_eq!(body["temperature"], 0.25);
    assert!(body.get("thinking").is_none());
}

#[tokio::test]
async fn pictures_and_pdf_documents_come_before_the_text() {
    let server = server_answering(answer("ok")).await;
    let mut message = Message::user_with_images("What is in these?", vec![ImageInput::from_bytes("image/png", b"png")]);
    message.documents = vec![DocumentInput::pdf(b"%PDF-1.7")];
    let mut req = request("claude-sonnet-5-5", ReasoningEffort::Auto, false);
    req.messages = vec![message];
    provider(&server).chat_completion(req).await.unwrap();
    let body = sent(&server).await;
    assert_eq!(
        body["messages"][0]["content"],
        json!([
            {"type": "document", "source": {"type": "base64", "media_type": "application/pdf", "data": "JVBERi0xLjc="}},
            {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "cG5n"}},
            {"type": "text", "text": "What is in these?"}
        ])
    );
}

#[tokio::test]
async fn errors_and_cut_off_answers_are_classified() {
    let error_for = |status: u16, body: Value| async move {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status).insert_header("retry-after", "7").set_body_json(body))
            .mount(&server)
            .await;
        provider(&server).chat_completion(request("claude-sonnet-5-5", ReasoningEffort::Auto, false)).await.unwrap_err()
    };
    let envelope = |kind: &str| json!({"type": "error", "error": {"type": kind, "message": "details"}});
    assert!(matches!(error_for(401, envelope("authentication_error")).await, LlmError::AuthError));
    assert!(matches!(error_for(429, envelope("rate_limit_error")).await, LlmError::RateLimited { retry_after_ms: 7000 }));
    let overloaded = error_for(529, envelope("overloaded_error")).await;
    assert!(overloaded.retryable(), "{overloaded}");

    let mut cut = answer("partial");
    cut["stop_reason"] = json!("max_tokens");
    let server = server_answering(cut).await;
    let error = provider(&server).chat_completion(request("claude-sonnet-5-5", ReasoningEffort::Auto, false)).await.unwrap_err();
    assert!(matches!(error, LlmError::OutputLimit { limit: 4096, .. }), "{error}");
}

#[tokio::test]
async fn the_catalog_lists_model_ids() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(query_param("limit", "1000"))
        .and(header("x-api-key", "ant-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [{"id": "claude-sonnet-5-5", "type": "model"}, {"id": "claude-haiku-5-5", "type": "model"}],
            "has_more": false
        })))
        .mount(&server)
        .await;
    assert_eq!(provider(&server).list_models().await.unwrap(), ["claude-haiku-5-5", "claude-sonnet-5-5"]);
}

#[tokio::test]
async fn settings_select_anthropic_with_its_key_and_all_models_see_images() {
    let config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "anthropic"),
        ("anthropic_api_key", "ant-key"),
        ("anthropic_model", "claude-sonnet-5-5"),
        ("anthropic_native_pdf", "true"),
    ]));
    assert_eq!(config.backend, LlmBackend::Anthropic);
    assert_eq!((config.anthropic_api_key.as_str(), config.anthropic_model.as_str(), config.anthropic_native_pdf), ("ant-key", "claude-sonnet-5-5", true));
    assert_eq!(vision_support(&config, "claude-sonnet-5-5").await, Some(true));
    // The PDF switch belongs to Anthropic only.
    let other = LlmManager::config_from_settings(&settings(&[("llm_provider", "deepseek"), ("anthropic_native_pdf", "true")]));
    assert!(!other.native_pdf());
    assert!(config.native_pdf());

    let missing = LlmManager::from_config(LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "anthropic"),
        ("anthropic_model", "claude-sonnet-5-5"),
    ])))
    .err()
    .unwrap();
    assert!(matches!(&missing, LlmError::NotConfigured(message) if message.contains("Anthropic API key")), "{missing}");
    let check = test_llm_connection(settings(&[("llm_provider", "anthropic"), ("anthropic_model", "m")])).await.unwrap_err();
    assert!(check.contains("Anthropic API key"), "{check}");
}

#[tokio::test]
async fn manual_rates_price_cache_reads_and_writes() {
    let server = server_answering(answer("ok")).await;
    let scope = serde_json::to_string(&("anthropic", "https://api.anthropic.com/v1", "claude-sonnet-5-5")).unwrap();
    let overrides = json!({(scope): {
        "input_per_million": 3.0, "output_per_million": 15.0,
        "cached_input_per_million": 0.3, "cache_write_per_million": 3.75
    }})
    .to_string();
    assert!(validate_pricing_setting("llm_pricing_overrides", &overrides).is_ok());
    let config = LlmConfig {
        backend: LlmBackend::Anthropic,
        anthropic_api_key: "ant-key".into(),
        anthropic_model: "claude-sonnet-5-5".into(),
        pricing_overrides: overrides,
        ..Default::default()
    };
    let budget = BudgetTracker::new(100.0);
    let manager = LlmManager::with_provider(std::sync::Arc::new(provider(&server)), config).with_accounting(budget.observer());
    assert_eq!(manager.provider_name(), "anthropic");
    manager.complete(vec![Message::user("hello")], false).await.unwrap();
    let stats = budget.snapshot();
    // 100 × 3 + 900 × 0.3 + 50 × 3.75 + 20 × 15, per million tokens.
    let expected = (100.0 * 3.0 + 900.0 * 0.3 + 50.0 * 3.75 + 20.0 * 15.0) / 1_000_000.0;
    assert_eq!(stats.unpriced_calls, 0);
    assert!((stats.estimated_usd - expected).abs() < 1e-12, "{} vs {expected}", stats.estimated_usd);
}
