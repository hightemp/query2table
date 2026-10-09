use std::collections::HashSet;
use tracing::{debug, warn};

use crate::providers::search::{ImageSearchResult, SearchError, SearchManager};
use crate::roles::search_executor::{report, QueryProgress};

/// Collected image search results.
#[derive(Debug)]
pub struct CollectedImageResults {
    pub results: Vec<ImageSearchResult>,
    pub total_queries_executed: usize,
    pub failed_queries: usize,
}

/// Executes image search queries, collecting and deduplicating by image URL.
pub struct ImageSearcher;

impl ImageSearcher {
    /// Execute a batch of image search queries.
    pub async fn execute(
        queries: &[String],
        search: &SearchManager,
        num_results: u32,
    ) -> Result<CollectedImageResults, SearchError> {
        Self::execute_reporting(queries, search, num_results, None).await
    }

    /// Like `execute`, reporting each query as it starts and ends.
    pub async fn execute_reporting(
        queries: &[String],
        search: &SearchManager,
        num_results: u32,
        progress: Option<&tokio::sync::mpsc::UnboundedSender<QueryProgress>>,
    ) -> Result<CollectedImageResults, SearchError> {
        debug!(query_count = queries.len(), "Executing image search queries");

        let mut all_results = Vec::new();
        let mut seen_urls = HashSet::new();
        let mut failed_count = 0;
        let mut executed_count = 0;

        let mut stopped = false;
        for (index, query) in queries.iter().enumerate() {
            if stopped {
                report(progress, QueryProgress::Skipped { index });
                continue;
            }
            report(progress, QueryProgress::Started { index });
            match search.search_images_with_count(query, num_results).await {
                Ok(results) => {
                    let count = results.len();
                    let before = all_results.len();
                    debug!(
                        query = %query,
                        results = results.len(),
                        "Image search query completed"
                    );
                    for result in results {
                        if seen_urls.insert(result.image_url.clone()) {
                            all_results.push(result);
                        }
                    }
                    report(progress, QueryProgress::Finished { index, results: count, kept: all_results.len() - before });
                }
                Err(SearchError::BudgetExceeded) => {
                    warn!("[FIX:cost] Stopping searches at spending limit");
                    report(progress, QueryProgress::Skipped { index });
                    stopped = true;
                    continue;
                }
                Err(e) => {
                    warn!(query = %query, error = %e, "Image search query failed");
                    report(progress, QueryProgress::Failed { index, error: e.to_string() });
                    failed_count += 1;
                }
            }
            executed_count += 1;
        }

        debug!(
            total_results = all_results.len(),
            total_queries = queries.len(),
            failed = failed_count,
            "Image search execution complete"
        );

        Ok(CollectedImageResults {
            results: all_results,
            total_queries_executed: executed_count,
            failed_queries: failed_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collected_image_results() {
        let results = CollectedImageResults {
            results: vec![
                ImageSearchResult {
                    image_url: "https://example.com/1.jpg".to_string(),
                    thumbnail_url: "https://example.com/1t.jpg".to_string(),
                    title: "Image 1".to_string(),
                    source_url: "https://example.com".to_string(),
                    width: Some(800),
                    height: Some(600),
                },
            ],
            total_queries_executed: 1,
            failed_queries: 0,
        };
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.total_queries_executed, 1);
    }
}
