use std::collections::HashMap;

use query2table_lib::providers::llm::{
    manager::{LlmBackend, LlmConfig, LlmManager},
    ollama::OllamaProvider,
    openai_compatible::OpenAiCompatibleProvider,
    openrouter::OpenRouterProvider,
    CompletionRequest, LlmError, LlmProvider, Message,
};
use query2table_lib::storage::db::Database;
use serde_json::{json, Value};
use wiremock::{
    matchers::{body_partial_json, header, method, path},
    Mock, MockServer, ResponseTemplate,
};

fn request() -> CompletionRequest {
    CompletionRequest {
        messages: vec![Message::system("Extract JSON"), Message::user("hello")],
        model: "test-model".into(),
        temperature: 0.25,
        max_tokens: 128,
        json_mode: true,
    }
}

fn completion() -> Value {
    json!({"choices": [{"message": {"content": "{\"ok\":true}"}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}})
}

#[tokio::test]
async fn openai_settings_route_to_custom_endpoint_without_auth_or_router_headers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/custom/v1/chat/completions"))
        .and(body_partial_json(
            json!({"model": "loaded-model", "temperature": 0.25,
            "max_tokens": 128, "response_format": {"type": "json_object"},
            "messages": [{"role": "user", "content": "hello"}]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .expect(1)
        .mount(&server)
        .await;
    let settings = HashMap::from([
        ("llm_provider".into(), "openai_compatible".into()),
        (
            "openai_base_url".into(),
            format!("{}/custom/v1/", server.uri()),
        ),
        ("openai_model".into(), "loaded-model".into()),
        ("openrouter_api_key".into(), "must-not-be-sent".into()),
        ("llm_temperature".into(), "0.25".into()),
        ("llm_max_tokens".into(), "128".into()),
    ]);
    let manager = LlmManager::from_config(LlmManager::config_from_settings(&settings)).unwrap();
    assert_eq!(manager.provider_name(), "openai_compatible");
    let response = manager
        .complete(vec![Message::user("hello")], true)
        .await
        .unwrap();
    assert_eq!(response.content, "{\"ok\":true}");
    assert_eq!(response.model, "loaded-model");
    assert_eq!(
        (
            response.prompt_tokens,
            response.completion_tokens,
            response.total_tokens
        ),
        (10, 5, 15)
    );
    let requests = server.received_requests().await.unwrap();
    for name in ["authorization", "http-referer", "x-title"] {
        assert!(!requests[0].headers.contains_key(name));
    }
}

#[tokio::test]
async fn openai_supports_bearer_auth_model_override_and_disabling_json_mode() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer local-key"))
        .and(body_partial_json(json!({"model": "override"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .expect(1)
        .mount(&server)
        .await;
    let settings = HashMap::from([
        ("llm_provider".into(), "openai_compatible".into()),
        ("openai_base_url".into(), format!("{}/", server.uri())),
        ("openai_model".into(), "default-model".into()),
        ("openai_api_key".into(), "local-key".into()),
        ("openai_json_mode".into(), "false".into()),
    ]);
    let manager = LlmManager::from_config(LlmManager::config_from_settings(&settings)).unwrap();
    manager
        .complete_with_model(vec![Message::user("Return JSON")], "override", true)
        .await
        .unwrap();
    let requests = server.received_requests().await.unwrap();
    assert!(requests[0]
        .body_json::<Value>()
        .unwrap()
        .get("response_format")
        .is_none());
}

#[tokio::test]
async fn ollama_settings_route_local_and_cloud_models_with_native_protocol() {
    for cloud in [false, true] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .and(body_partial_json(
                json!({"model": "configured-model", "stream": false,
                "options": {"temperature": 0.25, "num_predict": 128}}),
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "message": {"content": "{\"ok\":true}"}, "model": "returned-model",
                "prompt_eval_count": 11, "eval_count": 7
            })))
            .expect(1)
            .mount(&server)
            .await;
        let prefix = if cloud { "ollama_cloud" } else { "ollama" };
        let settings = HashMap::from([
            ("llm_provider".into(), prefix.into()),
            (format!("{prefix}_url"), format!("{}/api/", server.uri())),
            (format!("{prefix}_model"), "configured-model".into()),
            ("ollama_cloud_api_key".into(), "cloud-key".into()),
            ("openrouter_api_key".into(), "must-not-be-sent".into()),
            ("llm_temperature".into(), "0.25".into()),
            ("llm_max_tokens".into(), "128".into()),
        ]);
        let manager = LlmManager::from_config(LlmManager::config_from_settings(&settings)).unwrap();
        assert_eq!(manager.provider_name(), prefix);
        let response = manager
            .complete(vec![Message::user("hello")], true)
            .await
            .unwrap();
        assert_eq!(response.content, "{\"ok\":true}");
        assert_eq!(response.model, "returned-model");
        assert_eq!(response.total_tokens, 18);
        let requests = server.received_requests().await.unwrap();
        let body: Value = requests[0].body_json().unwrap();
        if cloud {
            assert_eq!(requests[0].headers["authorization"], "Bearer cloud-key");
            assert!(body.get("format").is_none());
            assert!(body["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("valid JSON"));
        } else {
            assert!(!requests[0].headers.contains_key("authorization"));
            assert_eq!(body["format"], "json");
        }
    }
}

#[tokio::test]
async fn openrouter_keeps_auth_headers_and_json_contract() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .and(header("authorization", "Bearer router-key"))
        .and(header("http-referer", "https://github.com/query2table"))
        .and(header("x-title", "Query2Table"))
        .and(body_partial_json(
            json!({"response_format": {"type": "json_object"}}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .expect(1)
        .mount(&server)
        .await;
    let provider = OpenRouterProvider::new("router-key".into())
        .unwrap()
        .with_base_url(server.uri());
    assert_eq!(provider.provider_name(), "openrouter");
    assert_eq!(
        provider
            .chat_completion(request())
            .await
            .unwrap()
            .total_tokens,
        15
    );
}

#[tokio::test]
async fn providers_classify_auth_rate_limit_missing_model_and_bad_responses() {
    for native in [false, true] {
        for status in [401, 403, 429, 404, 200] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .insert_header("retry-after", "2")
                        .set_body_json(if native {
                            json!({"error": "model not found"})
                        } else {
                            json!({"error": {"message": "model not found"}})
                        }),
                )
                .expect(1)
                .mount(&server)
                .await;
            let provider: Box<dyn LlmProvider> = if native {
                Box::new(OllamaProvider::cloud(server.uri(), "cloud-key".into()).unwrap())
            } else {
                Box::new(OpenAiCompatibleProvider::new(server.uri(), String::new(), true).unwrap())
            };
            let error = provider.chat_completion(request()).await.unwrap_err();
            match status {
                401 | 403 => assert!(matches!(error, LlmError::AuthError)),
                429 => assert!(matches!(
                    error,
                    LlmError::RateLimited {
                        retry_after_ms: 2000
                    }
                )),
                404 => assert!(matches!(error, LlmError::ModelNotFound(_))),
                _ => assert!(matches!(error, LlmError::ParseError(_))),
            }
        }
    }
}

#[tokio::test]
async fn health_checks_use_authenticated_model_endpoints_and_propagate_failures() {
    for native in [false, true] {
        for status in [200, 401, 503] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path(if native { "/api/tags" } else { "/v1/models" }))
                .and(header("authorization", "Bearer test-key"))
                .respond_with(ResponseTemplate::new(status).set_body_json(json!({})))
                .expect(1)
                .mount(&server)
                .await;
            let provider: Box<dyn LlmProvider> = if native {
                Box::new(OllamaProvider::cloud(server.uri(), "test-key".into()).unwrap())
            } else {
                Box::new(
                    OpenAiCompatibleProvider::new(server.uri(), "test-key".into(), true).unwrap(),
                )
            };
            assert_eq!(provider.health_check().await.is_ok(), status == 200);
        }
    }
}

#[test]
fn configuration_rejects_missing_cloud_key_model_and_invalid_url() {
    assert!(matches!(
        LlmManager::from_config(LlmConfig {
            backend: LlmBackend::OllamaCloud,
            ollama_cloud_api_key: "  ".into(),
            ..Default::default()
        }),
        Err(LlmError::NotConfigured(_))
    ));
    assert!(matches!(
        LlmManager::from_config(LlmConfig {
            backend: LlmBackend::OpenAiCompatible,
            ..Default::default()
        }),
        Err(LlmError::NotConfigured(_))
    ));
    for url in [
        "",
        "localhost:8080",
        "file:///tmp/model",
        "http://user:pass@localhost",
        "http://localhost/?key=secret",
    ] {
        assert!(matches!(
            OpenAiCompatibleProvider::new(url.into(), String::new(), true),
            Err(LlmError::NotConfigured(_))
        ));
    }
}

#[tokio::test]
async fn migration_adds_provider_defaults_and_preserves_saved_settings() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    let db = Database::with_pool(pool).await;
    db.migrate().await.unwrap();
    db.set_setting("llm_provider", "ollama_cloud")
        .await
        .unwrap();
    db.set_setting("ollama_cloud_model", "saved-model")
        .await
        .unwrap();
    db.set_setting("ollama_cloud_api_key", "saved-key")
        .await
        .unwrap();
    db.migrate().await.unwrap();
    let settings = db.get_all_settings().await.unwrap().into_iter().collect();
    let config = LlmManager::config_from_settings(&settings);
    assert_eq!(config.backend, LlmBackend::OllamaCloud);
    assert_eq!(config.ollama_cloud_url, "https://ollama.com");
    assert_eq!(config.ollama_cloud_model, "saved-model");
    assert_eq!(config.ollama_cloud_api_key, "saved-key");
    assert_eq!(config.openai_base_url, "http://localhost:8080/v1");
    assert!(config.openai_json_mode);
}
