use query2table_lib::commands::settings::list_ollama_cloud_models;
use std::collections::HashMap;

use query2table_lib::providers::llm::{
    manager::{LlmBackend, LlmConfig, LlmManager},
    ollama::OllamaProvider,
    openai_compatible::OpenAiCompatibleProvider,
    openrouter::OpenRouterProvider,
    CompletionRequest, LlmError, LlmProvider, Message, ReasoningEffort,
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
        reasoning_effort: ReasoningEffort::Auto,
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
                401 => assert!(matches!(error, LlmError::AuthError)),
                403 => assert!(matches!(error, LlmError::AccessDenied)),
                429 => assert!(matches!(
                    error,
                    LlmError::RateLimited {
                        retry_after_ms: 2000
                    }
                )),
                404 | 200 => assert!(matches!(error, LlmError::ModelNotFound(_))),
                _ => unreachable!(),
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

#[tokio::test]
async fn cloud_catalog_command_returns_sorted_unique_ids_with_optional_auth() {
    for key in ["", "test-cloud-key"] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"models": [
                {"name": "qwen:cloud", "size": 123}, {"name": "gpt-oss:120b"},
                {"name": "qwen:cloud"}, {"name": ""}
            ]})))
            .expect(1)
            .mount(&server)
            .await;
        let models = list_ollama_cloud_models(format!("{}/api/", server.uri()), key.into())
            .await
            .unwrap();
        assert_eq!(models, ["gpt-oss:120b", "qwen:cloud"]);
        let requests = server.received_requests().await.unwrap();
        if key.is_empty() {
            assert!(!requests[0].headers.contains_key("authorization"));
        } else {
            assert_eq!(
                requests[0].headers["authorization"],
                "Bearer test-cloud-key"
            );
        }
    }
}

#[tokio::test]
async fn cloud_catalog_reports_http_and_parse_errors_and_accepts_empty_lists() {
    for (status, body, succeeds) in [
        (401, json!({}), false),
        (429, json!({}), false),
        (503, json!({}), false),
        (200, json!({}), false),
        (200, json!({"models": [{"wrong": "field"}]}), false),
        (200, json!({"models": []}), true),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let result = list_ollama_cloud_models(server.uri(), String::new()).await;
        assert_eq!(
            result.is_ok(),
            succeeds,
            "Unexpected result for HTTP {status}: {result:?}"
        );
    }
}

#[tokio::test]
#[ignore = "Queries the live public Ollama Cloud model catalog"]
async fn cloud_catalog_live_endpoint() {
    let models = list_ollama_cloud_models("https://ollama.com".into(), String::new())
        .await
        .unwrap();
    assert!(!models.is_empty());
    println!("Ollama Cloud returned {} model IDs", models.len());
}

#[tokio::test]
async fn openrouter_catalog_uses_model_ids_and_optional_auth() {
    for key in ["", "router-key"] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [
                {"id": "openai/gpt-test", "name": "OpenAI: GPT Test"},
                {"id": "anthropic/claude-test", "name": "Anthropic: Claude Test"},
                {"id": "openai/gpt-test"}, {"id": ""}
            ]})))
            .expect(1)
            .mount(&server)
            .await;
        let provider =
            OpenAiCompatibleProvider::new(format!("{}/api/v1/", server.uri()), key.into(), true)
                .unwrap();
        assert_eq!(
            provider.list_models().await.unwrap(),
            ["anthropic/claude-test", "openai/gpt-test"]
        );
        let requests = server.received_requests().await.unwrap();
        if key.is_empty() {
            assert!(!requests[0].headers.contains_key("authorization"));
        } else {
            assert_eq!(requests[0].headers["authorization"], "Bearer router-key");
        }
    }
}

#[tokio::test]
async fn openrouter_catalog_propagates_errors_and_accepts_empty_lists() {
    for (status, body) in [
        (401, json!({})),
        (403, json!({})),
        (429, json!({})),
        (503, json!({})),
        (200, json!({})),
        (200, json!({"data": [{"name": "missing-id"}]})),
        (200, json!({"data": []})),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body.clone()))
            .expect(1)
            .mount(&server)
            .await;
        let provider = OpenAiCompatibleProvider::new(server.uri(), String::new(), true).unwrap();
        let result = provider.list_models().await;
        match status {
            401 => assert!(matches!(result, Err(LlmError::AuthError))),
            403 => assert!(matches!(result, Err(LlmError::AccessDenied))),
            429 => assert!(matches!(result, Err(LlmError::RateLimited { .. }))),
            503 => assert!(matches!(
                result,
                Err(LlmError::ProviderError {
                    retryable: true,
                    ..
                })
            )),
            _ if body == json!({"data": []}) => assert!(result.unwrap().is_empty()),
            _ => assert!(matches!(result, Err(LlmError::ParseError(_)))),
        }
    }
}

#[tokio::test]
async fn ollama_json_requests_preserve_answer_budget_without_disabling_plain_chat_thinking() {
    for (model, json_mode, expected_think) in [
        ("deepseek-v4.1-flash", true, Some(json!(false))),
        ("gpt-oss:120b", true, Some(json!("low"))),
        ("deepseek-v4.1-flash", false, None),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "message": {"content": "{\"queries\":[]}"}, "done_reason": "stop"
            })))
            .expect(1)
            .mount(&server)
            .await;
        let provider = OllamaProvider::cloud(server.uri(), "test-key".into()).unwrap();
        provider
            .chat_completion(CompletionRequest {
                model: model.into(),
                json_mode,
                ..request()
            })
            .await
            .unwrap();
        let requests = server.received_requests().await.unwrap();
        let body = requests[0].body_json::<Value>().unwrap();
        assert_eq!(body.get("think"), expected_think.as_ref());
    }
}

#[tokio::test]
async fn ollama_reports_truncated_and_empty_answers_before_role_json_parsing() {
    for (content, thinking, reason, expected) in [
        ("", "private reasoning", "length", "token limit"),
        ("{\"queries\":[", "", "length", "token limit"),
        (" ", "private reasoning", "stop", "empty answer"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "message": {"content": content, "thinking": thinking},
                "done_reason": reason, "eval_count": 4096
            })))
            .expect(1)
            .mount(&server)
            .await;
        let provider = OllamaProvider::cloud(server.uri(), "test-key".into()).unwrap();
        let error = provider.chat_completion(request()).await.unwrap_err();
        assert_eq!(
            error.code(),
            if reason == "length" {
                "output_limit"
            } else {
                "empty_response"
            }
        );
        assert!(error.to_string().contains(expected));
        assert!(!error.to_string().contains("private reasoning"));
    }
}

#[tokio::test]
async fn reasoning_choices_are_honored_by_each_provider_protocol() {
    use ReasoningEffort::*;
    for name in ["ollama", "openrouter", "openai"] {
        for effort in [Auto, ProviderDefault, Off, On, Low, Medium, High, Max] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(if name == "ollama" {
                        json!({"message": {"content": "{\"ok\":true}"}})
                    } else {
                        completion()
                    }),
                )
                .expect(1)
                .mount(&server)
                .await;
            let provider: Box<dyn LlmProvider> = match name {
                "ollama" => {
                    Box::new(OllamaProvider::cloud(server.uri(), "test-key".into()).unwrap())
                }
                "openrouter" => Box::new(
                    OpenRouterProvider::new("test-key".into())
                        .unwrap()
                        .with_base_url(server.uri()),
                ),
                _ => Box::new(
                    OpenAiCompatibleProvider::new(server.uri(), String::new(), true).unwrap(),
                ),
            };
            provider
                .chat_completion(CompletionRequest {
                    reasoning_effort: effort,
                    ..request()
                })
                .await
                .unwrap();
            let requests = server.received_requests().await.unwrap();
            let body = requests[0].body_json::<Value>().unwrap();
            let expected = match effort {
                Auto if name == "ollama" => Some(json!(false)),
                Auto | ProviderDefault => None,
                Off if name == "ollama" => Some(json!(false)),
                On if name == "ollama" => Some(json!(true)),
                Off if name == "openrouter" => Some(json!({"enabled": false})),
                On if name == "openrouter" => Some(json!({"enabled": true})),
                Off => Some(json!("none")),
                On => Some(json!("medium")),
                level if name == "openrouter" => Some(json!({"effort": level.level().unwrap()})),
                level => Some(json!(level.level().unwrap())),
            };
            let key = match name {
                "ollama" => "think",
                "openrouter" => "reasoning",
                _ => "reasoning_effort",
            };
            assert_eq!(body.get(key), expected.as_ref(), "{name} {effort:?}");
        }
    }
}

#[tokio::test]
async fn ollama_gpt_oss_validates_effort_without_silently_overriding_choices() {
    for effort in [ReasoningEffort::Off, ReasoningEffort::Max] {
        let server = MockServer::start().await;
        let provider = OllamaProvider::cloud(server.uri(), "test-key".into()).unwrap();
        let error = provider
            .chat_completion(CompletionRequest {
                model: "gpt-oss:120b".into(),
                reasoning_effort: effort,
                ..request()
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), "unsupported_setting");
        assert!(server.received_requests().await.unwrap().is_empty());
    }
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"think":"medium"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"message":{"content":"ok"}})))
        .expect(1)
        .mount(&server)
        .await;
    OllamaProvider::new(server.uri())
        .unwrap()
        .chat_completion(CompletionRequest {
            model: "gpt-oss:20b".into(),
            reasoning_effort: ReasoningEffort::On,
            json_mode: false,
            ..request()
        })
        .await
        .unwrap();
}

#[test]
fn reasoning_setting_round_trips_all_values_and_defaults_to_auto() {
    for value in [
        "auto", "default", "off", "on", "low", "medium", "high", "max",
    ] {
        let settings = HashMap::from([("llm_reasoning_effort".to_string(), value.to_string())]);
        let effort = LlmManager::config_from_settings(&settings).reasoning_effort;
        assert_eq!(serde_json::to_value(effort).unwrap(), json!(value));
    }
    assert_eq!(LlmConfig::default().reasoning_effort, ReasoningEffort::Auto);
}

#[tokio::test]
async fn http_and_error_envelopes_distinguish_permanent_limits_from_transient_failures() {
    for native in [false, true] {
        for (status, envelope, code, retryable) in [
            (402, json!({"message":"Not enough credits"}), "quota", false),
            (429, json!({"code":"insufficient_quota"}), "quota", false),
            (
                429,
                json!({"code":"rate_limit_exceeded"}),
                "rate_limit",
                true,
            ),
            (
                400,
                json!({"code":"context_length_exceeded"}),
                "context_limit",
                false,
            ),
            (
                400,
                json!({"message":"unsupported reasoning_effort value"}),
                "unsupported_setting",
                false,
            ),
            (
                403,
                json!({"code":"invalid_api_key", "message":"Regional access denied"}),
                "access_denied",
                false,
            ),
            (401, json!({"message":"Bad credentials"}), "auth", false),
            (504, json!({"message":"Gateway timeout"}), "timeout", true),
            (
                503,
                json!({"message":"Service unavailable"}),
                "provider_error",
                true,
            ),
            (
                400,
                json!({"message":"Invalid request"}),
                "provider_error",
                false,
            ),
            (
                200,
                json!({"code":402,"message":"Credits exhausted"}),
                "quota",
                false,
            ),
            (
                200,
                json!({"message":"Request failed"}),
                "provider_error",
                false,
            ),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .respond_with(
                    ResponseTemplate::new(status).set_body_json(json!({"error":envelope})),
                )
                .expect(1)
                .mount(&server)
                .await;
            let provider: Box<dyn LlmProvider> = if native {
                Box::new(OllamaProvider::new(server.uri()).unwrap())
            } else {
                Box::new(OpenAiCompatibleProvider::new(server.uri(), String::new(), true).unwrap())
            };
            let error = provider.chat_completion(request()).await.unwrap_err();
            assert_eq!(
                (error.code(), error.retryable()),
                (code, retryable),
                "native={native} status={status}"
            );
        }
    }
}

#[tokio::test]
async fn openai_length_and_empty_responses_include_token_usage_without_reasoning_text() {
    for (reason, content, expected_code) in [
        ("length", "", "output_limit"),
        ("length", "{\"partial\":", "output_limit"),
        ("stop", "   ", "empty_response"),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices":[{"message":{"content":content,"reasoning":"PRIVATE_THINKING"},"finish_reason":reason}],
                "usage":{"prompt_tokens":23,"completion_tokens":128,"completion_tokens_details":{"reasoning_tokens":120}}
            }))).expect(1).mount(&server).await;
        let error = OpenAiCompatibleProvider::new(server.uri(), String::new(), true)
            .unwrap()
            .chat_completion(request())
            .await
            .unwrap_err();
        assert_eq!(error.code(), expected_code);
        let usage = error.usage();
        assert_eq!(
            (
                usage.prompt_tokens,
                usage.completion_tokens,
                usage.reasoning_tokens
            ),
            (Some(23), Some(128), Some(120))
        );
        assert!(!error.to_string().contains("PRIVATE_THINKING"));
    }
}

#[tokio::test]
async fn raw_provider_error_body_is_never_exposed() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            json!({"error":{"message":"PRIVATE_KEY PRIVATE_PROMPT PRIVATE_THINKING"}}),
        ))
        .expect(1)
        .mount(&server)
        .await;
    let error = OpenAiCompatibleProvider::new(server.uri(), String::new(), true)
        .unwrap()
        .chat_completion(request())
        .await
        .unwrap_err();
    assert!(!error.to_string().contains("PRIVATE_"));
}

struct FailingProvider {
    error: LlmError,
    calls: std::sync::atomic::AtomicU32,
}

#[async_trait::async_trait]
impl LlmProvider for FailingProvider {
    async fn chat_completion(
        &self,
        _: CompletionRequest,
    ) -> Result<query2table_lib::providers::llm::CompletionResponse, LlmError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Err(self.error.clone())
    }
    fn provider_name(&self) -> &str {
        "mock"
    }
    async fn health_check(&self) -> Result<(), LlmError> {
        Ok(())
    }
}

#[tokio::test]
async fn manager_emits_each_retry_with_stage_and_actual_attempt_numbers() {
    let provider = std::sync::Arc::new(FailingProvider {
        error: LlmError::RateLimited { retry_after_ms: 0 },
        calls: Default::default(),
    });
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let manager =
        LlmManager::with_provider(provider.clone(), LlmConfig::default()).with_diagnostics(tx);
    let error = manager
        .complete_for_stage("query_expander", vec![Message::user("query")], true)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "rate_limit");
    for attempt in 1..=4 {
        let issue = rx.try_recv().unwrap();
        assert_eq!(
            (issue.attempt, issue.max_attempts, issue.will_retry),
            (attempt, 4, attempt < 4)
        );
        assert_eq!(issue.stage.as_deref(), Some("query_expander"));
        assert_eq!(issue.retry_after_ms, Some(0));
    }
    assert!(rx.try_recv().is_err());
    assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 4);
}

#[tokio::test]
async fn manager_does_not_retry_permanent_errors_and_preserves_usage() {
    use query2table_lib::providers::llm::LlmUsage;
    for error in [
        LlmError::QuotaExceeded,
        LlmError::AuthError,
        LlmError::AccessDenied,
        LlmError::ContextLimit,
        LlmError::UnsupportedSetting("unsupported".into()),
        LlmError::ParseError("invalid JSON".into()),
        LlmError::EmptyResponse {
            usage: LlmUsage::default(),
        },
        LlmError::OutputLimit {
            limit: 128,
            usage: LlmUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(128),
                reasoning_tokens: Some(120),
            },
        },
        LlmError::ProviderError {
            message: "bad request".into(),
            retryable: false,
        },
    ] {
        let expected = error.code();
        let provider = std::sync::Arc::new(FailingProvider {
            error,
            calls: Default::default(),
        });
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let manager = LlmManager::with_provider(
            provider.clone(),
            LlmConfig {
                max_tokens: 128,
                ..Default::default()
            },
        )
        .with_diagnostics(tx);
        manager
            .complete_for_stage("extractor", vec![], true)
            .await
            .unwrap_err();
        let issue = rx.try_recv().unwrap();
        assert_eq!(issue.code, expected);
        assert_eq!(
            (issue.attempt, issue.will_retry, issue.max_tokens),
            (1, false, 128)
        );
        if expected == "output_limit" {
            assert_eq!(issue.reasoning_tokens, Some(120));
        }
        assert!(rx.try_recv().is_err());
        assert_eq!(provider.calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}

#[tokio::test]
async fn setup_diagnostics_report_unsupported_reasoning_and_missing_cloud_keys() {
    for (effort, key, code) in [
        (ReasoningEffort::Off, "test-key", "unsupported_setting"),
        (ReasoningEffort::Auto, "", "not_configured"),
    ] {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let result = LlmManager::from_config_with_diagnostics(
            LlmConfig {
                backend: LlmBackend::OllamaCloud,
                ollama_cloud_model: "gpt-oss:120b".into(),
                ollama_cloud_api_key: key.into(),
                reasoning_effort: effort,
                ..Default::default()
            },
            tx,
        );
        assert!(result.is_err());
        let issue = rx.try_recv().unwrap();
        assert_eq!(
            (issue.code.as_str(), issue.stage.as_deref()),
            (code, Some("setup"))
        );
        assert_eq!(
            (issue.attempt, issue.max_attempts, issue.will_retry),
            (1, 1, false)
        );
    }
}

#[tokio::test]
async fn diagnostics_redact_configured_keys_and_bound_text() {
    let secret = "VERY_PRIVATE_TEST_KEY";
    let provider = std::sync::Arc::new(FailingProvider {
        error: LlmError::ParseError(format!("{secret} {}", "я".repeat(2000))),
        calls: Default::default(),
    });
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let manager = LlmManager::with_provider(
        provider,
        LlmConfig {
            openrouter_api_key: secret.into(),
            ..Default::default()
        },
    )
    .with_diagnostics(tx);
    manager.complete(vec![], true).await.unwrap_err();
    let issue = rx.try_recv().unwrap();
    assert!(!issue.message.contains(secret));
    assert!(issue.message.chars().count() <= 1200);
    assert!(issue.message.contains("[REDACTED]"));
}

#[tokio::test]
async fn parsing_diagnostic_keeps_response_usage_and_does_not_include_content() {
    use query2table_lib::providers::llm::CompletionResponse;
    let provider = std::sync::Arc::new(FailingProvider {
        error: LlmError::ParseError("unused".into()),
        calls: Default::default(),
    });
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let manager = LlmManager::with_provider(provider, LlmConfig::default()).with_diagnostics(tx);
    manager.report_invalid_response(
        "extractor",
        "The answer does not contain the required rows array.",
        &CompletionResponse {
            model: "test-model".into(),
            content: "PRIVATE_MODEL_RESPONSE".into(),
            prompt_tokens: 123,
            completion_tokens: 34,
            total_tokens: 157,
        },
    );
    let issue = rx.try_recv().unwrap();
    assert_eq!(
        (issue.code.as_str(), issue.stage.as_deref()),
        ("invalid_response", Some("extractor"))
    );
    assert_eq!(
        (issue.prompt_tokens, issue.completion_tokens),
        (Some(123), Some(34))
    );
    assert_eq!(
        (issue.attempt, issue.max_attempts, issue.will_retry),
        (1, 1, false)
    );
    assert!(!serde_json::to_string(&issue)
        .unwrap()
        .contains("PRIVATE_MODEL_RESPONSE"));
}

#[tokio::test]
async fn native_length_without_message_still_reports_output_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"done_reason":"length","eval_count":128})),
        )
        .expect(1)
        .mount(&server)
        .await;
    let error = OllamaProvider::new(server.uri())
        .unwrap()
        .chat_completion(request())
        .await
        .unwrap_err();
    assert_eq!(error.code(), "output_limit");
    assert_eq!(error.usage().completion_tokens, Some(128));
}
