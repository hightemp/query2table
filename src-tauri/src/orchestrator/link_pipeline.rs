use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::providers::http::client::HttpFetcher;
use crate::providers::http::rate_limiter::RateLimiter;
use crate::providers::llm::manager::LlmManager;
use crate::providers::search::manager::SearchManager;
use crate::storage::repository::Repository;

use crate::roles::link_ranker::{LinkRanker, PageCandidate};
use crate::roles::search_executor::SearchExecutor;
use crate::roles::search_planner::PlannedSearch;

use super::control::{RunControl, RunSupervisor};
use super::budget_tracker::BudgetTracker;
use super::events::{EventPublisher, ProgressStats};
use super::fetch_pool::{self, FetchJob, FetchResult};
use super::pipeline::{PipelineCommand, PipelineConfig, PipelineState};
use super::search_log::SearchLog;

/// Characters of the files' overview given with each page to rank.
const FILES_NOTE_CHARS: usize = 1200;

/// Simplified pipeline for relevance-filtered link search mode.
/// Flow: Generate Queries → Web Search → Fetch & Parse → LLM Relevance Score → Filter → Store
pub struct LinkPipeline {
    run_id: String,
    query: String,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    control: RunControl,
    supervisor: Option<RunSupervisor>,
    budget: BudgetTracker,
    start_time: Instant,
    /// Search provider set by tests instead of the configured one.
    search_override: Option<Arc<SearchManager>>,
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
        let (control, supervisor) =
            RunControl::new(run_id.clone(), repo.clone(), events.clone(), cmd_rx);
        let supervisor = supervisor.with_budget(budget.clone());

        let pipeline = Self {
            run_id,
            query,
            config,
            repo,
            events,
            control,
            supervisor: Some(supervisor),
            budget,
            start_time: Instant::now(),
            search_override: None,
        };

        (pipeline, cmd_tx)
    }

    /// Uses this search provider instead of the configured one (tests).
    pub fn set_search(&mut self, search: Arc<SearchManager>) {
        self.search_override = Some(search);
    }

    pub async fn run(mut self) -> Result<PipelineState, String> {
        info!(run_id = %self.run_id, query = %self.query, "Link pipeline started");

        // Create run in DB
        let config_json = self.config.run_config("links");
        self.repo
            .create_run_with_type(&self.run_id, &self.query, &config_json.to_string(), "links")
            .await
            .map_err(|e| format!("Storage: {e}"))?;
        self.repo
            .link_attachments(&self.run_id, &self.config.attachments, 0)
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        let supervisor = self.supervisor.take().ok_or_else(|| "Pipeline already started".to_string())?;
        supervisor.run(self.run_inner()).await
    }

    async fn run_inner(mut self) -> Result<PipelineState, String> {
        self.set_status("running").await;
        self.log("INFO", "link_pipeline", "Starting link search...").await;

        // Initialize providers
        let search = match self.search_override.take() {
            Some(search) => Arc::new(search.as_ref().clone().with_accounting(self.budget.observer())),
            None => Arc::new(
                SearchManager::from_config(self.config.search.clone())
                    .map_err(|e| format!("Search config: {e}"))?
                    .with_accounting(self.budget.observer()),
            ),
        };
        let llm = LlmManager::from_config_with_diagnostics(
            self.config.llm.clone(),
            self.control.issue_sender(),
        )
        .map(|manager| manager.with_accounting(self.budget.observer()))
        .ok();

        // Attached files shape the searches and tell the ranking what the user means.
        let files = match &llm {
            Some(llm_mgr) => {
                let (files, logs) = super::files::load_for_run(
                    self.repo.pool().clone(),
                    &self.config,
                    &self.run_id,
                    llm_mgr,
                    super::files::Prepare::AsText,
                )
                .await;
                for (level, message) in logs {
                    self.log(level, "files", &message).await;
                }
                files
            }
            None => None,
        };
        let files_context = match &files {
            Some(files) => Some(files.context(&self.query, self.config.context.file_chars / 2).await),
            None => None,
        };
        // The ranker judges pages against the request together with what the files are about.
        let ranking_query = match &files {
            Some(files) => format!("{}\n\n{}", self.query, files.summary(FILES_NOTE_CHARS).await),
            None => self.query.clone(),
        };

        // Generate search query variations (LLM-based or static fallback)
        self.stage("plan");
        let queries = match &llm {
            Some(llm_mgr) => {
                self.log("INFO", "link_pipeline", "Generating search queries with LLM...").await;
                match Self::generate_queries_with_llm(&self.query, files_context.as_deref(), llm_mgr).await {
                    Ok(q) => q,
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

        let listed: Vec<(String, String)> = queries.iter().map(|q| (q.clone(), String::new())).collect();
        let mut search_log = SearchLog::plan(self.repo.clone(), self.events.clone(), &self.run_id, &listed, search.primary_name())
            .await
            .map_err(|e| format!("Storage: {e}"))?;
        self.stage("search");
        let (progress, reports) = tokio::sync::mpsc::unbounded_channel();
        let max_per_query = self.config.max_pages_per_query;
        let searching = async {
            let result = SearchExecutor::execute_reporting(&planned, &search, Some(max_per_query), Some(&progress)).await;
            drop(progress);
            result
        };
        let (collected, ()) = tokio::join!(searching, search_log.follow(reports));
        let collected = collected.map_err(|e| format!("Search: {e}"))?;

        self.log(
            "INFO",
            "search_executor",
            &format!(
                "Found {} unique URLs from {} queries ({} failed)",
                collected.results.len(),
                collected.total_queries_executed,
                collected.failed_queries,
            ),
        )
        .await;

        if collected.results.is_empty() {
            self.log("WARN", "link_pipeline", "No search results found").await;
            self.set_status("completed").await;
            return Ok(PipelineState::Completed);
        }

        // Fetch & parse each page
        self.stage("read");
        let total_pages = collected.results.len();
        self.log("INFO", "fetcher", &format!("Fetching {} pages (max {} parallel)...", total_pages, self.config.max_parallel_fetches)).await;

        let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(self.config.rate_limit_ms));
        let fetcher = Arc::new(
            HttpFetcher::new(rate_limiter)
                .with_max_body_bytes(self.config.max_page_size_bytes)
                .with_timeout(std::time::Duration::from_secs(self.config.fetch_timeout_secs)),
        );
        let max_pdf_chars = if self.config.enable_content_truncation {
            Some(self.config.max_pdf_text_chars)
        } else {
            None
        };

        let (fetch_tx, mut fetch_rx) =
            fetch_pool::spawn_fetch_pool(fetcher, self.config.max_parallel_fetches, max_pdf_chars, self.control.pause_signal());

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

        fetch_rx.spawn(self.control.pause_signal(), async move {
            for job in jobs {
                if fetch_tx.send(job).await.is_err() {
                    break;
                }
            }
            drop(fetch_tx);
        });

        // Each page is scored as soon as it is fetched and shown right away, while the
        // remaining pages keep downloading in the pool.
        let max_text_chars = if self.config.enable_content_truncation {
            Some(self.config.max_extraction_text_chars)
        } else {
            None
        };
        let min_relevance = self.config.min_confidence;
        let max_links = self.config.stop.target_row_count as u64;
        if llm.is_some() {
            self.log("INFO", "link_ranker", "Scoring page relevance with LLM as pages arrive...")
                .await;
        } else {
            self.log("INFO", "link_pipeline", "LLM not configured, keeping all pages with snippet descriptions").await;
        }

        let mut pages_fetched: u64 = 0;
        let mut pages_failed: u64 = 0;
        let mut stored_count: u64 = 0;
        let mut relevant_count: u64 = 0;
        let mut stop_reason: Option<&str> = None;

        while let Some(result) = fetch_rx.recv().await {
            let candidate = match result {
                FetchResult::Success(doc) => {
                    pages_fetched += 1;
                    let (url, title, snippet) = meta
                        .get(&doc.search_result_id)
                        .cloned()
                        .unwrap_or_else(|| (doc.document.url.clone(), doc.document.title.clone(), String::new()));
                    Some(PageCandidate { url, title, snippet, document: doc.document })
                }
                FetchResult::Failure(f) => {
                    pages_failed += 1;
                    warn!(url = %f.url, error = %f.error, "Fetch failed");
                    None
                }
            };

            if let Some(candidate) = candidate {
                let link = match llm {
                    Some(ref llm_mgr) if llm_mgr.spending_limit_reached() => {
                        stop_reason = Some("spending limit reached");
                        None
                    }
                    Some(ref llm_mgr) => {
                        match LinkRanker::score(&ranking_query, &candidate, llm_mgr, max_text_chars).await {
                            Ok(link) => Some((link, true)),
                            Err(e) => {
                                warn!(url = %candidate.url, error = %e, "Link relevance scoring failed, skipping page");
                                None
                            }
                        }
                    }
                    None => Some((LinkRanker::unscored(&candidate), false)),
                };
                if let Some((link, scored)) = link {
                    let low_relevance = scored && link.relevance_score < min_relevance;
                    let link_id = self
                        .repo
                        .create_link_result(
                            &self.run_id,
                            &link.url,
                            &link.title,
                            &link.description,
                            &link.reason,
                            scored.then_some(link.relevance_score),
                            low_relevance,
                        )
                        .await
                        .map_err(|e| format!("Storage: {e}"))?;
                    stored_count += 1;
                    if !low_relevance {
                        relevant_count += 1;
                    }
                    if let Some(ref events) = self.events {
                        events.emit_link_added(
                            &link_id,
                            &link.url,
                            &link.title,
                            &link.description,
                            &link.reason,
                            scored.then_some(link.relevance_score),
                            low_relevance,
                        );
                    }
                }
            }

            if let Some(ref events) = self.events {
                // A page counts once it is loaded and scored, or failed to load.
                events.emit_stage_progress("read", (pages_fetched + pages_failed) as usize, total_pages);
                events.emit_progress(ProgressStats {
                    rows_found: relevant_count,
                    pages_fetched,
                    pages_total: total_pages as u64,
                    queries_executed: queries.len() as u64,
                    queries_total: queries.len() as u64,
                    elapsed_secs: self.start_time.elapsed().as_secs(),
                    spent_usd: self.budget.spent_usd(),
                });
            }

            if relevant_count >= max_links {
                stop_reason = Some("target number of links reached");
            }
            if stop_reason.is_some() {
                break;
            }
        }

        self.log("INFO", "fetcher", &format!(
            "Fetched {} pages ({} failed)", pages_fetched, pages_failed
        )).await;
        if let Some(reason) = stop_reason {
            self.log("INFO", "link_pipeline", &format!("Stopped early: {reason}")).await;
        }
        if pages_fetched == 0 {
            self.log("WARN", "link_pipeline", "No pages could be fetched").await;
        }
        self.log(
            "INFO",
            "link_storage",
            &format!("Stored {stored_count} links ({relevant_count} relevant)"),
        )
        .await;

        // Update run stats
        let stats = serde_json::json!({
            "link_count": relevant_count,
            "pages_fetched": pages_fetched,
            "queries_executed": queries.len(),
            "elapsed_secs": self.start_time.elapsed().as_secs(),
            "spent_usd": self.budget.spent_usd(),
        });
        self.repo
            .update_run_stats(&self.run_id, &stats.to_string())
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        self.log("INFO", "link_pipeline", &format!("Link search completed: {} relevant links", relevant_count)).await;
        self.set_status("completed").await;

        Ok(PipelineState::Completed)
    }

    /// Generate web search queries using LLM for better diversity and coverage.
    async fn generate_queries_with_llm(query: &str, files: Option<&str>, llm: &LlmManager) -> Result<Vec<String>, String> {
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
            Message::user(match files {
                Some(files) => format!(
                    "Generate web search queries for: {query}\n\nThe request comes with these attached files; use their topic, \
                     names and terms in the queries when the request refers to them:\n\n{files}"
                ),
                None => format!("Generate web search queries for: {query}"),
            }),
        ];

        let response = llm
            .complete_for_stage("link_search_planner", messages, true)
            .await
            .map_err(|e| format!("LLM error: {e}"))?;

        #[derive(serde::Deserialize)]
        struct QueriesResponse {
            queries: Vec<String>,
        }

        let parsed: QueriesResponse = serde_json::from_str(&response.content)
            .map_err(|e| {
                let message = format!("Failed to parse search queries: {e}");
                llm.report_invalid_response("link_search_planner", &message, &response, crate::providers::llm::IssueOutcome::Fallback);
                message
            })?;

        if parsed.queries.is_empty() {
            llm.report_invalid_response("link_search_planner", "LLM returned an empty query list", &response, crate::providers::llm::IssueOutcome::Fallback);
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

    /// Tells the interface which stage the run is in.
    fn stage(&self, stage: &str) {
        if let Some(events) = &self.events {
            events.emit_stage(stage);
        }
    }

    async fn set_status(&self, status: &str) {
        self.control.set_status(status).await;
    }

    async fn log(&self, level: &str, role: &str, message: &str) {
        info!(run_id = %self.run_id, role, "{}", message);
        let _ = self.repo.create_run_log(&self.run_id, level, Some(role), message, None).await;
        if let Some(ref events) = self.events {
            events.emit_log(level, role, message);
        }
    }
}
