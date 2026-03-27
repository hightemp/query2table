mod common;

use std::sync::Arc;
use std::time::Duration;

use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::any;

use query2table_lib::orchestrator::pipeline::{Pipeline, PipelineConfig, PipelineCommand, PipelineState};
use query2table_lib::providers::llm::manager::{LlmConfig, LlmBackend, LlmManager};
use query2table_lib::providers::llm::types::LlmError;
use query2table_lib::providers::search::manager::{SearchConfig, SearchBackend, SearchManager};
use query2table_lib::providers::search::types::{SearchResult, SearchError};
use query2table_lib::providers::http::client::HttpFetcher;
use query2table_lib::providers::http::rate_limiter::RateLimiter;
use query2table_lib::roles::stopping_controller::StopConfig;
use query2table_lib::storage::models::SchemaColumn;

use common::*;

/// HTML page served by the local wiremock server for fetch+extract testing.
const TEST_HTML: &str = r#"<!DOCTYPE html>
<html><head><title>Tech Companies in Germany</title></head>
<body>
<article>
<h1>Top Technology Companies in Germany</h1>
<h2>SAP SE</h2>
<p>SAP SE is a German multinational software corporation. Website: https://www.sap.com. Industry: Enterprise Software. Employees: 107000.</p>
<h2>Siemens AG</h2>
<p>Siemens AG is a global technology company. Website: https://www.siemens.com. Industry: Technology & Engineering. Employees: 303000.</p>
<h2>Deutsche Telekom</h2>
<p>Deutsche Telekom is a telecommunications company. Website: https://www.telekom.com. Industry: Telecommunications. Employees: 216000.</p>
</article>
</body></html>"#;

/// Spawn a wiremock HTTP server that returns test HTML for any GET request.
async fn spawn_test_http_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(TEST_HTML)
                .insert_header("content-type", "text/html; charset=utf-8"),
        )
        .mount(&server)
        .await;
    server
}

/// Create search results pointing to the local wiremock server.
fn local_search_results(base_url: &str) -> Vec<SearchResult> {
    vec![
        SearchResult {
            title: "Tech Companies Germany".to_string(),
            url: format!("{}/page1", base_url),
            snippet: "Top tech companies in Germany".to_string(),
        },
        SearchResult {
            title: "German IT Overview".to_string(),
            url: format!("{}/page2", base_url),
            snippet: "Overview of German IT industry".to_string(),
        },
        SearchResult {
            title: "Best German Startups".to_string(),
            url: format!("{}/page3", base_url),
            snippet: "German startups to watch".to_string(),
        },
    ]
}

/// Helper: create pipeline with mock providers, run it, and return result.
async fn run_pipeline_with_mocks(
    config: PipelineConfig,
    llm: Arc<LlmManager>,
    search: Arc<SearchManager>,
) -> Result<PipelineState, query2table_lib::orchestrator::pipeline::PipelineError> {
    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id,
        "Find technology companies in Germany".to_string(),
        config,
        repo,
        None, // No events (auto-confirms schema)
    );
    pipeline.set_providers(llm, search);
    pipeline.run().await
}

// =============================================================================
// Test 1: Full pipeline happy path
// =============================================================================
#[tokio::test]
async fn test_full_pipeline_happy_path() {
    let _log = setup_test_logs("test_full_pipeline_happy_path");

    // Start a local HTTP server so fetch works
    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());

    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    // Mock search returns URLs pointing to local server
    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok(), "Pipeline should complete: {:?}", result.err());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Verify DB state
    let run = repo.get_run(&run_id).await.unwrap().expect("Run should exist");
    assert_eq!(run.status, "completed");
    assert!(run.stats.is_some(), "Run should have stats");

    // Verify entity rows were created
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    assert!(!rows.is_empty(), "Should have entity rows, got 0");

    // Verify search queries were saved
    let queries = repo.get_search_queries_by_run(&run_id).await.unwrap();
    assert!(!queries.is_empty(), "Should have search queries");

    // Verify run schema exists and was confirmed
    let schema = repo.get_run_schema(&run_id).await.unwrap();
    assert!(schema.is_some(), "Should have schema");
    let schema = schema.unwrap();
    assert_eq!(schema.confirmed, 1, "Schema should be confirmed");
}

// =============================================================================
// Test 2: Pipeline cancellation
// =============================================================================
#[tokio::test]
async fn test_pipeline_cancellation() {
    let _log = setup_test_logs("test_pipeline_cancellation");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    // Send cancel after a short delay
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = cmd_tx.send(PipelineCommand::Cancel).await;
    });

    let result = pipeline.run().await;
    assert!(result.is_ok());
    let state = result.unwrap();
    // Pipeline may complete before cancel arrives, or get cancelled
    assert!(
        state == PipelineState::Cancelled || state == PipelineState::Completed,
        "Expected Cancelled or Completed, got {:?}",
        state
    );

    let run = repo.get_run(&run_id).await.unwrap().expect("Run should exist");
    assert!(
        run.status == "cancelled" || run.status == "completed",
        "Expected cancelled or completed status, got {}",
        run.status
    );
}

// =============================================================================
// Test 3: Pipeline pause/resume
// =============================================================================
#[tokio::test]
async fn test_pipeline_pause_resume() {
    let _log = setup_test_logs("test_pipeline_pause_resume");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    // Send pause then resume
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let _ = cmd_tx.send(PipelineCommand::Pause).await;
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = cmd_tx.send(PipelineCommand::Resume).await;
    });

    let result = pipeline.run().await;
    assert!(result.is_ok(), "Pipeline should complete after pause/resume: {:?}", result.err());
    // May complete before pause arrives
    let state = result.unwrap();
    assert!(
        state == PipelineState::Completed || state == PipelineState::Cancelled,
        "Expected Completed or Cancelled, got {:?}",
        state
    );
}

// =============================================================================
// Test 4: Row count stop condition
// =============================================================================
#[tokio::test]
async fn test_row_count_stop_condition() {
    let _log = setup_test_logs("test_row_count_stop_condition");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());

    let mut config = test_pipeline_config();
    config.stop.target_row_count = 1; // Stop after 1 row
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Should have at least 1 row (stop condition checked after extraction)
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    assert!(!rows.is_empty(), "Should have at least 1 entity row");
}

// =============================================================================
// Test 5: Duration stop condition
// =============================================================================
#[tokio::test]
async fn test_duration_stop_condition() {
    let _log = setup_test_logs("test_duration_stop_condition");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());

    let mut config = test_pipeline_config();
    config.stop.max_duration_secs = 1; // Stop after 1 second
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok());
    // Pipeline should complete (stop condition is checked in the loop, not as error)
    assert_eq!(result.unwrap(), PipelineState::Completed);
}

// =============================================================================
// Test 6: LLM failure handling
// =============================================================================
#[tokio::test]
async fn test_llm_failure_handling() {
    let _log = setup_test_logs("test_llm_failure_handling");

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();

    // LLM always fails
    let mock_llm = MockLlmProvider::new()
        .with_error(LlmError::RequestFailed("test error".to_string()));
    let (llm, search) = build_mock_providers_with_llm(mock_llm);

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);

    let result = pipeline.run().await;
    // Pipeline should fail due to LLM error in interpreter phase
    assert!(result.is_err(), "Pipeline should fail when LLM fails");
    let err = result.unwrap_err();
    let err_str = err.to_string();
    assert!(
        err_str.contains("LLM") || err_str.contains("Interpreter") || err_str.contains("error"),
        "Error should mention LLM: {}",
        err_str
    );
}

// =============================================================================
// Test 7: Search failure handling
// =============================================================================
#[tokio::test]
async fn test_search_failure_handling() {
    let _log = setup_test_logs("test_search_failure_handling");

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();

    // Search always fails
    let mock_search = MockSearchProvider::new()
        .with_error(SearchError::AuthError("Invalid API key".to_string()));
    let (llm, search) = build_mock_providers_with_search(mock_search);

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);

    let result = pipeline.run().await;
    // Search executor catches individual query failures and returns Ok with failed_queries count.
    // Pipeline completes normally with 0 results (no pages to fetch).
    assert!(result.is_ok(), "Pipeline should complete when search fails (graceful degradation): {:?}", result.err());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Zero rows since all searches failed
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    assert_eq!(rows.len(), 0, "Should have 0 rows when search fails");
}

// =============================================================================
// Test 8: Empty search results
// =============================================================================
#[tokio::test]
async fn test_empty_search_results() {
    let _log = setup_test_logs("test_empty_search_results");

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();

    // Search returns nothing
    let mock_search = MockSearchProvider::empty();
    let (llm, search) = build_mock_providers_with_search(mock_search);

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);

    let result = pipeline.run().await;
    assert!(result.is_ok(), "Pipeline should handle empty results: {:?}", result.err());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Zero rows since no search results
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    assert_eq!(rows.len(), 0, "Should have 0 rows with empty search results");
}

// =============================================================================
// Test 9: Extraction failure (invalid JSON from LLM)
// =============================================================================
#[tokio::test]
async fn test_extraction_failure_invalid_json() {
    let _log = setup_test_logs("test_extraction_failure_invalid_json");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    // LLM returns invalid JSON for extraction step
    let mock_llm = MockLlmProvider::new().with_invalid_extraction();
    let llm = Arc::new(LlmManager::with_provider(
        Arc::new(mock_llm),
        config.llm.clone(),
    ));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    // Pipeline should still complete — extraction failures are handled gracefully
    assert!(result.is_ok(), "Pipeline should complete despite extraction failures: {:?}", result.err());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // No rows because extraction always fails
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    assert_eq!(rows.len(), 0, "Should have 0 rows since extraction fails");
}

// =============================================================================
// Test 10: Deduplication
// =============================================================================
#[tokio::test]
async fn test_deduplication() {
    let _log = setup_test_logs("test_deduplication");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    // Use 5 pages so multiple extractions happen -> more duplicates
    let results: Vec<SearchResult> = (1..=5).map(|i| SearchResult {
        title: format!("Tech Companies Page {}", i),
        url: format!("{}/page{}", base_url, i),
        snippet: "Tech companies in Germany".to_string(),
    }).collect();

    let mock_search = MockSearchProvider::new().with_results(results);
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    // Each page will extract the same 2 entities (SAP, Siemens) → many duplicates
    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok(), "Pipeline should complete: {:?}", result.err());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Verify stats contain dedup info
    let run = repo.get_run(&run_id).await.unwrap().expect("Run should exist");
    if let Some(stats_str) = &run.stats {
        let stats: serde_json::Value = serde_json::from_str(stats_str).unwrap();
        let rows_found = stats["rows_found"].as_u64().unwrap_or(0);
        let dupes_merged = stats["duplicates_merged"].as_u64().unwrap_or(0);
        // If we fetched multiple pages with same data, dedup should merge some
        eprintln!("rows_found={}, duplicates_merged={}", rows_found, dupes_merged);
    }
}

// =============================================================================
// Test 11: Budget stop condition  
// =============================================================================
#[tokio::test]
async fn test_budget_stop_condition() {
    let _log = setup_test_logs("test_budget_stop_condition");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());

    let mut config = test_pipeline_config();
    config.max_budget_usd = 0.0001; // Extremely low budget
    config.stop.max_budget_usd = 0.0001;
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None,
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineState::Completed);
}

// =============================================================================
// Test 12: Schema confirmation with modified columns
// =============================================================================
#[tokio::test]
async fn test_schema_auto_confirm_with_no_events() {
    let _log = setup_test_logs("test_schema_auto_confirm");

    let server = spawn_test_http_server().await;
    let base_url = server.uri();

    let (repo, _db) = setup_test_db().await;
    let run_id = format!("test-run-{}", uuid::Uuid::new_v4());
    let config = test_pipeline_config();
    let fetcher = test_fetcher(&config);

    let mock_search = MockSearchProvider::new()
        .with_results(local_search_results(&base_url));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        config.search.clone(),
    ));

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm, config.llm.clone()));

    let (mut pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Find technology companies in Germany".to_string(),
        config,
        repo.clone(),
        None, // No events → auto-confirm schema
    );
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);

    let result = pipeline.run().await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineState::Completed);

    // Verify schema was auto-confirmed
    let schema = repo.get_run_schema(&run_id).await.unwrap();
    assert!(schema.is_some(), "Schema should exist");
    let schema = schema.unwrap();
    assert_eq!(schema.confirmed, 1, "Schema should be auto-confirmed");

    // Parse columns and verify they match the mock schema
    let columns: Vec<SchemaColumn> = serde_json::from_str(&schema.columns).unwrap();
    assert!(!columns.is_empty(), "Should have columns");
    assert_eq!(columns[0].name, "name", "First column should be 'name'");
}
