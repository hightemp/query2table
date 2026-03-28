use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

use crate::providers::llm::manager::{LlmManager, LlmConfig};
use crate::providers::search::manager::{SearchManager, SearchConfig};
use crate::providers::http::client::HttpFetcher;
use crate::providers::http::rate_limiter::RateLimiter;
use crate::storage::models::SchemaColumn;
use crate::storage::repository::Repository;

use crate::roles::query_interpreter::QueryInterpreter;
use crate::roles::schema_planner::SchemaPlanner;
use crate::roles::search_planner::SearchPlanner;
use crate::roles::query_expander::QueryExpander;
use crate::roles::search_executor::SearchExecutor;
use crate::roles::extractor::ExtractedRow;
use crate::roles::validator::Validator;
use crate::roles::deduplicator::Deduplicator;
use crate::roles::stopping_controller::{StoppingController, StopConfig, PipelineStats};

use super::budget_tracker::BudgetTracker;
use super::events::{EventPublisher, ProgressStats};
use super::fetch_pool::{self, FetchJob, FetchResult};
use super::extract_pool::{self, ExtractionJob, ExtractResult};

/// Pipeline configuration derived from settings.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub llm: LlmConfig,
    pub search: SearchConfig,
    pub stop: StopConfig,
    pub max_parallel_fetches: usize,
    pub max_parallel_extractions: usize,
    pub min_confidence: f64,
    pub dedup_similarity: f64,
    pub max_budget_usd: f64,
    pub rate_limit_ms: u64,
}

impl PipelineConfig {
    pub fn from_settings(settings: &HashMap<String, String>) -> Self {
        Self {
            llm: LlmManager::config_from_settings(settings),
            search: SearchManager::config_from_settings(settings),
            stop: StoppingController::config_from_settings(settings),
            max_parallel_fetches: settings.get("max_parallel_fetches")
                .and_then(|v| v.parse().ok())
                .unwrap_or(8),
            max_parallel_extractions: settings.get("max_parallel_extractions")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            min_confidence: settings.get("min_confidence_threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.5),
            dedup_similarity: settings.get("dedup_similarity_threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.85),
            max_budget_usd: settings.get("max_budget_usd")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            rate_limit_ms: settings.get("rate_limit_per_domain_ms")
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000),
        }
    }
}

/// Commands that can be sent to a running pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineCommand {
    Pause,
    Resume,
    Cancel,
    ConfirmSchema(Vec<SchemaColumn>),
}

/// Current state of the pipeline.
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineState {
    Pending,
    Interpreting,
    Planning,
    SchemaReview,
    Running,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

impl PipelineState {
    pub fn as_status_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Interpreting | Self::Planning => "running",
            Self::SchemaReview => "schema_review",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed => "completed",
            Self::Failed(_) => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// The main pipeline orchestrator.
pub struct Pipeline {
    run_id: String,
    query: String,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    cmd_rx: mpsc::Receiver<PipelineCommand>,
    state: PipelineState,
    budget: BudgetTracker,
    start_time: Instant,
    /// Pre-built LLM manager to use instead of creating from config.
    llm_override: Option<Arc<LlmManager>>,
    /// Pre-built Search manager to use instead of creating from config.
    search_override: Option<Arc<SearchManager>>,
    /// Pre-built HTTP fetcher to use instead of creating a default one.
    fetcher_override: Option<Arc<HttpFetcher>>,
}

impl Pipeline {
    /// Create a new pipeline. Returns the pipeline and a command sender for external control.
    pub fn new(
        run_id: String,
        query: String,
        config: PipelineConfig,
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
    ) -> (Self, mpsc::Sender<PipelineCommand>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        let budget = BudgetTracker::new(config.max_budget_usd);

        let pipeline = Self {
            run_id,
            query,
            config,
            repo,
            events,
            cmd_rx,
            state: PipelineState::Pending,
            budget,
            start_time: Instant::now(),
            llm_override: None,
            search_override: None,
            fetcher_override: None,
        };

        (pipeline, cmd_tx)
    }

    /// Set pre-built providers to avoid creating real ones from config.
    pub fn set_providers(&mut self, llm: Arc<LlmManager>, search: Arc<SearchManager>) {
        self.llm_override = Some(llm);
        self.search_override = Some(search);
    }

    /// Set a pre-built HTTP fetcher (e.g. one configured with no_proxy for tests).
    pub fn set_fetcher(&mut self, fetcher: Arc<HttpFetcher>) {
        self.fetcher_override = Some(fetcher);
    }

    /// Run the full pipeline to completion.
    pub async fn run(mut self) -> Result<PipelineState, PipelineError> {
        info!(run_id = %self.run_id, query = %self.query, "Pipeline started");

        // Create the run in DB
        let config_json = serde_json::json!({
            "max_parallel_fetches": self.config.max_parallel_fetches,
            "max_parallel_extractions": self.config.max_parallel_extractions,
            "min_confidence": self.config.min_confidence,
            "dedup_similarity": self.config.dedup_similarity,
        });
        self.repo.create_run(&self.run_id, &self.query, &config_json.to_string()).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        // Initialize providers (use overrides if set, otherwise create from config)
        let llm = if let Some(llm) = self.llm_override.take() {
            llm
        } else {
            Arc::new(
                LlmManager::from_config(self.config.llm.clone())
                    .map_err(|e| PipelineError::Config(format!("LLM: {e}")))?
            )
        };
        let search = if let Some(search) = self.search_override.take() {
            search
        } else {
            Arc::new(
                SearchManager::from_config(self.config.search.clone())
                    .map_err(|e| PipelineError::Config(format!("Search: {e}")))?
            )
        };

        // --- Phase 1: Interpret query ---
        self.set_state(PipelineState::Interpreting).await;
        self.log("INFO", "interpreter", "Analyzing query with LLM...").await;

        let intent = QueryInterpreter::interpret(&self.query, &llm).await
            .map_err(|e| PipelineError::Llm(format!("Interpreter: {e}")))?;
        self.budget.record_llm_call(500, 200); // estimated tokens

        self.log("INFO", "interpreter", &format!("Identified entity type: '{}'", intent.entity_type)).await;
        self.log("INFO", "interpreter", &format!("Attributes: {:?}", intent.attributes)).await;
        if !intent.constraints.is_empty() {
            self.log("INFO", "interpreter", &format!("Constraints: {:?}", intent.constraints)).await;
        }
        if !intent.languages.is_empty() {
            self.log("INFO", "interpreter", &format!("Languages: {:?}", intent.languages)).await;
        }

        // --- Phase 2: Plan schema ---
        self.set_state(PipelineState::Planning).await;
        self.log("INFO", "planner", "Generating table schema with LLM...").await;

        let proposed_schema = SchemaPlanner::plan(&intent, &llm).await
            .map_err(|e| PipelineError::Llm(format!("SchemaPlanner: {e}")))?;
        self.budget.record_llm_call(500, 300);

        // Save proposed schema
        let columns_json = serde_json::to_string(&proposed_schema.columns)
            .map_err(|e| PipelineError::Internal(e.to_string()))?;
        self.repo.create_run_schema(&self.run_id, &columns_json).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        let col_names: Vec<&str> = proposed_schema.columns.iter().map(|c| c.name.as_str()).collect();
        self.log("INFO", "planner", &format!("Proposed {} columns: {}", proposed_schema.columns.len(), col_names.join(", "))).await;

        // --- Phase 3: Schema review (wait for confirmation or auto-confirm) ---
        // Emit schema_proposed BEFORE status change so the frontend has columns
        // when SchemaEditor mounts (status_changed triggers the mount).
        if let Some(ref events) = self.events {
            let columns_val = serde_json::to_value(&proposed_schema.columns)
                .unwrap_or(serde_json::Value::Array(vec![]));
            events.emit_schema_proposed(&columns_val);
        }
        self.set_state(PipelineState::SchemaReview).await;

        // Wait for schema confirmation or timeout (auto-confirm after checking for command)
        let confirmed_columns = self.wait_for_schema_confirmation(proposed_schema.columns).await?;

        // Update schema in DB with confirmed columns
        let confirmed_json = serde_json::to_string(&confirmed_columns)
            .map_err(|e| PipelineError::Internal(e.to_string()))?;
        self.repo.update_run_schema_columns(&self.run_id, &confirmed_json).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;
        self.repo.confirm_run_schema(&self.run_id).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        // --- Phase 4: Search planning ---
        self.set_state(PipelineState::Running).await;
        self.log("INFO", "search_planner", "Generating search queries with LLM...").await;

        let search_plan = SearchPlanner::plan(&intent, &crate::roles::schema_planner::ProposedSchema { columns: confirmed_columns.clone() }, &llm).await
            .map_err(|e| PipelineError::Llm(format!("SearchPlanner: {e}")))?;
        self.budget.record_llm_call(500, 300);

        // Expand queries
        let languages = if intent.languages.is_empty() {
            vec!["en".to_string()]
        } else {
            intent.languages.clone()
        };
        let expanded = QueryExpander::expand(&search_plan.queries, &languages, &llm).await
            .map_err(|e| PipelineError::Llm(format!("QueryExpander: {e}")))?;
        self.budget.record_llm_call(300, 400);

        let all_queries = [search_plan.queries.as_slice(), expanded.queries.as_slice()].concat();
        self.log("INFO", "search_planner", &format!("{} search queries planned across {} languages", all_queries.len(), languages.len())).await;
        for (i, q) in all_queries.iter().enumerate() {
            self.log("DEBUG", "search_planner", &format!("  Query {}: [{}] {}", i + 1, q.language, q.query_text)).await;
        }

        // Save search queries to DB
        for (i, q) in all_queries.iter().enumerate() {
            self.repo.create_search_query(
                &self.run_id,
                &q.query_text,
                &q.language,
                q.geo_target.as_deref(),
                search.primary_name(),
                i as i64 / 5, // batch grouping
            ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;
        }

        // --- Phase 5: Execute searches ---
        self.log("INFO", "search_executor", &format!("Executing {} search queries via {}...", all_queries.len(), search.primary_name())).await;

        let collected = SearchExecutor::execute(&all_queries, &search).await
            .map_err(|e| PipelineError::Search(format!("SearchExecutor: {e}")))?;
        for _ in 0..collected.total_queries_executed {
            self.budget.record_search_call();
        }

        self.log("INFO", "search_executor", &format!(
            "Found {} URLs from {} queries ({} failed)",
            collected.results.len(),
            collected.total_queries_executed,
            collected.failed_queries,
        )).await;

        // Save search results to DB
        let search_queries = self.repo.get_search_queries_by_run(&self.run_id).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;
        let first_sq_id = search_queries.first().map(|sq| sq.id.clone()).unwrap_or_default();

        for (rank, sr) in collected.results.iter().enumerate() {
            self.repo.create_search_result(
                &first_sq_id,
                &self.run_id,
                &sr.result.url,
                &sr.result.title,
                &sr.result.snippet,
                rank as i64,
            ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;
        }

        // --- Phase 6: Fetch + Extract loop ---
        let pending_results = self.repo.get_pending_search_results(&self.run_id).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        let total_pages = pending_results.len();
        self.log("INFO", "fetcher", &format!("Fetching {} pages (max {} parallel)...", total_pages, self.config.max_parallel_fetches)).await;
        let fetcher = if let Some(f) = self.fetcher_override.take() {
            f
        } else {
            let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(self.config.rate_limit_ms));
            Arc::new(HttpFetcher::new(rate_limiter))
        };

        // Spawn worker pools
        let (fetch_tx, mut fetch_rx) = fetch_pool::spawn_fetch_pool(
            fetcher,
            self.config.max_parallel_fetches,
        );
        let (extract_tx, mut extract_rx) = extract_pool::spawn_extract_pool(
            llm.clone(),
            confirmed_columns.clone(),
            self.config.max_parallel_extractions,
        );

        // Submit fetch jobs in a background task to avoid deadlock:
        // If we submit all jobs synchronously before reading results, the result
        // channel can fill up, blocking workers, which blocks job submission.
        tokio::spawn(async move {
            for sr in pending_results {
                let job = FetchJob {
                    search_result_id: sr.id.clone(),
                    url: sr.url.clone(),
                    title: sr.title.clone().unwrap_or_default(),
                };
                if fetch_tx.send(job).await.is_err() {
                    break;
                }
            }
            // Drop the sender to signal no more jobs
            drop(fetch_tx);
        });

        // Process fetched pages → extraction → validation → dedup
        let mut all_valid_rows: Vec<ExtractedRow> = Vec::new();
        let mut pages_fetched: u64 = 0;
        let mut pages_failed: u64 = 0;
        let mut fetch_done = false;
        let mut extract_pending: u64 = 0;
        let mut extract_tx = Some(extract_tx);
        let mut last_batch_new_rows: usize = 0;
        let mut last_batch_total_rows: usize = 0;

        loop {
            // Check for commands (non-blocking)
            if let Ok(cmd) = self.cmd_rx.try_recv() {
                match cmd {
                    PipelineCommand::Cancel => {
                        self.set_state(PipelineState::Cancelled).await;
                        return Ok(PipelineState::Cancelled);
                    }
                    PipelineCommand::Pause => {
                        self.set_state(PipelineState::Paused).await;
                        // Wait for resume or cancel
                        loop {
                            if let Some(cmd) = self.cmd_rx.recv().await {
                                match cmd {
                                    PipelineCommand::Resume => {
                                        self.set_state(PipelineState::Running).await;
                                        break;
                                    }
                                    PipelineCommand::Cancel => {
                                        self.set_state(PipelineState::Cancelled).await;
                                        return Ok(PipelineState::Cancelled);
                                    }
                                    _ => {}
                                }
                            } else {
                                return Ok(PipelineState::Cancelled);
                            }
                        }
                    }
                    _ => {}
                }
            }

            // Check stop conditions
            let stats = PipelineStats {
                row_count: all_valid_rows.len(),
                estimated_cost_usd: self.budget.spent_usd(),
                start_time: self.start_time,
                last_batch_new_rows,
                last_batch_total_rows,
            };
            if let Some(reason) = StoppingController::should_stop(&self.config.stop, &stats) {
                self.log("INFO", "stopping_controller", &format!("Stopping: {:?}", reason)).await;
                break;
            }

            // Try to receive fetch results (with timeout to keep UI responsive)
            tokio::select! {
                fetch_result = fetch_rx.recv(), if !fetch_done => {
                    match fetch_result {
                        Some(FetchResult::Success(doc)) => {
                            self.budget.record_fetch_call();
                            pages_fetched += 1;

                            self.log("INFO", "fetcher", &format!("[{}/{}] Fetched: {}", pages_fetched, total_pages, &doc.document.url)).await;

                            // Save fetched page
                            let page_id = self.repo.create_fetched_page(
                                &doc.search_result_id,
                                &self.run_id,
                                &doc.document.url,
                                "success",
                                Some(&doc.document.text),
                                Some(doc.content_length as i64),
                                Some(doc.fetch_duration_ms as i64),
                                Some(doc.http_status as i64),
                            ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;

                            // Update search result status
                            self.repo.update_search_result_status(&doc.search_result_id, "fetched").await
                                .map_err(|e| PipelineError::Storage(e.to_string()))?;

                            // Only extract if we have meaningful content
                            if doc.document.text.len() > 50 {
                                if let Some(ref tx) = extract_tx {
                                    let extraction_job = ExtractionJob {
                                        fetched_page_id: page_id,
                                        document: doc.document,
                                    };
                                    if tx.send(extraction_job).await.is_ok() {
                                        extract_pending += 1;
                                    }
                                }
                            }

                            self.emit_progress(pages_fetched, total_pages as u64, all_valid_rows.len() as u64, all_queries.len() as u64);
                        }
                        Some(FetchResult::Failure(fail)) => {
                            pages_failed += 1;
                            self.log("WARN", "fetcher", &format!("Failed to fetch: {}", &fail.url)).await;
                            self.repo.create_fetched_page(
                                &fail.search_result_id,
                                &self.run_id,
                                &fail.url,
                                "failed",
                                None, None, None, None,
                            ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;
                            self.repo.update_search_result_status(&fail.search_result_id, "failed").await
                                .map_err(|e| PipelineError::Storage(e.to_string()))?;
                        }
                        None => {
                            fetch_done = true;
                            // Signal no more extraction jobs
                            extract_tx.take();
                        }
                    }
                }
                extract_result = extract_rx.recv() => {
                    match extract_result {
                        Some(ExtractResult::Success(output)) => {
                            extract_pending = extract_pending.saturating_sub(1);
                            self.budget.record_llm_call(2000, 500); // rough estimate per extraction

                            // Validate extracted rows
                            let validated = Validator::validate(&output.rows, &confirmed_columns, self.config.min_confidence);
                            let valid_rows = Validator::filter_valid(&validated);

                            if !valid_rows.is_empty() {
                                self.log("INFO", "extractor", &format!("Extracted {} valid rows (of {})", valid_rows.len(), output.rows.len())).await;
                            }

                            for row in &valid_rows {
                                // Save entity row
                                let data_str = row.data.to_string();
                                let row_id = self.repo.create_entity_row(
                                    &self.run_id,
                                    &data_str,
                                    row.confidence,
                                    "validated",
                                ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;

                                // Save row source
                                self.repo.create_row_source(
                                    &row_id,
                                    &row.source_url,
                                    Some(&row.source_title),
                                    None,
                                    Some(&output.fetched_page_id),
                                ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;

                                // Emit event
                                if let Some(ref events) = self.events {
                                    events.emit_row_added(&row_id, &row.data, row.confidence);
                                }
                            }

                            // Update saturation tracking
                            let new_count = valid_rows.len();
                            last_batch_new_rows = new_count;
                            last_batch_total_rows = output.rows.len();

                            all_valid_rows.extend(valid_rows);
                            self.emit_progress(pages_fetched, total_pages as u64, all_valid_rows.len() as u64, all_queries.len() as u64);
                        }
                        Some(ExtractResult::Failure(fail)) => {
                            extract_pending = extract_pending.saturating_sub(1);
                            self.log("WARN", "extractor", &format!("Extraction failed for {}: {}", fail.page_url, fail.error)).await;
                        }
                        None => {
                            // All extractions done
                            if fetch_done {
                                break;
                            }
                        }
                    }
                }
                // Periodic heartbeat to keep UI responsive and check stop conditions
                _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
                    self.emit_progress(pages_fetched, total_pages as u64, all_valid_rows.len() as u64, all_queries.len() as u64);
                }
            }
        }

        // --- Phase 7: Deduplication ---
        self.log("INFO", "pipeline", &format!("Fetch complete: {} pages fetched, {} failed", pages_fetched, pages_failed)).await;
        self.log("INFO", "deduplicator", &format!("Deduplicating {} rows (similarity threshold: {:.0}%)...", all_valid_rows.len(), self.config.dedup_similarity * 100.0)).await;

        let dedup_result = Deduplicator::deduplicate(
            &all_valid_rows,
            "name",
            self.config.dedup_similarity,
        );

        // Update dedup groups in DB
        for group in &dedup_result.groups {
            let data_str = group.merged.data.to_string();
            let rows = self.repo.get_entity_rows_by_run(&self.run_id).await
                .map_err(|e| PipelineError::Storage(e.to_string()))?;

            // Find the first matching row to update as the group representative
            if let Some(db_row) = rows.first() {
                self.repo.update_entity_row_dedup(
                    &db_row.id,
                    &group.group_id,
                    &data_str,
                    group.merged.confidence,
                ).await.map_err(|e| PipelineError::Storage(e.to_string()))?;
            }
        }

        self.log("INFO", "deduplicator", &format!(
            "{} unique entities, {} duplicates merged",
            dedup_result.unique_entities,
            dedup_result.duplicates_merged,
        )).await;

        // --- Phase 8: Finalize ---
        let final_count = dedup_result.unique_entities;
        let snapshot = self.budget.snapshot();
        let stats_json = serde_json::json!({
            "rows_found": final_count,
            "pages_fetched": pages_fetched,
            "pages_failed": pages_failed,
            "queries_executed": collected.total_queries_executed,
            "queries_failed": collected.failed_queries,
            "duplicates_merged": dedup_result.duplicates_merged,
            "spent_usd": snapshot.spent_usd,
            "llm_calls": snapshot.llm_calls,
            "search_calls": snapshot.search_calls,
            "elapsed_secs": self.start_time.elapsed().as_secs(),
        });

        self.repo.update_run_stats(&self.run_id, &stats_json.to_string()).await
            .map_err(|e| PipelineError::Storage(e.to_string()))?;

        self.set_state(PipelineState::Completed).await;
        self.log("INFO", "pipeline", &format!(
            "Pipeline completed: {} entities found in {}s",
            final_count,
            self.start_time.elapsed().as_secs(),
        )).await;

        Ok(PipelineState::Completed)
    }

    async fn wait_for_schema_confirmation(
        &mut self,
        proposed: Vec<SchemaColumn>,
    ) -> Result<Vec<SchemaColumn>, PipelineError> {
        // Check command channel for schema confirmation
        // If no frontend is connected (events is None), auto-confirm
        if self.events.is_none() {
            return Ok(proposed);
        }

        // Wait for confirmation command with a timeout
        loop {
            tokio::select! {
                cmd = self.cmd_rx.recv() => {
                    match cmd {
                        Some(PipelineCommand::ConfirmSchema(columns)) => {
                            self.log("INFO", "pipeline", "Schema confirmed by user").await;
                            return Ok(columns);
                        }
                        Some(PipelineCommand::Cancel) => {
                            self.set_state(PipelineState::Cancelled).await;
                            return Err(PipelineError::Cancelled);
                        }
                        Some(_) => continue,
                        None => {
                            // Channel closed, auto-confirm
                            return Ok(proposed);
                        }
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(300)) => {
                    // Auto-confirm after 5 minutes
                    self.log("INFO", "pipeline", "Schema auto-confirmed (timeout)").await;
                    return Ok(proposed);
                }
            }
        }
    }

    async fn set_state(&mut self, state: PipelineState) {
        let status_str = state.as_status_str().to_string();
        self.state = state;

        // Update DB
        if let Err(e) = self.repo.update_run_status(&self.run_id, &status_str).await {
            error!(error = %e, "Failed to update run status in DB");
        }

        // Emit event
        if let Some(ref events) = self.events {
            events.emit_status_changed(&status_str);
        }

        debug!(run_id = %self.run_id, status = %status_str, "Pipeline state changed");
    }

    async fn log(&self, level: &str, role: &str, message: &str) {
        if let Err(e) = self.repo.create_run_log(&self.run_id, level, Some(role), message, None).await {
            error!(error = %e, "Failed to write run log");
        }
        if let Some(ref events) = self.events {
            events.emit_log(level, role, message);
        }
        match level {
            "ERROR" => error!(run_id = %self.run_id, role, "{}", message),
            "WARN" => warn!(run_id = %self.run_id, role, "{}", message),
            _ => info!(run_id = %self.run_id, role, "{}", message),
        }
    }

    fn emit_progress(&self, pages_fetched: u64, pages_total: u64, rows_found: u64, queries_total: u64) {
        if let Some(ref events) = self.events {
            events.emit_progress(ProgressStats {
                rows_found,
                pages_fetched,
                pages_total,
                queries_executed: queries_total,
                queries_total,
                elapsed_secs: self.start_time.elapsed().as_secs(),
                spent_usd: self.budget.spent_usd(),
            });
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Search error: {0}")]
    Search(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Pipeline cancelled")]
    Cancelled,
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_state_as_status_str() {
        assert_eq!(PipelineState::Pending.as_status_str(), "pending");
        assert_eq!(PipelineState::Interpreting.as_status_str(), "running");
        assert_eq!(PipelineState::Planning.as_status_str(), "running");
        assert_eq!(PipelineState::SchemaReview.as_status_str(), "schema_review");
        assert_eq!(PipelineState::Running.as_status_str(), "running");
        assert_eq!(PipelineState::Paused.as_status_str(), "paused");
        assert_eq!(PipelineState::Completed.as_status_str(), "completed");
        assert_eq!(PipelineState::Failed("err".into()).as_status_str(), "failed");
        assert_eq!(PipelineState::Cancelled.as_status_str(), "cancelled");
    }

    #[test]
    fn test_pipeline_config_from_settings() {
        let mut settings = HashMap::new();
        settings.insert("max_parallel_fetches".to_string(), "4".to_string());
        settings.insert("max_parallel_extractions".to_string(), "2".to_string());
        settings.insert("min_confidence_threshold".to_string(), "0.7".to_string());
        settings.insert("dedup_similarity_threshold".to_string(), "0.9".to_string());
        settings.insert("max_budget_usd".to_string(), "5.0".to_string());
        settings.insert("rate_limit_per_domain_ms".to_string(), "1000".to_string());

        let config = PipelineConfig::from_settings(&settings);
        assert_eq!(config.max_parallel_fetches, 4);
        assert_eq!(config.max_parallel_extractions, 2);
        assert_eq!(config.min_confidence, 0.7);
        assert_eq!(config.dedup_similarity, 0.9);
        assert_eq!(config.max_budget_usd, 5.0);
        assert_eq!(config.rate_limit_ms, 1000);
    }

    #[test]
    fn test_pipeline_config_defaults() {
        let settings = HashMap::new();
        let config = PipelineConfig::from_settings(&settings);
        assert_eq!(config.max_parallel_fetches, 8);
        assert_eq!(config.max_parallel_extractions, 3);
        assert_eq!(config.min_confidence, 0.5);
        assert_eq!(config.dedup_similarity, 0.85);
        assert_eq!(config.max_budget_usd, 1.0);
        assert_eq!(config.rate_limit_ms, 2000);
    }

    #[test]
    fn test_pipeline_command_eq() {
        assert_eq!(PipelineCommand::Pause, PipelineCommand::Pause);
        assert_eq!(PipelineCommand::Resume, PipelineCommand::Resume);
        assert_eq!(PipelineCommand::Cancel, PipelineCommand::Cancel);
        assert_ne!(PipelineCommand::Pause, PipelineCommand::Cancel);
    }

    #[test]
    fn test_pipeline_error_display() {
        let e = PipelineError::Llm("test error".into());
        assert_eq!(e.to_string(), "LLM error: test error");

        let e = PipelineError::Cancelled;
        assert_eq!(e.to_string(), "Pipeline cancelled");
    }
}
