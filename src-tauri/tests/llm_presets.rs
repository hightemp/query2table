//! Provider presets: services speaking the OpenAI Chat Completions API (DeepSeek, Groq, Gemini,
//! LM Studio…) run through the one OpenAI-compatible client with their own address, key and model.

use std::collections::HashMap;

use query2table_lib::commands::connection::{list_openai_models, test_llm_connection};
use query2table_lib::orchestrator::budget_tracker::BudgetTracker;
use query2table_lib::providers::accounting::validate_pricing_setting;
use query2table_lib::providers::llm::{
    capabilities::vision_support,
    manager::{LlmBackend, LlmConfig, LlmManager},
    presets, LlmError, LlmUsage, Message, ReasoningEffort,
};
use serde_json::{json, Value};
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn settings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

fn completion() -> Value {
    json!({"choices": [{"message": {"content": "{\"ok\":true}"}}],
        "usage": {"prompt_tokens": 1_000_000, "completion_tokens": 1_000_000}})
}

/// A preset's configuration pointed at the mock server instead of the real service.
fn preset_config(id: &str, server: &MockServer, model: &str) -> LlmConfig {
    let mut config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", id),
        (&format!("{id}_api_key"), "preset-key"),
        (&format!("{id}_model"), model),
        ("llm_temperature", "0.25"),
        ("llm_max_tokens", "128"),
    ]));
    config.openai_base_url = format!("{}/v1", server.uri());
    config
}

async fn mount_completion(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .mount(server)
        .await;
}

async fn sent_body(server: &MockServer) -> Value {
    let requests = server.received_requests().await.unwrap();
    serde_json::from_slice(&requests.last().expect("a request was sent").body).unwrap()
}

#[test]
fn preset_settings_resolve_to_their_address_key_and_model() {
    let config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "deepseek"),
        ("deepseek_api_key", "ds-key"),
        ("deepseek_model", "deepseek-chat"),
        // "Custom address" keeps its own values.
        ("openai_base_url", "http://custom:9000/v1"),
        ("openai_api_key", "custom-key"),
        ("openai_model", "custom-model"),
    ]));
    assert_eq!(config.backend, LlmBackend::OpenAiCompatible);
    assert_eq!(config.preset.map(|p| p.id), Some("deepseek"));
    assert_eq!(
        (config.openai_base_url.as_str(), config.openai_api_key.as_str(), config.openai_model.as_str(), config.openai_json_mode),
        ("https://api.deepseek.com/v1", "ds-key", "deepseek-chat", true)
    );

    // Local servers: editable address, JSON mode from the preset unless set.
    let lmstudio = LlmManager::config_from_settings(&settings(&[("llm_provider", "lmstudio"), ("lmstudio_model", "qwen3")]));
    assert_eq!((lmstudio.openai_base_url.as_str(), lmstudio.openai_json_mode), ("http://localhost:1234/v1", false));
    let moved = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "lmstudio"),
        ("lmstudio_url", "http://studio.lan:1234/v1"),
        ("lmstudio_json_mode", "true"),
    ]));
    assert_eq!((moved.openai_base_url.as_str(), moved.openai_json_mode), ("http://studio.lan:1234/v1", true));

    // A cloud preset's address cannot be moved by a stray setting.
    let fixed = LlmManager::config_from_settings(&settings(&[("llm_provider", "groq"), ("groq_url", "http://elsewhere/v1")]));
    assert_eq!(fixed.openai_base_url, "https://api.groq.com/openai/v1");

    let custom = LlmManager::config_from_settings(&settings(&[("llm_provider", "openai_compatible"), ("openai_model", "m")]));
    assert_eq!((custom.preset, custom.openai_model.as_str()), (None, "m"));
}

#[test]
fn cloud_presets_need_a_key_and_local_ones_do_not() {
    let error = LlmManager::from_config(LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "deepseek"),
        ("deepseek_model", "deepseek-chat"),
    ])))
    .err()
    .expect("a cloud preset without a key is not configured");
    assert!(matches!(&error, LlmError::NotConfigured(message) if message.contains("DeepSeek API key")), "{error}");
    assert!(LlmManager::from_config(LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "llamacpp"),
        ("llamacpp_model", "local"),
    ])))
    .is_ok());
}

#[tokio::test]
async fn preset_requests_carry_its_key_and_name_and_skip_unsupported_options() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer preset-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .expect(1)
        .mount(&server)
        .await;
    let mut config = preset_config("deepseek", &server, "deepseek-chat");
    config.reasoning_effort = ReasoningEffort::High;
    let manager = LlmManager::from_config(config).unwrap();
    assert_eq!(manager.provider_name(), "deepseek");
    manager.complete(vec![Message::user("hello")], true).await.unwrap();
    let body = sent_body(&server).await;
    assert_eq!(body["model"], "deepseek-chat");
    assert_eq!(body["max_tokens"], 128);
    assert_eq!(body["response_format"], json!({"type": "json_object"}));
    // DeepSeek does not take `reasoning_effort`; the setting must not break its requests.
    assert!(body.get("reasoning_effort").is_none(), "{body}");
}

#[tokio::test]
async fn openai_reasoning_models_get_completion_tokens_and_their_own_temperature() {
    let server = MockServer::start().await;
    mount_completion(&server).await;
    let manager = LlmManager::from_config(preset_config("openai_cloud", &server, "gpt-5-mini")).unwrap();
    manager.complete(vec![Message::user("hello")], false).await.unwrap();
    let body = sent_body(&server).await;
    assert_eq!(body["max_completion_tokens"], 128);
    assert!(body.get("max_tokens").is_none() && body.get("temperature").is_none(), "{body}");

    let manager = LlmManager::from_config(preset_config("openai_cloud", &server, "gpt-4.1-mini")).unwrap();
    manager.complete(vec![Message::user("hello")], false).await.unwrap();
    let body = sent_body(&server).await;
    assert_eq!((body["max_completion_tokens"].clone(), body["temperature"].clone()), (json!(128), json!(0.25)));
}

#[tokio::test]
async fn local_presets_cost_nothing_and_cloud_presets_need_a_rate() {
    let server = MockServer::start().await;
    mount_completion(&server).await;

    let budget = BudgetTracker::new(100.0);
    let local = LlmManager::from_config(preset_config("lmstudio", &server, "qwen3")).unwrap().with_accounting(budget.observer());
    local.complete(vec![Message::user("hello")], true).await.unwrap();
    let stats = budget.snapshot();
    assert_eq!((stats.unpriced_calls, stats.estimated_usd), (0, 0.0));

    let budget = BudgetTracker::new(100.0);
    let cloud = LlmManager::from_config(preset_config("deepseek", &server, "deepseek-chat")).unwrap().with_accounting(budget.observer());
    cloud.complete(vec![Message::user("hello")], true).await.unwrap();
    assert_eq!(budget.snapshot().unpriced_calls, 1, "no rate: the cost is unknown");

    // A manual rate is scoped to the preset, its address and the model.
    let mut config = preset_config("deepseek", &server, "deepseek-chat");
    let scope = serde_json::to_string(&("deepseek", config.openai_base_url.as_str(), "deepseek-chat")).unwrap();
    config.pricing_overrides = json!({(scope): {"input_per_million": 0.5, "output_per_million": 2.0}}).to_string();
    let budget = BudgetTracker::new(100.0);
    let priced = LlmManager::from_config(config).unwrap().with_accounting(budget.observer());
    priced.complete(vec![Message::user("hello")], true).await.unwrap();
    let stats = budget.snapshot();
    assert_eq!((stats.unpriced_calls, stats.estimated_usd), (0, 2.5));
}

#[test]
fn manual_rates_accept_presets_and_reject_unknown_providers() {
    let rate = |provider: &str| {
        let scope = serde_json::to_string(&(provider, "https://api.groq.com/openai/v1", "llama")).unwrap();
        validate_pricing_setting("llm_pricing_overrides", &json!({(scope): {"input_per_million": 1, "output_per_million": 1}}).to_string())
    };
    assert!(rate("groq").is_ok());
    assert!(rate("nonexistent").is_err());
}

#[tokio::test]
async fn presets_are_not_asked_whether_models_see_images() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"data": [{"id": "grok-4", "architecture": {"input_modalities": ["text", "image"]}}]}),
        ))
        .expect(0)
        .mount(&server)
        .await;
    let config = preset_config("xai", &server, "grok-4");
    assert_eq!(vision_support(&config, "grok-4").await, None);
}

#[tokio::test]
async fn gemini_catalog_lists_model_ids_without_their_prefix() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", "Bearer g-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"data": [{"id": "models/gemini-2.5-flash"}, {"id": "models/gemini-2.5-pro"}]}),
        ))
        .mount(&server)
        .await;
    let url = format!("{}/v1", server.uri());
    assert_eq!(
        list_openai_models(url.clone(), "g-key".into(), Some("gemini".into())).await.unwrap(),
        ["gemini-2.5-flash", "gemini-2.5-pro"]
    );
    // Other servers keep the ids they list.
    assert_eq!(list_openai_models(url, "g-key".into(), None).await.unwrap(), ["models/gemini-2.5-flash", "models/gemini-2.5-pro"]);
}

#[tokio::test]
async fn connection_check_names_the_preset() {
    let missing = test_llm_connection(settings(&[("llm_provider", "mistral"), ("mistral_model", "mistral-small")])).await.unwrap_err();
    assert!(missing.contains("Mistral API key"), "{missing}");

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "qwen3"}]})))
        .mount(&server)
        .await;
    let report = test_llm_connection(settings(&[
        ("llm_provider", "lmstudio"),
        ("lmstudio_url", &format!("{}/v1", server.uri())),
        ("lmstudio_model", "qwen3"),
    ]))
    .await
    .unwrap();
    assert!(report.ok && report.message.contains("LM Studio"), "{}", report.message);
}

#[test]
fn deepseek_cache_hits_count_as_cached_input() {
    let usage = LlmUsage::from_json(
        &json!({"usage": {"prompt_tokens": 100, "completion_tokens": 5, "prompt_cache_hit_tokens": 60}}),
        false,
    );
    assert_eq!(usage.cached_prompt_tokens, Some(60));
}

#[test]
fn every_preset_has_a_fixed_or_editable_address() {
    for preset in presets::PRESETS {
        let config = LlmManager::config_from_settings(&settings(&[("llm_provider", preset.id)]));
        assert_eq!(config.openai_base_url, preset.url, "{}", preset.id);
    }
}
