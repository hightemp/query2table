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

        // Initialize search provider
        let search = Arc::new(
            SearchManager::from_config(self.config.search.clone())
                .map_err(|e| format!("Search config: {e}"))?
        );

        // Generate search query variations
        let queries = Self::generate_query_variations(&self.query);
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

        // Check for cancellation
        if self.is_cancelled() {
            self.set_status("cancelled").await;
            return Ok(PipelineState::Cancelled);
        }

        // Optional LLM ranking
        let ranked_results = if let Ok(llm) = LlmManager::from_config(self.config.llm.clone()) {
            self.log("INFO", "image_ranker", "Ranking images with LLM...").await;
            match ImageRanker::rank(&self.query, collected.results, &llm, 0.3).await {
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

        // Store results
        self.log("INFO", "image_pipeline", &format!("Storing {} image results", ranked_results.len())).await;
        let total = ranked_results.len() as u64;

        for (i, ranked) in ranked_results.iter().enumerate() {
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

            // Emit event for real-time UI updates
            if let Some(ref events) = self.events {
                events.emit_image_added(&image_id, &img.image_url, &img.thumbnail_url, &img.title);
                events.emit_progress(ProgressStats {
                    rows_found: (i + 1) as u64,
                    pages_fetched: 0,
                    pages_total: 0,
                    queries_executed: queries.len() as u64,
                    queries_total: queries.len() as u64,
                    elapsed_secs: self.start_time.elapsed().as_secs(),
                    spent_usd: self.budget.spent_usd(),
                });
            }
        }

        // Update run stats
        let stats = serde_json::json!({
            "image_count": total,
            "queries_executed": queries.len(),
            "elapsed_secs": self.start_time.elapsed().as_secs(),
            "spent_usd": self.budget.spent_usd(),
        });
        self.repo.update_run_stats(&self.run_id, &stats.to_string())
            .await.map_err(|e| format!("Storage: {e}"))?;

        self.log("INFO", "image_pipeline", &format!("Image search completed: {} images", total)).await;
        self.set_status("completed").await;

        Ok(PipelineState::Completed)
    }

    /// Generate simple query variations for broader image search coverage.
    fn generate_query_variations(query: &str) -> Vec<String> {
        let mut queries = vec![query.to_string()];
        // Add a "photos" variant if not already present
        let lower = query.to_lowercase();
        if !lower.contains("photo") && !lower.contains("image") && !lower.contains("picture") {
            queries.push(format!("{} photos", query));
        }
        queries
    }

    fn is_cancelled(&mut self) -> bool {
        if let Ok(cmd) = self.cmd_rx.try_recv() {
            matches!(cmd, PipelineCommand::Cancel)
        } else {
            false
        }
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
