use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::providers::http::client::HttpFetcher;
use crate::providers::http::rate_limiter::RateLimiter;
use crate::providers::llm::manager::LlmManager;
use crate::providers::search::manager::SearchManager;
use crate::storage::repository::Repository;

use crate::roles::link_ranker::{LinkRanker, PageCandidate};
use crate::roles::search_executor::SearchExecutor;
use crate::roles::search_planner::PlannedSearch;
use crate::roles::stopping_controller::{PipelineStats, StoppingController};

use super::budget_tracker::BudgetTracker;
use super::events::{EventPublisher, ProgressStats};
use super::fetch_pool::{self, FetchJob, FetchResult};
use super::pipeline::{PipelineCommand, PipelineConfig, PipelineState};

/// Simplified pipeline for relevance-filtered link search mode.
/// Flow: Generate Queries → Web Search → Fetch & Parse → LLM Relevance Score → Filter → Store
pub struct LinkPipeline {
    run_id: String,
    query: String,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    cmd_rx: mpsc::Receiver<PipelineCommand>,
    budget: BudgetTracker,
    start_time: Instant,
}

impl LinkPipeline {
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
            budget,
            start_time: Instant::now(),
        };

        (pipeline, cmd_tx)
    }

    pub async fn run(mut self) -> Result<PipelineState, String> {
        info!(run_id = %self.run_id, query = %self.query, "Link pipeline started");

        // Create run in DB
        let config_json = serde_json::json!({ "mode": "links" });
        self.repo
            .create_run_with_type(&self.run_id, &self.query, &config_json.to_string(), "links")
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        self.set_status("running").await;
        self.log("INFO", "link_pipeline", "Starting link search...").await;

        // Initialize providers
        let search = Arc::new(
            SearchManager::from_config(self.config.search.clone())
                .map_err(|e| format!("Search config: {e}"))?,
        );
        let llm = LlmManager::from_config(self.config.llm.clone()).ok();

        // Generate search query variations (LLM-based or static fallback)
        let queries = match &llm {
            Some(llm_mgr) => {
                self.log("INFO", "link_pipeline", "Generating search queries with LLM...").await;
                match Self::generate_queries_with_llm(&self.query, llm_mgr).await {
                    Ok(q) => {
                        self.budget.record_llm_call(500, 300);
                        q
                    }
                    Err(e) => {
                        warn!(error = %e, "LLM query generation failed, using static fallback");
                        self.log("WARN", "link_pipeline", &format!("LLM query generation failed: {e}, using static fallback")).await;
                        Self::generate_query_variations(&self.query)
                    }
                }
            }
            None => {
                self.log("INFO", "link_pipeline", "LLM not configured, using static query variations").await;
                Self::generate_query_variations(&self.query)
            }
        };
        self.log("INFO", "search_executor", &format!("Searching with {} query variations", queries.len())).await;

        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Execute web searches (deduplicated by URL)
        let planned: Vec<PlannedSearch> = queries
            .iter()
            .enumerate()
            .map(|(i, q)| PlannedSearch {
                query_text: q.clone(),
                language: "en".to_string(),
                geo_target: None,
                priority: (i + 1) as u8,
            })
            .collect();

        let collected = SearchExecutor::execute(&planned, &search)
            .await
            .map_err(|e| format!("Search: {e}"))?;

        for _ in 0..collected.total_queries_executed {
            self.budget.record_search_call();
        }

        self.log("INFO", "search_executor", &format!(
            "Found {} unique URLs from {} queries ({} failed)",
            collected.results.len(),
            collected.total_queries_executed,
            collected.failed_queries,
        )).await;

        if collected.results.is_empty() {
            self.log("WARN", "link_pipeline", "No search results found").await;
            self.set_status("completed").await;
            return Ok(PipelineState::Completed);
        }

        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Fetch & parse each page
        let total_pages = collected.results.len();
        self.log("INFO", "fetcher", &format!("Fetching {} pages (max {} parallel)...", total_pages, self.config.max_parallel_fetches)).await;

        let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(self.config.rate_limit_ms));
        let fetcher = Arc::new(
            HttpFetcher::new(rate_limiter).with_max_body_bytes(self.config.max_page_size_bytes),
        );
        let max_pdf_chars = if self.config.enable_content_truncation {
            Some(self.config.max_pdf_text_chars)
        } else {
            None
        };

        let (fetch_tx, mut fetch_rx) =
            fetch_pool::spawn_fetch_pool(fetcher, self.config.max_parallel_fetches, max_pdf_chars);

        // Map search_result index -> (url, title, snippet) for joining fetched docs
        let mut meta: std::collections::HashMap<String, (String, String, String)> =
            std::collections::HashMap::new();
        for (idx, sr) in collected.results.iter().enumerate() {
            let key = idx.to_string();
            meta.insert(
                key.clone(),
                (sr.result.url.clone(), sr.result.title.clone(), sr.result.snippet.clone()),
            );
        }

        let jobs: Vec<FetchJob> = collected
            .results
            .iter()
            .enumerate()
            .map(|(idx, sr)| FetchJob {
                search_result_id: idx.to_string(),
                url: sr.result.url.clone(),
                title: sr.result.title.clone(),
            })
            .collect();

        tokio::spawn(async move {
            for job in jobs {
                if fetch_tx.send(job).await.is_err() {
                    break;
                }
            }
            drop(fetch_tx);
        });

        let mut candidates: Vec<PageCandidate> = Vec::new();
        let mut pages_fetched: u64 = 0;
        let mut pages_failed: u64 = 0;

        while let Some(result) = fetch_rx.recv().await {
            if self.is_cancelled() {
                self.set_status("cancelled").await;
                return Ok(PipelineState::Cancelled);
            }
            match result {
                FetchResult::Success(doc) => {
                    pages_fetched += 1;
                    let (url, title, snippet) = meta
                        .get(&doc.search_result_id)
                        .cloned()
                        .unwrap_or_else(|| (doc.document.url.clone(), doc.document.title.clone(), String::new()));
                    candidates.push(PageCandidate {
                        url,
                        title,
                        snippet,
                        document: doc.document,
                    });
                }
                FetchResult::Failure(f) => {
                    pages_failed += 1;
                    warn!(url = %f.url, error = %f.error, "Fetch failed");
                }
            }

            if let Some(ref events) = self.events {
                events.emit_progress(ProgressStats {
                    rows_found: 0,
                    pages_fetched,
                    pages_total: total_pages as u64,
                    queries_executed: queries.len() as u64,
                    queries_total: queries.len() as u64,
                    elapsed_secs: self.start_time.elapsed().as_secs(),
                    spent_usd: self.budget.spent_usd(),
                });
            }
        }

        self.log("INFO", "fetcher", &format!(
            "Fetched {} pages ({} failed)", pages_fetched, pages_failed
        )).await;

        if candidates.is_empty() {
            self.log("WARN", "link_pipeline", "No pages could be fetched").await;
            self.set_status("completed").await;
            return Ok(PipelineState::Completed);
        }

        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Score relevance from page content + generate descriptions
        let max_text_chars = if self.config.enable_content_truncation {
            Some(self.config.max_extraction_text_chars)
        } else {
            None
        };

        let ranked = if let Some(ref llm_mgr) = llm {
            self.log("INFO", "link_ranker", "Scoring page relevance with LLM...").await;
            let min_relevance = self.config.min_confidence;
            let candidate_count = candidates.len();
            match LinkRanker::rank(&self.query, candidates, llm_mgr, min_relevance, max_text_chars).await {
                Ok(r) => {
                    self.budget.record_llm_call(1000 * candidate_count as u32, 150 * candidate_count as u32);
                    self.log("INFO", "link_ranker", &format!("Ranked: {} pages passed relevance filter", r.len())).await;
                    r
                }
                Err(e) => {
                    return Err(format!("Link ranking: {e}"));
                }
            }
        } else {
            self.log("INFO", "link_pipeline", "LLM not configured, keeping all pages with snippet descriptions").await;
            candidates
                .into_iter()
                .map(|c| crate::roles::link_ranker::RankedLink {
                    url: c.url,
                    title: c.title,
                    description: c.snippet,
                    relevance_score: 0.5,
                })
                .collect()
        };

        // Apply target limit
        let max_links = self.config.stop.target_row_count;
        let ranked = if ranked.len() > max_links {
            self.log("INFO", "link_pipeline", &format!("Limiting results from {} to {} (max links)", ranked.len(), max_links)).await;
            ranked.into_iter().take(max_links).collect::<Vec<_>>()
        } else {
            ranked
        };

        // Store results
        self.log("INFO", "link_storage", &format!("Storing {} link results", ranked.len())).await;
        let mut stored_count = 0u64;

        for link in &ranked {
            let link_id = self
                .repo
                .create_link_result(
                    &self.run_id,
                    &link.url,
                    &link.title,
                    &link.description,
                    Some(link.relevance_score),
                )
                .await
                .map_err(|e| format!("Storage: {e}"))?;

            stored_count += 1;

            if let Some(ref events) = self.events {
                events.emit_link_added(
                    &link_id,
                    &link.url,
                    &link.title,
                    &link.description,
                    Some(link.relevance_score),
                );
                events.emit_progress(ProgressStats {
                    rows_found: stored_count,
                    pages_fetched,
                    pages_total: total_pages as u64,
                    queries_executed: queries.len() as u64,
                    queries_total: queries.len() as u64,
                    elapsed_secs: self.start_time.elapsed().as_secs(),
                    spent_usd: self.budget.spent_usd(),
                });
            }

            if let Some(reason) = self.check_stop_conditions(stored_count as usize) {
                self.log("INFO", "stopping_controller", &format!("Stopping during storage: {:?}", reason)).await;
                break;
            }
        }

        // Update run stats
        let stats = serde_json::json!({
            "link_count": stored_count,
            "pages_fetched": pages_fetched,
            "queries_executed": queries.len(),
            "elapsed_secs": self.start_time.elapsed().as_secs(),
            "spent_usd": self.budget.spent_usd(),
        });
        self.repo
            .update_run_stats(&self.run_id, &stats.to_string())
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        self.log("INFO", "link_pipeline", &format!("Link search completed: {} links", stored_count)).await;
        self.set_status("completed").await;

        Ok(PipelineState::Completed)
    }

    /// Generate web search queries using LLM for better diversity and coverage.
    async fn generate_queries_with_llm(query: &str, llm: &LlmManager) -> Result<Vec<String>, String> {
        use crate::providers::llm::Message;

        let system = r#"You are a web search query generator. Given a user's research request, generate 6-10 diverse search queries optimized for finding the most relevant web pages.

Strategy:
1. Include the original query
2. Add variations with different phrasings and synonyms
3. Add queries targeting authoritative or list/directory sources
4. If relevant, add queries in different languages

Respond with valid JSON: {"queries": ["query1", "query2", ...]}. No markdown, no explanation."#;

        let messages = vec![
            Message::system(system),
            Message::user(format!("Generate web search queries for: {}", query)),
        ];

        let response = llm
            .complete(messages, true)
            .await
            .map_err(|e| format!("LLM error: {e}"))?;

        #[derive(serde::Deserialize)]
        struct QueriesResponse {
            queries: Vec<String>,
        }

        let parsed: QueriesResponse = serde_json::from_str(&response.content)
            .map_err(|e| format!("Failed to parse LLM response: {e}"))?;

        if parsed.queries.is_empty() {
            return Err("LLM returned empty query list".to_string());
        }

        let mut queries = parsed.queries;
        let lower_queries: Vec<String> = queries.iter().map(|q| q.to_lowercase()).collect();
        if !lower_queries.contains(&query.to_lowercase()) {
            queries.insert(0, query.to_string());
        }
        queries.truncate(12);

        Ok(queries)
    }

    /// Static fallback: generate query variations without LLM.
    fn generate_query_variations(query: &str) -> Vec<String> {
        let mut queries = vec![query.to_string()];
        queries.push(format!("{} guide", query));
        queries.push(format!("{} overview", query));
        queries.push(format!("best {} resources", query));
        queries.push(format!("{} list", query));
        queries
    }

    fn is_cancelled(&mut self) -> bool {
        if let Ok(cmd) = self.cmd_rx.try_recv() {
            matches!(cmd, PipelineCommand::Cancel)
        } else {
            false
        }
    }

    fn check_stop_conditions(&self, link_count: usize) -> Option<String> {
        let stats = PipelineStats {
            row_count: link_count,
            estimated_cost_usd: self.budget.spent_usd(),
            start_time: self.start_time,
            last_batch_new_rows: 0,
            last_batch_total_rows: 0,
        };
        StoppingController::should_stop(&self.config.stop, &stats).map(|reason| format!("{:?}", reason))
    }

    async fn set_status(&self, status: &str) {
        if let Err(e) = self.repo.update_run_status(&self.run_id, status).await {
            error!(error = %e, "Failed to update run status");
        }
        if let Some(ref events) = self.events {
            events.emit_status_changed(status);
        }
    }

    async fn log(&self, level: &str, role: &str, message: &str) {
        info!(run_id = %self.run_id, role, "{}", message);
        let _ = self.repo.create_run_log(&self.run_id, level, Some(role), message, None).await;
        if let Some(ref events) = self.events {
            events.emit_log(level, role, message);
        }
    }
}
