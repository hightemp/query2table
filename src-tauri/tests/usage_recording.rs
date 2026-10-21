//! Recording usage while runs work: the run's accounting writes each finished model call,
//! search and download to the usage journal, with the bytes sent and received.

use std::sync::Arc;

use query2table_lib::orchestrator::budget_tracker::BudgetTracker;
use query2table_lib::orchestrator::usage_journal::UsageJournal;
use query2table_lib::providers::accounting::{PriceQuote, RequestInfo, RequestKind};
use query2table_lib::providers::http::client::HttpFetcher;
use query2table_lib::providers::http::rate_limiter::RateLimiter;
use query2table_lib::providers::llm::{
    anthropic::AnthropicProvider,
    manager::{LlmBackend, LlmConfig, LlmManager},
    openai_compatible::OpenAiCompatibleProvider,
    LlmProvider, LlmUsage, Message,
};
use query2table_lib::providers::search::{brave::BraveSearchProvider, manager::SearchManager};
use query2table_lib::storage::{
    db::Database,
    repository::Repository,
    usage::{UsageEvent, UsageRun},
};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn new_repo() -> Arc<Repository> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    Arc::new(Repository::new(pool))
}

fn run() -> UsageRun {
    UsageRun { run_id: "r1".into(), run_type: "research".into(), title: "Red pandas".into(), started_at: chrono::Utc::now().timestamp() }
}

async fn tracked(repo: &Arc<Repository>) -> BudgetTracker {
    BudgetTracker::new(10.0).with_journal(UsageJournal::start(repo.clone(), run()))
}

async fn events(repo: &Repository) -> Vec<UsageEvent> {
    repo.usage_events(0, i64::MAX).await.unwrap()
}

#[tokio::test]
async fn finished_calls_searches_and_downloads_are_journaled() {
    let repo = new_repo().await;
    let budget = tracked(&repo).await;
    let observer = budget.observer();
    let before = chrono::Utc::now().timestamp();

    let call = observer.begin(RequestInfo { kind: RequestKind::Llm, provider: "openrouter".into(), model: "m".into() }).unwrap();
    observer.capture(
        call,
        LlmUsage {
            prompt_tokens: Some(100),
            completion_tokens: Some(20),
            reasoning_tokens: Some(5),
            cached_prompt_tokens: Some(40),
            cache_write_tokens: Some(10),
            cost_usd: Some(0.01),
            model: Some("m-actual".into()),
            bytes_sent: Some(3000),
            bytes_received: Some(800),
            ..Default::default()
        },
    );
    observer.finish(call, None);

    let search = observer.begin(RequestInfo { kind: RequestKind::Search, provider: "brave".into(), model: "web".into() }).unwrap();
    observer.capture(search, LlmUsage { bytes_received: Some(5000), ..Default::default() });
    let rate = PriceQuote { per_request: 0.005, source: "Manual search rate".into(), ..Default::default() };
    observer.finish(search, Some(rate));

    observer.downloaded(12_345);
    // A call still waiting when the run ends counts without a price.
    observer.begin(RequestInfo { kind: RequestKind::Llm, provider: "ollama".into(), model: "x".into() }).unwrap();
    budget.finalize_pending();
    budget.flush_journal().await;

    let rows = events(&repo).await;
    assert!(rows.iter().all(|e| e.at >= before && e.run_id.as_deref() == Some("r1") && e.run_type == "research" && e.calls == 1));
    let brief: Vec<_> = rows
        .iter()
        .map(|e| (e.kind.as_str(), e.provider.as_str(), e.model.as_str(), e.reported_usd, e.estimated_usd, e.unpriced_calls, e.bytes_sent, e.bytes_received))
        .collect();
    assert_eq!(
        brief,
        [
            ("llm", "openrouter", "m-actual", 0.01, 0.0, 0, 3000, 800),
            ("search", "brave", "web", 0.0, 0.005, 0, 0, 5000),
            ("download", "", "", 0.0, 0.0, 0, 0, 12_345),
            ("llm", "ollama", "x", 0.0, 0.0, 1, 0, 0),
        ]
    );
    let first = &rows[0];
    assert_eq!(
        (first.prompt_tokens, first.completion_tokens, first.reasoning_tokens, first.cached_tokens, first.cache_write_tokens),
        (100, 20, 5, 40, 10)
    );
    assert_eq!(budget.snapshot().fetch_calls, 1);
    assert_eq!(repo.usage_runs().await.unwrap(), [run()]);
}

#[tokio::test]
async fn a_tracker_without_a_journal_records_nothing() {
    let repo = new_repo().await;
    let budget = BudgetTracker::new(10.0);
    budget.observer().downloaded(10);
    budget.flush_journal().await;
    assert!(events(&repo).await.is_empty());
    assert_eq!(budget.snapshot().fetch_calls, 1);
}

/// Sends one completion through the manager with the run's accounting; returns the request the
/// server got and the response it gave.
async fn complete_through(provider: Arc<dyn LlmProvider>, config: LlmConfig, budget: &BudgetTracker) {
    let manager = LlmManager::with_provider(provider, config).with_accounting(budget.observer());
    manager.complete(vec![Message::user("Hello there")], false).await.unwrap();
    budget.flush_journal().await;
}

async fn sizes(server: &MockServer, answer: &serde_json::Value) -> (i64, i64) {
    let requests = server.received_requests().await.unwrap();
    (requests.last().unwrap().body.len() as i64, answer.to_string().len() as i64)
}

#[tokio::test]
async fn models_report_the_bytes_they_send_and_receive() {
    // OpenAI-compatible servers (and the presets and OpenRouter through the same client).
    let server = MockServer::start().await;
    let answer = json!({"model": "local", "choices": [{"message": {"content": "Hi"}, "finish_reason": "stop"}], "usage": {"prompt_tokens": 9, "completion_tokens": 1}});
    Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(ResponseTemplate::new(200).set_body_string(answer.to_string())).mount(&server).await;
    let repo = new_repo().await;
    let budget = tracked(&repo).await;
    let provider = OpenAiCompatibleProvider::new(format!("{}/v1", server.uri()), String::new(), false).unwrap();
    let config = LlmConfig { backend: LlmBackend::OpenAiCompatible, openai_model: "local".into(), ..Default::default() };
    complete_through(Arc::new(provider), config, &budget).await;
    let (sent, received) = sizes(&server, &answer).await;
    let row = events(&repo).await.pop().unwrap();
    assert_eq!((row.bytes_sent, row.bytes_received), (sent, received));
    assert!(sent > 20, "{sent}");

    // Anthropic.
    let server = MockServer::start().await;
    let answer = json!({"model": "claude", "content": [{"type": "text", "text": "Hi"}], "stop_reason": "end_turn", "usage": {"input_tokens": 9, "output_tokens": 1}});
    Mock::given(method("POST")).and(path("/v1/messages")).respond_with(ResponseTemplate::new(200).set_body_string(answer.to_string())).mount(&server).await;
    let repo = new_repo().await;
    let budget = tracked(&repo).await;
    let provider = AnthropicProvider::new(format!("{}/v1", server.uri()), "key".into()).unwrap();
    let config = LlmConfig { backend: LlmBackend::Anthropic, anthropic_api_key: "key".into(), anthropic_model: "claude".into(), ..Default::default() };
    complete_through(Arc::new(provider), config, &budget).await;
    let (sent, received) = sizes(&server, &answer).await;
    let row = events(&repo).await.pop().unwrap();
    assert_eq!((row.bytes_sent, row.bytes_received), (sent, received));
}

#[tokio::test]
async fn page_downloads_count_with_their_size() {
    let server = MockServer::start().await;
    let page = "<html><body>".to_string() + &"x".repeat(5000) + "</body></html>";
    Mock::given(method("GET")).and(path("/page")).respond_with(ResponseTemplate::new(200).set_body_string(page.clone()).insert_header("content-type", "text/html")).mount(&server).await;
    let repo = new_repo().await;
    let budget = tracked(&repo).await;
    let fetcher = HttpFetcher::new_no_proxy(RateLimiter::new(std::time::Duration::from_millis(1))).with_accounting(budget.observer());
    fetcher.fetch(&format!("{}/page", server.uri())).await.unwrap();
    // A failed download moves no content.
    fetcher.fetch(&format!("{}/missing", server.uri())).await.unwrap_err();
    budget.flush_journal().await;
    let rows = events(&repo).await;
    assert_eq!(rows.iter().map(|e| (e.kind.as_str(), e.bytes_received)).collect::<Vec<_>>(), [("download", page.len() as i64)]);
    assert_eq!(budget.snapshot().fetch_calls, 1);
}

#[tokio::test]
async fn searches_report_their_provider_price_and_response_size() {
    let server = MockServer::start().await;
    let answer = json!({"web": {"results": [{"title": "Red panda", "url": "https://example.com/panda", "description": "A small mammal."}]}});
    Mock::given(method("GET")).and(path("/web/search")).respond_with(ResponseTemplate::new(200).set_body_string(answer.to_string())).mount(&server).await;
    let repo = new_repo().await;
    let budget = tracked(&repo).await;
    let brave = BraveSearchProvider::new("key").with_base_url(server.uri());
    let config = query2table_lib::providers::search::manager::SearchConfig { brave_price_per_1000: Some(5.0), ..Default::default() };
    let search = SearchManager::with_providers(Arc::new(brave), None, config).with_accounting(budget.observer());
    search.search("red panda").await.unwrap();
    budget.flush_journal().await;
    let row = events(&repo).await.pop().unwrap();
    assert_eq!((row.kind.as_str(), row.provider.as_str(), row.estimated_usd, row.bytes_received), ("search", "brave", 0.005, answer.to_string().len() as i64));
}
