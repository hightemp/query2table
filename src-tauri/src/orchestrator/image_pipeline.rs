use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{info, warn, error};

use crate::providers::llm::manager::LlmManager;
use crate::providers::search::manager::SearchManager;
use crate::storage::repository::Repository;

use crate::roles::image_searcher::ImageSearcher;
use crate::roles::image_ranker::ImageRanker;

use super::budget_tracker::BudgetTracker;
use super::events::{EventPublisher, ProgressStats};
use crate::roles::stopping_controller::{StoppingController, PipelineStats};

use super::pipeline::{PipelineCommand, PipelineConfig, PipelineState};

/// Simplified pipeline for image search mode.
/// Flow: Search Images → (optional) LLM Rank → Store Results → Done
pub struct ImagePipeline {
    run_id: String,
    query: String,
    config: PipelineConfig,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    cmd_rx: mpsc::Receiver<PipelineCommand>,
    budget: BudgetTracker,
    start_time: Instant,
}

impl ImagePipeline {
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
        info!(run_id = %self.run_id, query = %self.query, "Image pipeline started");

        // Create run in DB
        let config_json = serde_json::json!({ "mode": "images" });
        self.repo.create_run_with_type(&self.run_id, &self.query, &config_json.to_string(), "images")
            .await.map_err(|e| format!("Storage: {e}"))?;

        self.set_status("running").await;
        self.log("INFO", "image_pipeline", "Starting image search...").await;

        // Initialize providers
        let search = Arc::new(
            SearchManager::from_config(self.config.search.clone())
                .map_err(|e| format!("Search config: {e}"))?
        );
        let llm = LlmManager::from_config(self.config.llm.clone()).ok();

        // Generate search query variations (LLM-based or static fallback)
        let queries = match &llm {
            Some(llm_mgr) => {
                self.log("INFO", "image_pipeline", "Generating image search queries with LLM...").await;
                match Self::generate_queries_with_llm(&self.query, llm_mgr).await {
                    Ok(q) => {
                        self.budget.record_llm_call(500, 300);
                        q
                    }
                    Err(e) => {
                        warn!(error = %e, "LLM query generation failed, using static fallback");
                        self.log("WARN", "image_pipeline", &format!("LLM query generation failed: {e}, using static fallback")).await;
                        Self::generate_query_variations(&self.query)
                    }
                }
            }
            None => {
                self.log("INFO", "image_pipeline", "LLM not configured, using static query variations").await;
                Self::generate_query_variations(&self.query)
            }
        };
        self.log("INFO", "image_searcher", &format!("Searching with {} query variations", queries.len())).await;

        // Check for cancellation
        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Execute image searches
        let num_results = self.config.search.num_results;
        let collected = ImageSearcher::execute(&queries, &search, num_results)
            .await
            .map_err(|e| format!("Image search: {e}"))?;

        for _ in 0..collected.total_queries_executed {
            self.budget.record_search_call();
        }

        self.log("INFO", "image_searcher", &format!(
            "Found {} unique images from {} queries ({} failed)",
            collected.results.len(),
            collected.total_queries_executed,
            collected.failed_queries,
        )).await;

        if collected.results.is_empty() {
            self.log("WARN", "image_pipeline", "No images found").await;
            self.set_status("completed").await;
            return Ok(PipelineState::Completed);
        }

        // Check stop conditions after search
        if let Some(reason) = self.check_stop_conditions(0) {
            self.log("INFO", "stopping_controller", &format!("Stopping after search: {:?}", reason)).await;
            self.set_status("completed").await;
            return Ok(PipelineState::Completed);
        }

        // Check for cancellation
        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Optional LLM ranking
        let ranked_results = if let Some(ref llm_mgr) = llm {
            self.log("INFO", "image_ranker", "Ranking images with LLM...").await;
            match ImageRanker::rank(&self.query, collected.results, llm_mgr, 0.7).await {
                Ok(ranked) => {
                    self.budget.record_llm_call(1000, 500);
                    self.log("INFO", "image_ranker", &format!("Ranked: {} images passed relevance filter", ranked.len())).await;
                    ranked
                }
                Err(e) => {
                    warn!(error = %e, "LLM ranking failed, using unranked results");
                    self.log("WARN", "image_ranker", &format!("Ranking failed: {e}, using unranked results")).await;
                    // Cannot recover results after move — return empty
                    vec![]
                }
            }
        } else {
            self.log("INFO", "image_pipeline", "LLM not configured, skipping ranking").await;
            collected.results.into_iter()
                .map(|r| crate::roles::image_ranker::RankedImageResult {
                    result: r,
                    relevance_score: 0.5,
                })
                .collect()
        };

        // Apply target limit ("Max Images" stop condition)
        let max_images = self.config.stop.target_row_count;
        let ranked_results = if ranked_results.len() > max_images {
            self.log("INFO", "image_pipeline", &format!("Limiting results from {} to {} (max images)", ranked_results.len(), max_images)).await;
            ranked_results.into_iter().take(max_images).collect::<Vec<_>>()
        } else {
            ranked_results
        };

        // Store results
        self.log("INFO", "image_storage", &format!("Storing {} image results", ranked_results.len())).await;
        let mut stored_count = 0u64;

        for ranked in &ranked_results {
            let img = &ranked.result;
            let image_id = self.repo.create_image_result(
                &self.run_id,
                &img.image_url,
                &img.thumbnail_url,
                &img.title,
                &img.source_url,
                img.width,
                img.height,
                Some(ranked.relevance_score),
            ).await.map_err(|e| format!("Storage: {e}"))?;

            stored_count += 1;

            // Emit event for real-time UI updates
            if let Some(ref events) = self.events {
                events.emit_image_added(&image_id, &img.image_url, &img.thumbnail_url, &img.title, &img.source_url, img.width, img.height, Some(ranked.relevance_score));
                events.emit_progress(ProgressStats {
                    rows_found: stored_count,
                    pages_fetched: 0,
                    pages_total: 0,
                    queries_executed: queries.len() as u64,
                    queries_total: queries.len() as u64,
                    elapsed_secs: self.start_time.elapsed().as_secs(),
                    spent_usd: self.budget.spent_usd(),
                });
            }

            // Check stop conditions during storage
            if let Some(reason) = self.check_stop_conditions(stored_count as usize) {
                self.log("INFO", "stopping_controller", &format!("Stopping during storage: {:?}", reason)).await;
                break;
            }
        }

        // Update run stats
        let stats = serde_json::json!({
            "image_count": stored_count,
            "queries_executed": queries.len(),
            "elapsed_secs": self.start_time.elapsed().as_secs(),
            "spent_usd": self.budget.spent_usd(),
        });
        self.repo.update_run_stats(&self.run_id, &stats.to_string())
            .await.map_err(|e| format!("Storage: {e}"))?;

        self.log("INFO", "image_pipeline", &format!("Image search completed: {} images", stored_count)).await;
        self.set_status("completed").await;

        Ok(PipelineState::Completed)
    }

    /// Generate image search queries using LLM for better diversity and coverage.
    async fn generate_queries_with_llm(query: &str, llm: &LlmManager) -> Result<Vec<String>, String> {
        use crate::providers::llm::Message;

        let system = r#"You are an image search query generator. Given a user's image search request, generate 6-10 diverse search queries optimized for finding relevant images.

Strategy:
1. Include the original query
2. Add variations with different phrasings
3. Add queries with visual descriptors (e.g. "high resolution", "professional photo", "close-up")
4. Add queries targeting specific image sources (e.g. "stock photo", "infographic", "diagram")
5. If the topic has common synonyms, use them
6. If relevant, add queries in different languages

Respond with valid JSON: {"queries": ["query1", "query2", ...]}. No markdown, no explanation."#;

        let messages = vec![
            Message::system(system),
            Message::user(format!("Generate image search queries for: {}", query)),
        ];

        let response = llm.complete(messages, true).await
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

        // Ensure original query is always included
        let mut queries = parsed.queries;
        let lower_queries: Vec<String> = queries.iter().map(|q| q.to_lowercase()).collect();
        if !lower_queries.contains(&query.to_lowercase()) {
            queries.insert(0, query.to_string());
        }

        // Cap at 12 to avoid excessive API calls
        queries.truncate(12);

        Ok(queries)
    }

    /// Static fallback: generate query variations without LLM.
    fn generate_query_variations(query: &str) -> Vec<String> {
        let mut queries = vec![query.to_string()];
        let lower = query.to_lowercase();

        // Add visual-type suffixes
        if !lower.contains("photo") && !lower.contains("image") && !lower.contains("picture") {
            queries.push(format!("{} photos", query));
            queries.push(format!("{} images", query));
        }

        // Add quality/style variations
        if !lower.contains("high resolution") && !lower.contains("hd") {
            queries.push(format!("{} high resolution", query));
        }

        // Add different angles
        queries.push(format!("best {} pictures", query));
        queries.push(format!("{} examples", query));

        queries
    }

    fn is_cancelled(&mut self) -> bool {
        if let Ok(cmd) = self.cmd_rx.try_recv() {
            matches!(cmd, PipelineCommand::Cancel)
        } else {
            false
        }
    }

    fn check_stop_conditions(&self, image_count: usize) -> Option<String> {
        let stats = PipelineStats {
            row_count: image_count,
            estimated_cost_usd: self.budget.spent_usd(),
            start_time: self.start_time,
            last_batch_new_rows: 0,
            last_batch_total_rows: 0,
        };
        StoppingController::should_stop(&self.config.stop, &stats)
            .map(|reason| format!("{:?}", reason))
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
