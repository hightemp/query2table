use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::providers::http::client::HttpFetcher;
use crate::providers::http::rate_limiter::RateLimiter;
use crate::providers::llm::manager::LlmManager;
use crate::providers::http::images::picture;
use crate::providers::llm::capabilities::VisionPlan;
use crate::providers::llm::types::{ImageInput, Message};
use crate::providers::search::manager::SearchManager;
use crate::storage::repository::Repository;

use crate::roles::chart;
use crate::roles::pdf_parser::PdfParser;
use crate::roles::research_agent::{markdown_images, AgentAction, PriorTurn, ResearchAgent, Tools};
use crate::storage::repository::ResearchImageRow;

use super::files::RunFiles;

use super::control::{RunControl, RunSupervisor};
use super::budget_tracker::BudgetTracker;
use super::usage_journal::UsageJournal;
use super::events::{EventPublisher, ProgressStats};
use super::pipeline::{PipelineCommand, PipelineConfig, PipelineState};

/// Maximum number of agent tool-call iterations per research run.
/// Upper bound for the steps of one research turn, whatever the run asks for.
pub const MAX_RESEARCH_STEPS: u32 = 200;

/// Steps allowed for a turn: the run's "Max steps" stop condition, within 1..=200.
pub fn max_steps(config: &PipelineConfig) -> u32 {
    (config.stop.target_row_count as u32).clamp(1, MAX_RESEARCH_STEPS)
}
/// Image results shown to the agent from one image search.
const IMAGE_RESULTS: usize = 10;
/// Results whose pictures a model that sees images gets with an image search, at most
/// "Pictures per request" (`max_inline_images`).
const IMAGE_PREVIEWS: usize = 6;
const PREVIEW_SIDE: u32 = 384;
/// Longest side of an image the agent looks at with `view_image`.
const VIEW_SIDE: u32 = 1024;
/// Longest side of the pictures kept with an answer.
const KEEP_SIDE: u32 = 1600;
const IMAGE_BYTES: usize = 15 * 1024 * 1024;

/// An image address the agent came across: in image results, on a page it read, or one it looked at.
#[derive(Debug, Clone)]
struct SeenImage {
    /// The page the image is on.
    source_url: Option<String>,
    title: String,
    /// A smaller copy offered by the search provider.
    thumbnail: Option<String>,
}

/// The reply to a web tool in a files-only run.
const WEB_OFF: &str = "Web search is turned off for this run. Answer from the attached files with read_file.";

/// Agentic research pipeline.
/// The LLM drives a tool-calling loop (search / fetch / think) and finishes by
/// producing a Markdown answer. Each step is streamed and persisted.
pub struct ResearchPipeline {
    run_id: String,
    /// The question of this turn.
    query: String,
    turn_index: i64,
    /// Earlier turns of the conversation; empty for a new research run.
    history: Vec<PriorTurn>,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    control: RunControl,
    supervisor: Option<RunSupervisor>,
    budget: BudgetTracker,
    start_time: Instant,
    /// Search used instead of one built from the settings (tests).
    search_override: Option<SearchManager>,
}

impl ResearchPipeline {
    /// Uses this search manager instead of building one from the settings.
    pub fn with_search(mut self, search: SearchManager) -> Self {
        self.search_override = Some(search);
        self
    }

    pub fn new(
        run_id: String,
        query: String,
        config: PipelineConfig,
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
    ) -> (Self, mpsc::Sender<PipelineCommand>) {
        Self::turn(run_id, query, 0, Vec::new(), config, repo, events)
    }

    /// A follow-up question in an existing research conversation.
    pub fn follow_up(
        run_id: String,
        question: String,
        turn_index: i64,
        history: Vec<PriorTurn>,
        config: PipelineConfig,
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
    ) -> (Self, mpsc::Sender<PipelineCommand>) {
        Self::turn(run_id, question, turn_index, history, config, repo, events)
    }

    fn turn(
        run_id: String,
        query: String,
        turn_index: i64,
        history: Vec<PriorTurn>,
        config: PipelineConfig,
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
    ) -> (Self, mpsc::Sender<PipelineCommand>) {
        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        let budget = BudgetTracker::new(config.max_budget_usd)
            .with_journal(UsageJournal::for_run(repo.clone(), &run_id, "research", &query));
        let (control, supervisor) =
            RunControl::new(run_id.clone(), repo.clone(), events.clone(), cmd_rx);
        let supervisor = supervisor.with_budget(budget.clone());

        let pipeline = Self {
            run_id,
            query,
            turn_index,
            history,
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

    pub async fn run(mut self) -> Result<PipelineState, String> {
        info!(run_id = %self.run_id, query = %self.query, "Research pipeline started");

        if self.turn_index == 0 {
            let config_json = self.config.run_config("research");
            self.repo
                .create_run_with_type(&self.run_id, &self.query, &config_json.to_string(), "research")
                .await
                .map_err(|e| format!("Storage: {e}"))?;
        }
        self.repo
            .link_attachments(&self.run_id, &self.config.attachments, self.turn_index as i64)
            .await
            .map_err(|e| format!("Storage: {e}"))?;
        let limits = serde_json::json!({
            "target_row_count": max_steps(&self.config),
            "max_budget_usd": self.config.max_budget_usd,
            "max_duration_seconds": self.config.stop.max_duration_secs,
        });
        self.repo
            .create_research_turn(&self.run_id, self.turn_index, &self.query, &limits.to_string())
            .await
            .map_err(|e| format!("Storage: {e}"))?;

        let supervisor = self.supervisor.take().ok_or_else(|| "Pipeline already started".to_string())?;
        let (repo, run_id, turn_index, budget) =
            (self.repo.clone(), self.run_id.clone(), self.turn_index, self.budget.clone());
        let result = supervisor.run(self.run_inner()).await;

        // A cancelled or failed turn keeps the steps it collected; earlier turns are untouched.
        let status = match &result {
            Ok(PipelineState::Completed) => None,
            Ok(PipelineState::Cancelled) => Some("cancelled"),
            _ => Some("failed"),
        };
        if let Some(status) = status {
            if let Err(e) = repo.finish_research_turn(&run_id, turn_index, None, &[], status).await {
                error!(error = %e, "Failed to record the research turn status");
            }
        }
        if let Ok(snapshot) = serde_json::to_string(&budget.snapshot()) {
            if let Err(e) = repo.set_research_turn_accounting(&run_id, turn_index, &snapshot).await {
                error!(error = %e, "Failed to store the research turn cost");
            }
        }
        result
    }

    async fn run_inner(mut self) -> Result<PipelineState, String> {
        self.set_status("running").await;
        self.log("INFO", "research", "Starting agentic research...").await;

        // LLM is mandatory for research mode.
        let llm = match LlmManager::from_config_with_diagnostics(
            self.config.llm.clone(),
            self.control.issue_sender(),
        ) {
            Ok(m) => m
                .with_accounting(self.budget.observer())
                .with_turn(self.turn_index as u32),
            Err(e) => {
                let msg = format!("LLM not configured: {e}");
                self.log("ERROR", "research", &msg).await;
                return Ok(PipelineState::Failed(msg));
            }
        };

        // A files-only run needs no search provider.
        let search = if !self.config.web_search {
            None
        } else if let Some(search) = self.search_override.take() {
            Some(Arc::new(search.with_accounting(self.budget.observer())))
        } else {
            match SearchManager::from_config(self.config.search.clone()) {
                Ok(s) => Some(Arc::new(s.with_accounting(self.budget.observer()))),
                Err(e) => {
                    let msg = format!("Search not configured: {e}");
                    self.log("ERROR", "research", &msg).await;
                    return Ok(PipelineState::Failed(msg));
                }
            }
        };

        // Files of the whole conversation, with scans read and pictures described as needed.
        let files = self.load_files(&llm).await;

        let rate_limiter = RateLimiter::new(std::time::Duration::from_millis(self.config.rate_limit_ms));
        let fetcher = Arc::new(
            HttpFetcher::new(rate_limiter)
                .with_max_body_bytes(self.config.max_page_size_bytes)
                .with_accounting(self.budget.observer()),
        );
        let max_pdf_chars = if self.config.enable_content_truncation {
            Some(self.config.max_pdf_text_chars)
        } else {
            None
        };

        let max_steps = max_steps(&self.config);

        // Build the running transcript: earlier turns as context, the files, then this turn's request.
        let images = search.is_some() && self.config.research_images;
        let view = images
            && VisionPlan::for_config(&self.config.llm, &self.config.llm_vision, &self.config.vision_model).await.main_sees;
        let tools = Tools { web: search.is_some(), files: files.is_some(), images, view };
        let mut seen: HashMap<String, SeenImage> = HashMap::new();
        let mut messages = vec![Message::system(ResearchAgent::system_prompt_with(max_steps as usize, tools))];
        if let Some(context) = ResearchAgent::conversation_context(&self.history, self.config.context.history_chars) {
            messages.push(Message::user(context));
        }
        if let Some(files) = &files {
            let (documents, whole) = files.documents().await;
            if !whole.is_empty() {
                self.log("INFO", "research", &format!("Sent {} PDF file(s) to the model whole", whole.len())).await;
            }
            let mut message = Message::user_with_images(
                files.context_with(&self.query, self.config.context.file_chars, &whole).await,
                files.images(self.config.max_inline_images).await,
            );
            message.documents = documents;
            messages.push(message);
        }
        messages.push(Message::user(format!("Research request:\n{}", self.query)));

        let mut step_index: u32 = 0;
        let mut answered = false;
        let mut search_count: u32 = 0;
        let mut fetch_count: u32 = 0;

        while step_index < max_steps {
            // Stop on budget/time limits.
            if let Some(reason) = self.check_limits() {
                self.log("INFO", "research", &format!("Stopping: {reason}")).await;
                break;
            }

            let (action, _prompt_tokens, _completion_tokens) =
                match ResearchAgent::decide_next_step(&llm, messages.clone()).await {
                    Ok(v) => v,
                    Err(e) => {
                        warn!(error = %e, "Agent step failed");
                        self.record_error(
                            step_index,
                            &format!("The model produced an invalid response: {e}"),
                        )
                        .await;
                        // Nudge the model to emit valid JSON next time.
                        messages.push(Message::user(
                            "Your previous reply was not a valid tool call. Reply with ONE JSON tool object.".to_string(),
                        ));
                        step_index += 1;
                        continue;
                    }
                };

            match action {
                AgentAction::Search { query } if search.is_none() => {
                    messages.push(Message::assistant(serde_json::json!({ "action": "search", "query": query }).to_string()));
                    messages.push(Message::user(WEB_OFF.to_string()));
                }
                AgentAction::Fetch { url } if search.is_none() => {
                    messages.push(Message::assistant(serde_json::json!({ "action": "fetch", "url": url }).to_string()));
                    messages.push(Message::user(WEB_OFF.to_string()));
                }
                AgentAction::ReadFile { file, place, query } => {
                    let action = serde_json::json!({ "action": "read_file", "file": file, "place": place, "query": query }).to_string();
                    let observation = match &files {
                        None => "No files are attached to this conversation.".to_string(),
                        Some(files) => match files.read(&file, place, query.as_deref(), self.config.context.page_chars).await {
                            Ok(read) => {
                                self.log("INFO", "research", &format!("Read file: {}", read.label)).await;
                                fetch_count += 1;
                                self.record_step(step_index, "read", &read.label, Some(&read.url)).await;
                                format!("File content ({}):\n{}", read.label, read.text)
                            }
                            Err(e) => {
                                self.record_error(step_index, &format!("Could not read the file: {e}")).await;
                                e
                            }
                        },
                    };
                    messages.push(Message::assistant(action));
                    messages.push(Message::user(observation));
                }
                AgentAction::Search { query } => {
                    let Some(search) = &search else { unreachable!("handled above") };
                    self.log("INFO", "research", &format!("Search: {query}"))
                        .await;

                    search_count += 1;
                    let (observation, failed) = match search
                        .search(&query)
                        .await
                    {
                        Ok(results) => (format_search_results(&results), false),
                        Err(e) => {
                            warn!(error = %e, "Search failed");
                            (format!("Search failed: {e}"), true)
                        }
                    };
                    if failed {
                        self.record_error(
                            step_index,
                            &format!("Search failed for \"{query}\": {observation}"),
                        )
                        .await;
                    } else {
                        self.record_step(step_index, "search", &query, None).await;
                    }
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "search", "query": query }).to_string(),
                    ));
                    messages.push(Message::user(format!("Search results:\n{observation}")));
                }
                AgentAction::Fetch { url } => {
                    self.log("INFO", "research", &format!("Fetch: {url}")).await;
                    fetch_count += 1;
                    let mut title = None;
                    let (observation, failed) = match fetcher.fetch(&url).await {
                        Ok(page) => {
                            let markdown = if page.is_pdf() {
                                PdfParser::parse(&page.body_bytes, &url, max_pdf_chars).text
                            } else {
                                title = ResearchAgent::page_title(&page.body);
                                ResearchAgent::html_to_markdown(&page.body, &url)
                            };
                            let truncated = crate::utils::text::truncate_chars(
                                &markdown,
                                self.config.context.page_chars,
                            )
                            .to_string();
                            (truncated, false)
                        }
                        Err(e) => {
                            warn!(url = %url, error = %e, "Fetch failed");
                            (format!("Failed to fetch page: {e}"), true)
                        }
                    };
                    if failed {
                        self.record_error(
                            step_index,
                            &format!("Failed to fetch {url}: {observation}"),
                        )
                        .await;
                    } else {
                        // The full page text is fed to the model below; the step timeline
                        // shows the page title, or the text length when there is none.
                        let chars = observation.chars().count();
                        for (image, alt) in markdown_images(&observation) {
                            let title = if alt.is_empty() { title.clone().unwrap_or_default() } else { alt };
                            seen.entry(image).or_insert(SeenImage { source_url: Some(url.clone()), title, thumbnail: None });
                        }
                        let summary =
                            title.unwrap_or_else(|| format!("Read page ({chars} characters of content)."));
                        self.record_step(step_index, "fetch", &summary, Some(&url)).await;
                    }
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "fetch", "url": url }).to_string(),
                    ));
                    messages.push(Message::user(format!("Page content ({url}):\n{observation}")));
                }
                AgentAction::ImageSearch { query } => {
                    let reply = match &search {
                        Some(search) if tools.images => {
                            self.log("INFO", "research", &format!("Image search: {query}")).await;
                            search_count += 1;
                            match search.search_images(&query).await {
                                Ok(results) => {
                                    let results: Vec<_> = results.into_iter().take(IMAGE_RESULTS).collect();
                                    for r in &results {
                                        seen.entry(r.image_url.clone()).or_insert(SeenImage {
                                            source_url: Some(r.source_url.clone()).filter(|u| !u.is_empty()),
                                            title: r.title.clone(),
                                            thumbnail: Some(r.thumbnail_url.clone()).filter(|u| !u.is_empty()),
                                        });
                                    }
                                    self.record_step(step_index, "image_search", &query, None).await;
                                    let text = format!("Image results:\n{}", format_image_results(&results));
                                    if tools.view {
                                        let (shown, pictures) = image_previews(&results, IMAGE_PREVIEWS.min(self.config.max_inline_images), &self.budget).await;
                                        if shown.is_empty() {
                                            Message::user(text)
                                        } else {
                                            let list = shown.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(", ");
                                            Message::user_with_images(format!("{text}\nPictures of results {list} follow, in this order."), pictures)
                                        }
                                    } else {
                                        Message::user(text)
                                    }
                                }
                                Err(e) => {
                                    self.record_error(step_index, &format!("Image search failed for \"{query}\": {e}")).await;
                                    Message::user(format!("Image search failed: {e}"))
                                }
                            }
                        }
                        _ => Message::user("Image search is not available in this run.".to_string()),
                    };
                    messages.push(Message::assistant(serde_json::json!({ "action": "image_search", "query": query }).to_string()));
                    messages.push(reply);
                }
                AgentAction::ViewImage { url } => {
                    let reply = if !tools.view {
                        Message::user("You cannot look at images in this run; judge them by their captions and pages.".to_string())
                    } else {
                        self.log("INFO", "research", &format!("View image: {url}")).await;
                        match picture(&[&url], VIEW_SIDE, IMAGE_BYTES).await {
                            Ok(p) => {
                                self.budget.downloaded(p.downloaded as u64);
                                let known = seen.entry(url.clone()).or_insert(SeenImage { source_url: None, title: String::new(), thumbnail: None });
                                let title = Some(known.title.clone()).filter(|t| !t.trim().is_empty()).unwrap_or_else(|| file_name(&url));
                                self.record_step(step_index, "view_image", &title, Some(&url)).await;
                                Message::user_with_images(
                                    format!("Image {url} ({}×{} px):", p.width, p.height),
                                    vec![ImageInput::from_bytes(p.media_type, &p.bytes)],
                                )
                            }
                            Err(e) => {
                                self.record_error(step_index, &format!("Could not open the image {url}: {e}")).await;
                                Message::user(format!("Could not open the image: {e}"))
                            }
                        }
                    };
                    messages.push(Message::assistant(serde_json::json!({ "action": "view_image", "url": url }).to_string()));
                    messages.push(reply);
                }
                AgentAction::Think { thought } => {
                    self.log("INFO", "research", &format!("Think: {thought}")).await;
                    self.record_step(step_index, "think", &thought, None).await;
                    messages.push(Message::assistant(
                        serde_json::json!({ "action": "think", "thought": thought }).to_string(),
                    ));
                    messages.push(Message::user("Acknowledged. Continue.".to_string()));
                }
                AgentAction::Answer { markdown, follow_ups } => {
                    self.log("INFO", "research", "Agent produced final answer").await;
                    let (markdown, follow_ups) = self.checked_answer(&llm, &mut messages, markdown, follow_ups).await;
                    self.keep_answer_images(&markdown, &seen).await;
                    self.store_answer(&markdown, &follow_ups).await;
                    answered = true;
                    break;
                }
            }

            self.emit_progress(step_index + 1, max_steps, search_count, fetch_count);
            step_index += 1;
        }

        // If the loop ended without an explicit answer, request one final answer.
        if !answered && self.budget.is_exceeded() {
            self.store_answer("_The run reached its spending limit before a final answer was available. Saved activity remains available._", &[]).await;
        } else if !answered {
            self.log(
                "INFO",
                "research",
                "Step/limit reached, requesting final answer",
            )
            .await;
            messages.push(Message::user(
                "You have reached your step limit. Provide your best final answer now as a JSON answer tool call.".to_string(),
            ));
            match ResearchAgent::decide_next_step(&llm, messages.clone()).await {
                Ok((AgentAction::Answer { markdown, follow_ups }, _p, _c)) => {
                    let (markdown, follow_ups) = self.checked_answer(&llm, &mut messages, markdown, follow_ups).await;
                    self.keep_answer_images(&markdown, &seen).await;
                    self.store_answer(&markdown, &follow_ups).await;
                }
                _ => {
                    self.record_error(
                        step_index,
                        "The agent reached its step/time limit without producing a final answer.",
                    )
                    .await;
                    let fallback = "_The research agent did not produce a final answer within its limits._";
                    self.store_answer(fallback, &[]).await;
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

    /// The conversation's files with scans read and pictures described for this run's models;
    /// None when nothing is attached.
    async fn load_files(&self, llm: &LlmManager) -> Option<RunFiles> {
        let (files, logs) =
            super::files::load_for_run(self.repo.pool().clone(), &self.config, &self.run_id, llm, super::files::Prepare::ForModel).await;
        for (level, message) in logs {
            self.log(level, "files", &message).await;
        }
        files
    }

    /// The answer, with its broken ```chart blocks sent back to the agent once to be fixed (even
    /// past the step limit). A fix that is still broken is kept; a failed one keeps the answer.
    async fn checked_answer(
        &self,
        llm: &LlmManager,
        messages: &mut Vec<Message>,
        markdown: String,
        follow_ups: Vec<String>,
    ) -> (String, Vec<String>) {
        let problems = chart::problems(&markdown);
        if problems.is_empty() {
            return (markdown, follow_ups);
        }
        let list = problems.iter().map(|p| format!("- {p}")).collect::<Vec<_>>().join("\n");
        self.log("INFO", "research", &format!("Asking the agent to fix its charts:\n{list}")).await;
        messages.push(Message::assistant(
            serde_json::json!({ "action": "answer", "markdown": markdown, "follow_ups": follow_ups }).to_string(),
        ));
        messages.push(Message::user(format!(
            "Some ```chart blocks in your answer cannot be drawn:\n{list}\nFix these blocks and answer again: call \"answer\" with the whole corrected answer."
        )));
        match ResearchAgent::decide_next_step(llm, messages.clone()).await {
            Ok((AgentAction::Answer { markdown: fixed, follow_ups: fixed_follow_ups }, _, _)) => {
                let left = chart::problems(&fixed);
                if !left.is_empty() {
                    self.log("WARN", "research", &format!("Charts still broken after the fix: {}", left.join("; "))).await;
                }
                (fixed, if fixed_follow_ups.is_empty() { follow_ups } else { fixed_follow_ups })
            }
            _ => {
                self.log("WARN", "research", "The agent did not fix its charts; keeping the answer as written").await;
                (markdown, follow_ups)
            }
        }
    }

    /// Downloads the web images the answer shows and keeps them with the run, so the answer
    /// does not load them from the web. Only images the agent came across are kept.
    async fn keep_answer_images(&self, markdown: &str, seen: &HashMap<String, SeenImage>) {
        let kept: Vec<String> = match self.repo.get_research_images(&self.run_id).await {
            Ok(rows) => rows.into_iter().map(|r| r.url).collect(),
            Err(e) => {
                error!(error = %e, "Failed to read kept research images");
                Vec::new()
            }
        };
        let mut wanted = Vec::new();
        for (url, _) in markdown_images(markdown) {
            if kept.contains(&url) || !url.starts_with("http") {
                continue;
            }
            match seen.get(&url) {
                Some(found) => wanted.push((url, found.clone())),
                None => self.log("WARN", "research", &format!("Not keeping an image the agent did not find or read: {url}")).await,
            }
        }
        let most = self.config.research_max_images;
        if wanted.len() > most {
            self.log("WARN", "research", &format!("Keeping the first {most} of {} images of the answer", wanted.len())).await;
            wanted.truncate(most);
        }
        let downloads = futures::future::join_all(wanted.iter().map(|(url, found)| {
            let mut sources = vec![url.as_str()];
            sources.extend(found.thumbnail.as_deref());
            async move { picture(&sources, KEEP_SIDE, IMAGE_BYTES).await }
        }))
        .await;
        let mut count = 0;
        for ((url, found), result) in wanted.into_iter().zip(downloads) {
            match result {
                Ok(p) => {
                    self.budget.downloaded(p.downloaded as u64);
                    let row = ResearchImageRow {
                        run_id: self.run_id.clone(),
                        turn_index: self.turn_index,
                        url,
                        source_url: found.source_url,
                        title: found.title,
                        media_type: p.media_type.to_string(),
                        width: p.width as i64,
                        height: p.height as i64,
                    };
                    match self.repo.add_research_image(&row, &p.bytes).await {
                        Ok(()) => count += 1,
                        Err(e) => error!(error = %e, "Failed to keep a research image"),
                    }
                }
                Err(e) => self.log("WARN", "research", &format!("Could not keep the image {url}: {e}")).await,
            }
        }
        if count > 0 {
            self.log("INFO", "research", &format!("Kept {count} image(s) of the answer")).await;
        }
    }

    async fn store_answer(&self, markdown: &str, follow_ups: &[String]) {
        if let Err(e) = self
            .repo
            .finish_research_turn(&self.run_id, self.turn_index, Some(markdown), follow_ups, "completed")
            .await
        {
            error!(error = %e, "Failed to store research answer");
        }
        if let Some(ref events) = self.events {
            events.emit_research_answer(self.turn_index, markdown, follow_ups);
        }
    }

    async fn record_step(&self, step_index: u32, step_type: &str, content: &str, url: Option<&str>) {
        let step_id = match self
            .repo
            .create_research_step(&self.run_id, self.turn_index, step_index as i64, step_type, content, url)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                error!(error = %e, "Failed to store research step");
                String::new()
            }
        };
        if let Some(ref events) = self.events {
            events.emit_research_step(&step_id, self.turn_index, step_index, step_type, content, url);
        }
    }

    /// Record a visible error step so the user can see what went wrong, and log it.
    async fn record_error(&self, step_index: u32, message: &str) {
        self.log("WARN", "research", message).await;
        self.record_step(step_index, "error", message, None).await;
    }

    fn emit_progress(&self, steps_done: u32, max_steps: u32, searches: u32, fetches: u32) {
        if let Some(ref events) = self.events {
            events.emit_progress(ProgressStats {
                rows_found: searches as u64,
                pages_fetched: fetches as u64,
                pages_total: 0,
                queries_executed: steps_done as u64,
                queries_total: max_steps as u64,
                elapsed_secs: self.start_time.elapsed().as_secs(),
                spent_usd: self.budget.spent_usd(),
            });
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

/// Image results as a numbered list for the agent.
fn format_image_results(results: &[crate::providers::search::types::ImageSearchResult]) -> String {
    if results.is_empty() {
        return "No images found.".to_string();
    }
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        let size = match (r.width, r.height) {
            (Some(w), Some(h)) => format!(" ({w}×{h})"),
            _ => String::new(),
        };
        out.push_str(&format!("{}. {}{size}\n   Image: {}\n   Page: {}\n", i + 1, r.title.trim(), r.image_url, r.source_url));
    }
    out
}

/// Small pictures of the first image results for a model that sees images: the result numbers
/// that have one, and the pictures in the same order.
async fn image_previews(results: &[crate::providers::search::types::ImageSearchResult], most: usize, budget: &BudgetTracker) -> (Vec<usize>, Vec<ImageInput>) {
    let loaded = futures::future::join_all(
        results.iter().take(most).map(|r| async move { picture(&[&r.thumbnail_url, &r.image_url], PREVIEW_SIDE, IMAGE_BYTES).await }),
    )
    .await;
    let mut numbers = Vec::new();
    let mut pictures = Vec::new();
    for (i, p) in loaded.into_iter().enumerate() {
        if let Ok(p) = p {
            budget.downloaded(p.downloaded as u64);
            numbers.push(i + 1);
            pictures.push(ImageInput::from_bytes(p.media_type, &p.bytes));
        }
    }
    (numbers, pictures)
}

/// The last part of an address's path, for naming an image.
fn file_name(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| u.path_segments()?.rev().find(|s| !s.is_empty()).map(str::to_string))
        .unwrap_or_else(|| url.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn the_step_limit_comes_from_the_run_and_stays_within_bounds() {
        let mut config = PipelineConfig::from_settings(&HashMap::new());
        config.stop.target_row_count = 8;
        assert_eq!(max_steps(&config), 8);
        config.stop.target_row_count = 0;
        assert_eq!(max_steps(&config), 1);
        config.stop.target_row_count = 150;
        assert_eq!(max_steps(&config), 150);
        config.stop.target_row_count = 500;
        assert_eq!(max_steps(&config), 200);
    }
}
