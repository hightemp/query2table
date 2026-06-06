use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::providers::http::client::HttpFetcher;
use crate::providers::http::rate_limiter::RateLimiter;
use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;
use crate::providers::search::manager::SearchManager;
use crate::storage::repository::Repository;

use crate::roles::pdf_parser::PdfParser;
use crate::roles::research_agent::{AgentAction, ResearchAgent};

use super::budget_tracker::BudgetTracker;
use super::events::{EventPublisher, ProgressStats};
use super::pipeline::{PipelineCommand, PipelineConfig, PipelineState};

/// Maximum number of agent tool-call iterations per research run.
const DEFAULT_MAX_STEPS: u32 = 16;
/// Number of search results to feed back to the agent per search.
const SEARCH_RESULTS_PER_QUERY: u32 = 8;
/// Max characters of fetched page markdown to feed back to the agent.
const FETCH_MARKDOWN_CHAR_LIMIT: usize = 8000;

/// Agentic research pipeline.
/// The LLM drives a tool-calling loop (search / fetch / think) and finishes by
/// producing a Markdown answer. Each step is streamed and persisted.
pub struct ResearchPipeline {
    run_id: String,
    query: String,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    cmd_rx: mpsc::Receiver<PipelineCommand>,
    budget: BudgetTracker,
    start_time: Instant,
}

impl ResearchPipeline {
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
        info!(run_id = %self.run_id, query = %self.query, "Research pipeline started");

        let config_json = serde_json::json!({ "mode": "research" });
        self.repo
            .create_run_with_type(&self.run_id, &self.query, &config_json.to_string(), "research")
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        self.set_status("running").await;
        self.log("INFO", "research", "Starting agentic research...").await;

        // LLM is mandatory for research mode.
        let llm = match LlmManager::from_config(self.config.llm.clone()) {
            Ok(m) => m,
            Err(e) => {
                let msg = format!("LLM not configured: {e}");
                self.log("ERROR", "research", &msg).await;
                self.fail(&msg).await;
                return Ok(PipelineState::Failed(msg));
            }
        };

        let search = match SearchManager::from_config(self.config.search.clone()) {
            Ok(s) => Arc::new(s),
            Err(e) => {
                let msg = format!("Search not configured: {e}");
                self.log("ERROR", "research", &msg).await;
                self.fail(&msg).await;
                return Ok(PipelineState::Failed(msg));
            }
        };

        let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(self.config.rate_limit_ms));
        let fetcher = Arc::new(
            HttpFetcher::new(rate_limiter).with_max_body_bytes(self.config.max_page_size_bytes),
        );
        let max_pdf_chars = if self.config.enable_content_truncation {
            Some(self.config.max_pdf_text_chars)
        } else {
            None
        };

        let max_steps = DEFAULT_MAX_STEPS;

        // Build the running conversation transcript.
        let mut messages = vec![
            Message::system(ResearchAgent::system_prompt(max_steps as usize)),
            Message::user(format!("Research request:\n{}", self.query)),
        ];

        let mut step_index: u32 = 0;
        let mut answered = false;

        while step_index < max_steps {
            // Handle pause/resume/cancel.
            if self.handle_commands().await {
                self.set_status("cancelled").await;
                return Ok(PipelineState::Cancelled);
            }

            // Stop on budget/time limits.
            if let Some(reason) = self.check_limits() {
                self.log("INFO", "research", &format!("Stopping: {reason}")).await;
                break;
            }

            let (action, prompt_tokens, completion_tokens) =
                match ResearchAgent::decide_next_step(&llm, messages.clone()).await {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "Agent step failed");
                        self.log("WARN", "research", &format!("Agent step failed: {e}")).await;
                        // Nudge the model to emit valid JSON next time.
                        messages.push(Message::user(
                            "Your previous reply was not a valid tool call. Reply with ONE JSON tool object.".to_string(),
                        ));
                        step_index += 1;
                        continue;
                    }
                };
            self.budget.record_llm_call(prompt_tokens, completion_tokens);

            match action {
                AgentAction::Search { query } => {
                    self.log("INFO", "research", &format!("Search: {query}")).await;
                    self.budget.record_search_call();
                    let observation = match search
                        .search_with_count(&query, SEARCH_RESULTS_PER_QUERY)
                        .await
                    {
                        Ok(results) => format_search_results(&results),
                        Err(e) => {
                            warn!(error = %e, "Search failed");
                            format!("Search failed: {e}")
                        }
                    };
                    self.record_step(step_index, "search", &query, None).await;
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "search", "query": query }).to_string(),
                    ));
                    messages.push(Message::user(format!("Search results:\n{observation}")));
                }
                AgentAction::Fetch { url } => {
                    self.log("INFO", "research", &format!("Fetch: {url}")).await;
                    let observation = match fetcher.fetch(&url).await {
                        Ok(page) => {
                            let markdown = if page.is_pdf() {
                                PdfParser::parse(&page.body_bytes, &url, max_pdf_chars).text
                            } else {
                                ResearchAgent::html_to_markdown(&page.body, &url)
                            };
                            crate::utils::text::truncate_chars(&markdown, FETCH_MARKDOWN_CHAR_LIMIT)
                                .to_string()
                        }
                        Err(e) => {
                            warn!(url = %url, error = %e, "Fetch failed");
                            format!("Failed to fetch page: {e}")
                        }
                    };
                    self.record_step(step_index, "fetch", &observation, Some(&url)).await;
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "fetch", "url": url }).to_string(),
                    ));
                    messages.push(Message::user(format!("Page content ({url}):\n{observation}")));
                }
                AgentAction::Think { thought } => {
                    self.log("INFO", "research", &format!("Think: {thought}")).await;
                    self.record_step(step_index, "think", &thought, None).await;
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "think", "thought": thought }).to_string(),
                    ));
                    messages.push(Message::user("Acknowledged. Continue.".to_string()));
                }
                AgentAction::Answer { markdown } => {
                    self.log("INFO", "research", "Agent produced final answer").await;
                    self.store_answer(&markdown).await;
                    answered = true;
                    break;
                }
            }

            self.emit_progress(step_index + 1, max_steps);
            step_index += 1;
        }

        // If the loop ended without an explicit answer, request one final answer.
        if !answered {
            self.log("INFO", "research", "Step/limit reached, requesting final answer").await;
            messages.push(Message::user(
                "You have reached your step limit. Provide your best final answer now as a JSON answer tool call.".to_string(),
            ));
            match ResearchAgent::decide_next_step(&llm, messages.clone()).await {
                Ok((AgentAction::Answer { markdown }, p, c)) => {
                    self.budget.record_llm_call(p, c);
                    self.store_answer(&markdown).await;
                }
                _ => {
                    let fallback = "_The research agent did not produce a final answer within its limits._";
                    self.store_answer(fallback).await;
                }
            }
        }

        let stats = serde_json::json!({
            "steps": step_index,
            "elapsed_secs": self.start_time.elapsed().as_secs(),
            "spent_usd": self.budget.spent_usd(),
        });
        self.repo
            .update_run_stats(&self.run_id, &stats.to_string())
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        self.log("INFO", "research", "Research completed").await;
        self.set_status("completed").await;
        Ok(PipelineState::Completed)
    }

    async fn store_answer(&self, markdown: &str) {
        if let Err(e) = self.repo.create_research_result(&self.run_id, markdown).await {
            error!(error = %e, "Failed to store research answer");
        }
        if let Some(ref events) = self.events {
            events.emit_research_answer(markdown);
        }
    }

    async fn record_step(&self, step_index: u32, step_type: &str, content: &str, url: Option<&str>) {
        let step_id = match self
            .repo
            .create_research_step(&self.run_id, step_index as i64, step_type, content, url)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                error!(error = %e, "Failed to store research step");
                String::new()
            }
        };
        if let Some(ref events) = self.events {
            events.emit_research_step(&step_id, step_index, step_type, content, url);
        }
    }

    fn emit_progress(&self, steps_done: u32, max_steps: u32) {
        if let Some(ref events) = self.events {
            events.emit_progress(ProgressStats {
                rows_found: steps_done as u64,
                pages_fetched: 0,
                pages_total: 0,
                queries_executed: steps_done as u64,
                queries_total: max_steps as u64,
                elapsed_secs: self.start_time.elapsed().as_secs(),
                spent_usd: self.budget.spent_usd(),
            });
        }
    }

    /// Returns true if the run was cancelled. Blocks while paused.
    async fn handle_commands(&mut self) -> bool {
        loop {
            match self.cmd_rx.try_recv() {
                Ok(PipelineCommand::Cancel) => return true,
                Ok(PipelineCommand::Pause) => {
                    self.set_status("paused").await;
                    // Block until resumed or cancelled.
                    while let Some(cmd) = self.cmd_rx.recv().await {
                        match cmd {
                            PipelineCommand::Resume => {
                                self.set_status("running").await;
                                break;
                            }
                            PipelineCommand::Cancel => return true,
                            _ => {}
                        }
                    }
                }
                Ok(_) => {}
                Err(_) => return false,
            }
        }
    }

    fn check_limits(&self) -> Option<String> {
        if self.budget.spent_usd() >= self.config.max_budget_usd {
            return Some(format!("budget limit reached (${:.4})", self.budget.spent_usd()));
        }
        let max_secs = self.config.stop.max_duration_secs;
        if max_secs > 0 && self.start_time.elapsed().as_secs() >= max_secs {
            return Some(format!("time limit reached ({max_secs}s)"));
        }
        None
    }

    async fn set_status(&self, status: &str) {
        if let Err(e) = self.repo.update_run_status(&self.run_id, status).await {
            error!(error = %e, "Failed to update run status");
        }
        if let Some(ref events) = self.events {
            events.emit_status_changed(status);
        }
    }

    async fn fail(&self, error_msg: &str) {
        let _ = self.repo.update_run_error(&self.run_id, error_msg).await;
        if let Some(ref events) = self.events {
            events.emit_error(error_msg);
            events.emit_status_changed("failed");
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

/// Format web search results into a compact numbered list for the agent.
fn format_search_results(results: &[crate::providers::search::types::SearchResult]) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        out.push_str(&format!(
            "{}. {}\n   URL: {}\n   {}\n",
            i + 1,
            r.title,
            r.url,
            r.snippet
        ));
    }
    out
}
