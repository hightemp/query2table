use async_trait::async_trait;
use query2table_lib::{
    orchestrator::budget_tracker::BudgetTracker,
    providers::{
        llm::{
            manager::LlmConfig, openrouter::OpenRouterProvider, CompletionRequest,
            CompletionResponse, LlmError, LlmManager, LlmProvider, LlmUsage, Message,
        },
        search::{
            manager::{SearchBackend, SearchConfig},
            SearchError, SearchManager, SearchProvider, SearchQuery, SearchResult,
        },
    },
    roles::image_ranker::ImageRanker,
};
use serde_json::json;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

fn manager(server: &MockServer, budget: &BudgetTracker) -> LlmManager {
    let provider = OpenRouterProvider::new("test-key".into())
        .unwrap()
        .with_base_url(format!("{}/v1", server.uri()));
    LlmManager::with_provider(
        Arc::new(provider),
        LlmConfig {
            openrouter_model: "requested/model".into(),
            ..Default::default()
        },
    )
    .with_accounting(budget.observer())
}
fn usage(cost: f64) -> serde_json::Value {
    json!({"prompt_tokens":17,"completion_tokens":9,"completion_tokens_details":{"reasoning_tokens":4},"cost":cost})
}

#[tokio::test]
async fn reported_charges_survive_truncation_empty_malformed_and_error_responses() {
    for body in [
        json!({"choices":[{"message":{"content":"{}"}}],"usage":usage(0.01)}),
        json!({"choices":[{"finish_reason":"length","message":{"content":""}}],"usage":usage(0.01)}),
        json!({"choices":[],"usage":usage(0.01)}),
        json!({"error":{"code":400,"message":"rejected"},"usage":usage(0.01)}),
        json!({"choices":[{"message":{"content":""}}],"usage":usage(0.01)}),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .expect(1)
            .mount(&server)
            .await;
        let budget = BudgetTracker::new(1.0);
        let manager = manager(&server, &budget);
        if let Ok(response) = manager.complete(vec![Message::user("test")], true).await {
            manager.report_invalid_response("test", "Invalid role shape", &response, query2table_lib::providers::llm::IssueOutcome::Skipped);
        }
        let stats = budget.snapshot();
        assert_eq!(stats.llm_calls, 1);
        assert_eq!(stats.reported_calls, 1);
        assert_eq!(stats.spent_usd, 0.01);
        assert_eq!(stats.prompt_tokens, 17);
        assert_eq!(stats.completion_tokens, 9);
        assert_eq!(stats.reasoning_tokens, 4);
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}

#[tokio::test]
async fn actual_model_catalog_rates_are_cached_without_reusing_requested_model_tariffs() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"model":"returned/model","choices":[{"message":{"content":"{}"}}],"usage":{"prompt_tokens":30,"completion_tokens":2,"prompt_tokens_details":{"cached_tokens":10},"completion_tokens_details":{"reasoning_tokens":1}}}))).expect(2).mount(&server).await;
    Mock::given(method("GET")).and(path("/v1/models")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":[{"id":"returned/model","pricing":{"prompt":"0.000002","completion":"0.000008","input_cache_read":"0.0000005","request":"0.01"}},{"id":"requested/model","pricing":{"prompt":"1","completion":"1"}}]}))).expect(1).mount(&server).await;
    let budget = BudgetTracker::new(1.0);
    let manager = manager(&server, &budget);
    for _ in 0..2 {
        manager
            .complete(vec![Message::user("test")], true)
            .await
            .unwrap();
    }
    let snapshot = budget.snapshot();
    assert_eq!(snapshot.llm_calls, 2);
    assert_eq!(snapshot.estimated_calls, 2);
    assert_eq!(snapshot.breakdown[0].model, "returned/model");
    assert_eq!(snapshot.breakdown[0].requested_model, "requested/model");
    assert!((snapshot.estimated_usd - 0.020122).abs() < 1e-12);
}

#[tokio::test]
async fn missing_usage_stays_unknown_and_explicit_zero_does_not_trigger_price_lookup() {
    for (field, known) in [(json!({}), false), (json!({"cost":0}), true)] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"choices":[{"message":{"content":"{}"}}],"usage":field})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let budget = BudgetTracker::new(1.0);
        manager(&server, &budget)
            .complete(vec![Message::user("test")], true)
            .await
            .unwrap();
        let stats = budget.snapshot();
        assert_eq!(stats.reported_calls, u64::from(known));
        assert_eq!(stats.unpriced_calls, u64::from(!known));
        assert_eq!(stats.missing_usage_calls, 1);
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}

#[tokio::test]
async fn budget_prevents_a_retry_after_a_charged_failure() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                json!({"error":{"code":500,"message":"temporary"},"usage":usage(0.25)}),
            ),
        )
        .expect(1)
        .mount(&server)
        .await;
    let budget = BudgetTracker::new(0.25);
    let error = manager(&server, &budget)
        .complete(vec![Message::user("test")], true)
        .await
        .unwrap_err();
    assert_eq!(error.code(), "budget_limit");
    assert_eq!(budget.snapshot().llm_calls, 1);
    assert_eq!(budget.spent_usd(), 0.25);
}

struct PaidImageScorer {
    calls: AtomicUsize,
}
#[async_trait]
impl LlmProvider for PaidImageScorer {
    async fn chat_completion(
        &self,
        _request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(CompletionResponse {
            content: json!({"scores":vec![0.9;15]}).to_string(),
            model: "scorer".into(),
            prompt_tokens: 100,
            completion_tokens: 20,
            total_tokens: 120,
            usage: LlmUsage {
                prompt_tokens: Some(100),
                completion_tokens: Some(20),
                cost_usd: Some(0.25),
                ..Default::default()
            },
        })
    }
    fn provider_name(&self) -> &str {
        "test"
    }
    async fn health_check(&self) -> Result<(), LlmError> {
        Ok(())
    }
}
#[tokio::test]
async fn ranking_preserves_the_paid_batch_when_its_cost_reaches_the_limit() {
    let budget = BudgetTracker::new(0.25);
    let provider = Arc::new(PaidImageScorer {
        calls: AtomicUsize::new(0),
    });
    let manager = LlmManager::with_provider(provider.clone(), LlmConfig::default())
        .with_accounting(budget.observer());
    let images = (0..30)
        .map(|i| query2table_lib::providers::search::ImageSearchResult {
            image_url: format!("https://example.com/{i}"),
            thumbnail_url: String::new(),
            title: format!("Image {i}"),
            source_url: String::new(),
            width: None,
            height: None,
        })
        .collect();
    let ranked = ImageRanker::rank("images", images, &manager, 0.7)
        .await
        .unwrap();
    assert_eq!(ranked.len(), 15);
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    assert_eq!(budget.snapshot().llm_calls, 1);
}

struct SearchStub {
    name: &'static str,
    fail: bool,
    calls: AtomicUsize,
}
#[async_trait]
impl SearchProvider for SearchStub {
    async fn search(&self, _query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            Err(SearchError::AuthError("test".into()))
        } else {
            Ok(vec![])
        }
    }
    fn provider_name(&self) -> &str {
        self.name
    }
    async fn health_check(&self) -> Result<(), SearchError> {
        Ok(())
    }
}
#[tokio::test]
async fn default_zero_search_prices_do_not_create_unknown_costs() {
    for name in ["brave", "serper"] {
        let provider = Arc::new(SearchStub {
            name, fail: false, calls: AtomicUsize::new(0),
        });
        let budget = BudgetTracker::new(1.0);
        let search = SearchManager::with_providers(provider, None, SearchConfig::default())
            .with_accounting(budget.observer());
        for _ in 0..25 { search.search("fixture").await.unwrap(); }
        let stats = budget.snapshot();
        assert_eq!(stats.search_calls, 25);
        assert_eq!(stats.estimated_calls, 25);
        assert_eq!(stats.unpriced_calls, 0);
        assert_eq!(stats.spent_usd, 0.0);
        assert_eq!(stats.breakdown[0].pricing.as_ref().unwrap().per_request, 0.0);
    }
}

#[tokio::test]
async fn fallback_attempts_are_counted_and_unpriced_failures_stay_visible() {
    let primary = Arc::new(SearchStub {
        name: "brave",
        fail: true,
        calls: AtomicUsize::new(0),
    });
    let fallback = Arc::new(SearchStub {
        name: "serper",
        fail: false,
        calls: AtomicUsize::new(0),
    });
    let budget = BudgetTracker::new(0.001);
    let search = SearchManager::with_providers(
        primary.clone(),
        Some(fallback.clone()),
        SearchConfig {
            primary: SearchBackend::Brave,
            serper_price_per_1000: Some(2.0),
            ..Default::default()
        },
    )
    .with_accounting(budget.observer());
    search.search("test").await.unwrap();
    assert!(matches!(
        search.search("blocked").await,
        Err(SearchError::BudgetExceeded)
    ));
    let stats = budget.snapshot();
    assert_eq!(stats.search_calls, 2);
    assert_eq!(stats.unpriced_calls, 1);
    assert_eq!(stats.estimated_calls, 1);
    assert_eq!(stats.spent_usd, 0.002);
    assert_eq!(primary.calls.load(Ordering::SeqCst), 1);
    assert_eq!(fallback.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_usage_is_recorded_once_for_each_actual_attempt() {
    let server = MockServer::start().await;
    let calls = Arc::new(AtomicUsize::new(0));
    let seen = calls.clone();
    Mock::given(method("POST"))
        .respond_with(move |_: &wiremock::Request| {
            let first = seen.fetch_add(1, Ordering::SeqCst) == 0;
            ResponseTemplate::new(200).set_body_json(if first {
                json!({"error":{"code":500,"message":"temporary"},"usage":usage(0.01)})
            } else {
                json!({"choices":[{"message":{"content":"{}"}}],"usage":usage(0.02)})
            })
        })
        .expect(2)
        .mount(&server)
        .await;
    let budget = BudgetTracker::new(1.0);
    manager(&server, &budget)
        .complete(vec![Message::user("test")], true)
        .await
        .unwrap();
    let stats = budget.snapshot();
    assert_eq!(stats.llm_calls, 2);
    assert_eq!(stats.prompt_tokens, 34);
    assert_eq!(stats.completion_tokens, 18);
    assert!((stats.reported_usd - 0.03).abs() < 1e-12);
}

#[tokio::test]
async fn aborted_http_attempt_is_retained_as_unknown_cost() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(10)))
        .mount(&server)
        .await;
    let budget = BudgetTracker::new(1.0);
    let manager = manager(&server, &budget);
    let task =
        tokio::spawn(async move { manager.complete(vec![Message::user("test")], true).await });
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while server.received_requests().await.unwrap().is_empty() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    let _ = task.await;
    let snapshot = budget.snapshot();
    assert_eq!(snapshot.llm_calls, 1);
    assert_eq!(snapshot.unpriced_calls, 1);
    assert_eq!(snapshot.pending_calls, 0);
    assert_eq!(snapshot.reported_calls, 0);
}

#[tokio::test]
async fn native_ollama_keeps_cached_tokens_and_never_prices_an_explicit_foreign_cloud_host() {
    use query2table_lib::providers::llm::ollama::OllamaProvider;
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path("/api/chat")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"model":"alias:cloud","remote_model":"deepseek-v4.1-flash","remote_host":"https://other-provider.example","prompt_eval_count":100,"prompt_eval_cached_count":80,"eval_count":20,"message":{"content":"ok"}}))).mount(&server).await;
    let provider = OllamaProvider::new(server.uri()).unwrap();
    let response = provider
        .chat_completion(CompletionRequest {
            messages: vec![Message::user("test")],
            model: "alias:cloud".into(),
            temperature: 0.0,
            max_tokens: 64,
            json_mode: false,
            reasoning_effort: Default::default(),
        })
        .await
        .unwrap();
    assert_eq!(response.usage.cached_prompt_tokens, Some(80));
    assert_eq!(response.usage.prompt_tokens, Some(100));
    assert!(provider
        .pricing("alias:cloud", &response.usage, chrono::Utc::now())
        .await
        .is_none());
}

struct UsageOnly;
#[async_trait]
impl LlmProvider for UsageOnly {
    async fn chat_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: "{}".into(),
            model: request.model,
            prompt_tokens: 1_000_000,
            completion_tokens: 1_000_000,
            total_tokens: 2_000_000,
            usage: LlmUsage {
                prompt_tokens: Some(1_000_000),
                completion_tokens: Some(1_000_000),
                ..Default::default()
            },
        })
    }
    fn provider_name(&self) -> &str {
        "openai_compatible"
    }
    async fn health_check(&self) -> Result<(), LlmError> {
        Ok(())
    }
}
#[tokio::test]
async fn manual_rates_never_leak_between_models_or_endpoints() {
    use query2table_lib::providers::llm::manager::LlmBackend;
    let scope =
        serde_json::to_string(&("openai_compatible", "http://localhost:8080/v1", "alpha")).unwrap();
    let config = LlmConfig {
        backend: LlmBackend::OpenAiCompatible,
        openai_model: "alpha".into(),
        openai_base_url: "http://localhost:8080/v1/".into(),
        pricing_overrides: json!({(scope):{"input_per_million":2.0,"output_per_million":8.0}})
            .to_string(),
        ..Default::default()
    };
    let budget = BudgetTracker::new(100.0);
    let manager = LlmManager::with_provider(Arc::new(UsageOnly), config.clone())
        .with_accounting(budget.observer());
    manager
        .complete(vec![Message::user("test")], true)
        .await
        .unwrap();
    manager
        .complete_with_model(vec![Message::user("test")], "beta", true)
        .await
        .unwrap();
    assert_eq!(budget.snapshot().estimated_usd, 10.0);
    assert_eq!(budget.snapshot().unpriced_calls, 1);
    let other_budget = BudgetTracker::new(100.0);
    let other = LlmManager::with_provider(
        Arc::new(UsageOnly),
        LlmConfig {
            openai_base_url: "http://another-endpoint/v1".into(),
            ..config
        },
    )
    .with_accounting(other_budget.observer());
    other
        .complete(vec![Message::user("test")], true)
        .await
        .unwrap();
    assert_eq!(other_budget.snapshot().unpriced_calls, 1);
    assert_eq!(other_budget.spent_usd(), 0.0);
}

struct TransientSearch { calls: AtomicUsize }
#[async_trait]
impl SearchProvider for TransientSearch {
    async fn search(&self,_query:SearchQuery)->Result<Vec<SearchResult>,SearchError> {
        if self.calls.fetch_add(1,Ordering::SeqCst)<2 {Err(SearchError::ConnectionError("temporary".into()))} else {Ok(vec![])}
    }
    fn provider_name(&self)->&str {"brave"}
    async fn health_check(&self)->Result<(),SearchError>{Ok(())}
}
#[tokio::test]
async fn search_retries_count_as_attempts_not_as_one_logical_query() {
    let provider=Arc::new(TransientSearch {calls:AtomicUsize::new(0)});let budget=BudgetTracker::new(1.0);
    let search=SearchManager::with_providers(provider.clone(),None,SearchConfig {brave_price_per_1000:Some(5.0),..Default::default()}).with_accounting(budget.observer());
    search.search("test").await.unwrap();let stats=budget.snapshot();
    assert_eq!(provider.calls.load(Ordering::SeqCst),3);assert_eq!(stats.search_calls,3);assert_eq!(stats.unpriced_calls,2);assert_eq!(stats.estimated_usd,0.005);
}

#[test]
fn invalid_pricing_settings_are_rejected_before_persistence() {
    use query2table_lib::providers::accounting::validate_pricing_setting;
    assert!(validate_pricing_setting("brave_price_per_1000", "").is_ok());
    assert!(validate_pricing_setting("brave_price_per_1000", "0").is_ok());
    for value in ["-1","NaN","inf"] { assert!(validate_pricing_setting("brave_price_per_1000",value).is_err()); }
    let scope=serde_json::to_string(&("ollama","http://localhost:11434","model")).unwrap();
    assert!(validate_pricing_setting("llm_pricing_overrides",&json!({(scope.clone()):{"input_per_million":0,"output_per_million":0}}).to_string()).is_ok());
    assert!(validate_pricing_setting("llm_pricing_overrides",&json!({(scope):{"input_per_million":null,"output_per_million":0}}).to_string()).is_err());
}

#[tokio::test]
async fn research_does_not_buy_a_final_answer_after_reaching_its_budget() {
    use query2table_lib::{storage::{db::Database,repository::Repository},orchestrator::{pipeline::{PipelineConfig,PipelineState},research_pipeline::ResearchPipeline}};
    use std::collections::HashMap;
    let server=MockServer::start().await;
    Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"choices":[{"message":{"content":"{\"action\":\"think\",\"thought\":\"Planning\"}"}}],"usage":{"prompt_tokens":17,"completion_tokens":9,"cost_usd":0.01}}))).expect(1).mount(&server).await;
    let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    let db=Database::with_pool(pool.clone()).await;db.migrate().await.unwrap();let repo=Arc::new(Repository::new(pool));
    let settings=HashMap::from([("llm_provider".into(),"openai_compatible".into()),("openai_base_url".into(),format!("{}/v1",server.uri())),("openai_model".into(),"test-model".into()),("brave_api_key".into(),"test-key".into()),("max_budget_usd".into(),"0.01".into())]);
    let (pipeline,_commands)=ResearchPipeline::new("research-cost".into(),"Test research".into(),PipelineConfig::from_settings(&settings),repo.clone(),None);
    assert_eq!(pipeline.run().await.unwrap(),PipelineState::Completed);
    let run=repo.get_run("research-cost").await.unwrap().unwrap();let stats:serde_json::Value=serde_json::from_str(run.stats.as_ref().unwrap()).unwrap();
    assert_eq!(stats["accounting"]["llm_calls"],1);assert_eq!(stats["accounting"]["reported_usd"],0.01);
}

#[tokio::test]
async fn a_follow_up_records_which_question_an_unreadable_reply_belonged_to() {
    use query2table_lib::{storage::{db::Database,repository::Repository},orchestrator::{pipeline::{PipelineConfig,PipelineState},research_pipeline::ResearchPipeline}};
    use std::collections::HashMap;
    let server=MockServer::start().await;
    let reply=|content:&str| ResponseTemplate::new(200).set_body_json(json!({"choices":[{"message":{"content":content}}],"usage":{"prompt_tokens":5,"completion_tokens":3}}));
    Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(reply("not a tool call")).up_to_n_times(1).with_priority(1).mount(&server).await;
    Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(reply("{\"action\":\"answer\",\"markdown\":\"Done.\"}")).with_priority(2).mount(&server).await;
    let pool=sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    let db=Database::with_pool(pool.clone()).await;db.migrate().await.unwrap();let repo=Arc::new(Repository::new(pool));
    repo.create_run_with_type("talk","First question","{}","research").await.unwrap();
    let settings=HashMap::from([("llm_provider".into(),"openai_compatible".into()),("openai_base_url".into(),format!("{}/v1",server.uri())),("openai_model".into(),"test-model".into()),("brave_api_key".into(),"test-key".into())]);
    let (pipeline,_commands)=ResearchPipeline::follow_up("talk".into(),"Second question".into(),1,vec![],PipelineConfig::from_settings(&settings),repo.clone(),None);
    assert_eq!(pipeline.run().await.unwrap(),PipelineState::Completed);
    let issues=repo.get_run_llm_issue_details("talk").await.unwrap();
    assert_eq!(issues.len(),1);
    let issue:query2table_lib::providers::llm::LlmIssue=serde_json::from_str(&issues[0]).unwrap();
    assert_eq!((issue.code.as_str(),issue.stage.as_deref(),issue.outcome.as_deref(),issue.turn_index),("invalid_response",Some("research"),Some("continued"),Some(1)));
}
