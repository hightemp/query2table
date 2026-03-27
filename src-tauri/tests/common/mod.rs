use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::path::PathBuf;
use std::fs;

use async_trait::async_trait;
use sqlx::sqlite::SqlitePoolOptions;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

use query2table_lib::providers::llm::types::*;
use query2table_lib::providers::llm::manager::{LlmConfig, LlmBackend, LlmManager};
use query2table_lib::providers::search::types::*;
use query2table_lib::providers::search::manager::{SearchConfig, SearchBackend, SearchManager};
use query2table_lib::providers::http::client::HttpFetcher;
use query2table_lib::providers::http::rate_limiter::RateLimiter;
use query2table_lib::storage::db::Database;
use query2table_lib::storage::repository::Repository;
use query2table_lib::orchestrator::pipeline::PipelineConfig;
use query2table_lib::roles::stopping_controller::StopConfig;

/// Set up an in-memory SQLite database with full migration for testing.
pub async fn setup_test_db() -> (Arc<Repository>, Database) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool");
    let db = Database::with_pool(pool).await;
    db.migrate().await.expect("Failed to run migrations");
    let repo = Arc::new(Repository::new(db.pool().clone()));
    (repo, db)
}

/// Initialize test log capture to a file.
/// Returns a guard that must be held for the duration of the test.
pub fn setup_test_logs(test_name: &str) -> tracing::subscriber::DefaultGuard {
    let logs_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("logs");
    fs::create_dir_all(&logs_dir).ok();

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let log_path = logs_dir.join(format!("{}_{}.log", test_name, timestamp));

    let file = fs::File::create(&log_path).expect("Failed to create log file");
    let file_layer = fmt::layer()
        .with_writer(std::sync::Mutex::new(file))
        .with_ansi(false)
        .with_target(true)
        .with_level(true);

    let subscriber = tracing_subscriber::registry()
        .with(file_layer)
        .with(tracing_subscriber::filter::LevelFilter::DEBUG);

    // Use set_default for thread-local (works for single-thread runtime).
    // For multi_thread tests, logs from spawned tasks on other threads
    // may not appear in this file — use set_global_default instead if needed.
    let guard = tracing::subscriber::set_default(subscriber);

    eprintln!("Test logs: {}", log_path.display());
    guard
}

// --- Mock LLM Provider ---

/// A mock LLM provider that routes responses based on system prompt content.
pub struct MockLlmProvider {
    call_count: AtomicUsize,
    /// If set, return this error on all calls.
    error: Option<LlmError>,
    /// Fixed responses by role keyword detection in system prompt.
    interpret_response: String,
    schema_response: String,
    search_plan_response: String,
    expand_response: String,
    extract_response: String,
    /// If set, return invalid JSON for extraction to test error handling.
    extract_returns_invalid: bool,
}

impl MockLlmProvider {
    pub fn new() -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            error: None,
            interpret_response: include_str!("../fixtures/interpret_response.json").to_string(),
            schema_response: include_str!("../fixtures/schema_response.json").to_string(),
            search_plan_response: include_str!("../fixtures/search_plan_response.json").to_string(),
            expand_response: include_str!("../fixtures/expand_response.json").to_string(),
            extract_response: include_str!("../fixtures/extract_response.json").to_string(),
            extract_returns_invalid: false,
        }
    }

    pub fn with_error(mut self, error: LlmError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn with_invalid_extraction(mut self) -> Self {
        self.extract_returns_invalid = true;
        self
    }

    pub fn with_extract_response(mut self, response: String) -> Self {
        self.extract_response = response;
        self
    }

    pub fn call_count(&self) -> usize {
        self.call_count.load(Ordering::Relaxed)
    }

    fn detect_role(&self, request: &CompletionRequest) -> &str {
        let system_msg = request.messages.iter()
            .find(|m| matches!(m.role, MessageRole::System))
            .map(|m| m.content.as_str())
            .unwrap_or("");

        if system_msg.contains("query interpreter") || system_msg.contains("entity_type") {
            "interpret"
        } else if system_msg.contains("schema designer") || system_msg.contains("column") && system_msg.contains("snake_case") {
            "schema"
        } else if system_msg.contains("search strategist") || system_msg.contains("search queries") && system_msg.contains("priority") {
            "search_plan"
        } else if system_msg.contains("multilingual search query expander") || system_msg.contains("query expander") {
            "expand"
        } else if system_msg.contains("data extraction specialist") || system_msg.contains("extraction") {
            "extract"
        } else {
            "unknown"
        }
    }
}

#[async_trait]
impl LlmProvider for MockLlmProvider {
    async fn chat_completion(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.call_count.fetch_add(1, Ordering::Relaxed);

        if let Some(ref err) = self.error {
            return Err(match err {
                LlmError::RequestFailed(msg) => LlmError::RequestFailed(msg.clone()),
                LlmError::AuthError => LlmError::AuthError,
                LlmError::RateLimited { retry_after_ms } => LlmError::RateLimited { retry_after_ms: *retry_after_ms },
                LlmError::NotConfigured(msg) => LlmError::NotConfigured(msg.clone()),
                LlmError::ConnectionError(msg) => LlmError::ConnectionError(msg.clone()),
                LlmError::ParseError(msg) => LlmError::ParseError(msg.clone()),
                LlmError::ModelNotFound(msg) => LlmError::ModelNotFound(msg.clone()),
            });
        }

        let role = self.detect_role(&request);
        let content = match role {
            "interpret" => self.interpret_response.clone(),
            "schema" => self.schema_response.clone(),
            "search_plan" => self.search_plan_response.clone(),
            "expand" => self.expand_response.clone(),
            "extract" => {
                if self.extract_returns_invalid {
                    "not valid json {{{".to_string()
                } else {
                    self.extract_response.clone()
                }
            }
            _ => r#"{"error": "unknown role"}"#.to_string(),
        };

        Ok(CompletionResponse {
            content,
            model: "mock-model".to_string(),
            prompt_tokens: 100,
            completion_tokens: 200,
            total_tokens: 300,
        })
    }

    fn provider_name(&self) -> &str {
        "mock"
    }

    async fn health_check(&self) -> Result<(), LlmError> {
        Ok(())
    }
}

// --- Mock Search Provider ---

/// A mock search provider that returns configurable fixture results.
pub struct MockSearchProvider {
    results: Vec<SearchResult>,
    error: Option<SearchError>,
    name: String,
}

impl MockSearchProvider {
    pub fn new() -> Self {
        Self {
            results: default_search_results(),
            error: None,
            name: "mock_search".to_string(),
        }
    }

    pub fn with_results(mut self, results: Vec<SearchResult>) -> Self {
        self.results = results;
        self
    }

    pub fn with_error(mut self, error: SearchError) -> Self {
        self.error = Some(error);
        self
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn empty() -> Self {
        Self {
            results: Vec::new(),
            error: None,
            name: "mock_search".to_string(),
        }
    }
}

#[async_trait]
impl SearchProvider for MockSearchProvider {
    async fn search(&self, _query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        if let Some(ref err) = self.error {
            return Err(match err {
                SearchError::RequestFailed(msg) => SearchError::RequestFailed(msg.clone()),
                SearchError::AuthError(msg) => SearchError::AuthError(msg.clone()),
                SearchError::RateLimited { retry_after_secs } => SearchError::RateLimited { retry_after_secs: *retry_after_secs },
                SearchError::NotConfigured(msg) => SearchError::NotConfigured(msg.clone()),
                SearchError::ConnectionError(msg) => SearchError::ConnectionError(msg.clone()),
                SearchError::ParseError(msg) => SearchError::ParseError(msg.clone()),
            });
        }
        Ok(self.results.clone())
    }

    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn health_check(&self) -> Result<(), SearchError> {
        Ok(())
    }
}

fn default_search_results() -> Vec<SearchResult> {
    vec![
        SearchResult {
            title: "Top Tech Companies in Germany - TechCrunch".to_string(),
            url: "https://example.com/tech-companies-germany".to_string(),
            snippet: "A comprehensive list of the largest technology companies in Germany.".to_string(),
        },
        SearchResult {
            title: "German IT Industry Overview".to_string(),
            url: "https://example.com/german-it-overview".to_string(),
            snippet: "Overview of the German IT industry and major players.".to_string(),
        },
        SearchResult {
            title: "Best German Startups 2024".to_string(),
            url: "https://example.com/german-startups".to_string(),
            snippet: "The best German tech startups to watch.".to_string(),
        },
    ]
}

// --- Test Pipeline Config ---

/// Create a PipelineConfig suitable for tests.
pub fn test_pipeline_config() -> PipelineConfig {
    PipelineConfig {
        llm: LlmConfig {
            backend: LlmBackend::OpenRouter,
            openrouter_api_key: "test-key".to_string(),
            openrouter_model: "mock-model".to_string(),
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
            temperature: 0.2,
            max_tokens: 4096,
        },
        search: SearchConfig {
            primary: SearchBackend::Brave,
            brave_api_key: "test-key".to_string(),
            serper_api_key: String::new(),
            num_results: 10,
        },
        stop: StopConfig {
            target_row_count: 100,
            max_budget_usd: 10.0,
            max_duration_secs: 300,
            saturation_threshold: 0.05,
        },
        max_parallel_fetches: 2,
        max_parallel_extractions: 1,
        min_confidence: 0.3,
        dedup_similarity: 0.85,
        max_budget_usd: 10.0,
        rate_limit_ms: 100, // Fast for tests
    }
}

/// Build mock LLM and Search managers for pipeline injection.
pub fn build_mock_providers() -> (Arc<LlmManager>, Arc<SearchManager>) {
    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(
        mock_llm,
        test_pipeline_config().llm,
    ));

    let mock_search = Arc::new(MockSearchProvider::new());
    let search = Arc::new(SearchManager::with_providers(
        mock_search,
        None,
        test_pipeline_config().search,
    ));

    (llm, search)
}

/// Build mock providers with custom LLM mock.
pub fn build_mock_providers_with_llm(mock_llm: MockLlmProvider) -> (Arc<LlmManager>, Arc<SearchManager>) {
    let llm = Arc::new(LlmManager::with_provider(
        Arc::new(mock_llm),
        test_pipeline_config().llm,
    ));

    let mock_search = Arc::new(MockSearchProvider::new());
    let search = Arc::new(SearchManager::with_providers(
        mock_search,
        None,
        test_pipeline_config().search,
    ));

    (llm, search)
}

/// Build mock providers with custom search mock.
pub fn build_mock_providers_with_search(mock_search: MockSearchProvider) -> (Arc<LlmManager>, Arc<SearchManager>) {
    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(
        mock_llm,
        test_pipeline_config().llm,
    ));

    let search = Arc::new(SearchManager::with_providers(
        Arc::new(mock_search),
        None,
        test_pipeline_config().search,
    ));

    (llm, search)
}

/// Create an HttpFetcher that bypasses system proxy settings.
/// Required for tests using local wiremock servers when a system proxy is active.
pub fn test_fetcher(config: &PipelineConfig) -> Arc<HttpFetcher> {
    let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(config.rate_limit_ms));
    Arc::new(HttpFetcher::new_no_proxy(rate_limiter))
}
