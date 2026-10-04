use tracing::{debug, warn};

use crate::providers::search::{SearchManager, SearchResult, SearchError};
use super::search_planner::PlannedSearch;

/// Collected and deduplicated search results from executing search queries.
#[derive(Debug, Clone)]
pub struct CollectedResults {
    pub results: Vec<SearchResultWithQuery>,
    pub total_queries_executed: usize,
    pub failed_queries: usize,
}

/// A search result coupled with the query that produced it.
#[derive(Debug, Clone)]
pub struct SearchResultWithQuery {
    pub result: SearchResult,
    pub query_text: String,
    pub language: String,
}

/// Executes search queries against search providers, collecting and deduplicating URLs.
pub struct SearchExecutor;

impl SearchExecutor {
    /// Execute a batch of search queries, collecting results and deduplicating by URL.
    pub async fn execute(
        queries: &[PlannedSearch],
        search: &SearchManager,
    ) -> Result<CollectedResults, SearchError> {
        Self::execute_limited(queries, search, None).await
    }

    /// Like `execute`, keeping at most `max_per_query` new pages from each query
    /// ("Pages per query" setting).
    pub async fn execute_limited(
        queries: &[PlannedSearch],
        search: &SearchManager,
        max_per_query: Option<usize>,
    ) -> Result<CollectedResults, SearchError> {
        debug!(query_count = queries.len(), "Executing search queries");

        let mut all_results = Vec::new();
        let mut seen_urls = std::collections::HashSet::new();
        let mut failed_count = 0;
        let mut executed_count = 0;

        // Execute queries sorted by priority (1 = highest)
        let mut sorted_queries: Vec<&PlannedSearch> = queries.iter().collect();
        sorted_queries.sort_by_key(|q| q.priority);

        for query in &sorted_queries {
            match search.search(&query.query_text).await {
                Ok(results) => {
                    debug!(
                        query = %query.query_text,
                        results = results.len(),
                        "Search query completed"
                    );
                    let mut taken = 0;
                    for result in results {
                        if max_per_query.is_some_and(|max| taken >= max) {
                            break;
                        }
                        // Deduplicate by URL
                        if seen_urls.insert(result.url.clone()) {
                            taken += 1;
                            all_results.push(SearchResultWithQuery {
                                result,
                                query_text: query.query_text.clone(),
                                language: query.language.clone(),
                            });
                        }
                    }
                }
                Err(SearchError::BudgetExceeded) => {
                    warn!("[FIX:cost] Stopping searches at spending limit");
                    break;
                }
                Err(e) => {
                    warn!(
                        query = %query.query_text,
                        error = %e,
                        "Search query failed"
                    );
                    failed_count += 1;
                }
            }
            executed_count += 1;
        }

        debug!(
            total_results = all_results.len(),
            total_queries = queries.len(),
            failed = failed_count,
            "Search execution complete"
        );

        Ok(CollectedResults {
            results: all_results,
            total_queries_executed: executed_count,
            failed_queries: failed_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::search::SearchResult;

    struct FixedSearch;

    #[async_trait::async_trait]
    impl crate::providers::search::types::SearchProvider for FixedSearch {
        async fn search(
            &self,
            query: crate::providers::search::types::SearchQuery,
        ) -> Result<Vec<SearchResult>, SearchError> {
            Ok((0..5)
                .map(|i| SearchResult {
                    title: format!("{} {i}", query.query),
                    url: format!("https://{}.example/{i}", query.query.replace(' ', "-")),
                    snippet: String::new(),
                })
                .collect())
        }
        fn provider_name(&self) -> &str {
            "fixed"
        }
        async fn health_check(&self) -> Result<(), SearchError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn each_query_contributes_at_most_the_pages_per_query_setting() {
        let search = SearchManager::with_providers(std::sync::Arc::new(FixedSearch), None, Default::default());
        let planned = |text: &str| PlannedSearch {
            query_text: text.into(),
            language: "en".into(),
            geo_target: None,
            priority: 1,
        };
        let queries = [planned("first"), planned("second")];
        let limited = SearchExecutor::execute_limited(&queries, &search, Some(2)).await.unwrap();
        assert_eq!(limited.results.len(), 4);
        let all = SearchExecutor::execute(&queries, &search).await.unwrap();
        assert_eq!(all.results.len(), 10);
    }

    #[test]
    fn test_collected_results() {
        let results = CollectedResults {
            results: vec![
                SearchResultWithQuery {
                    result: SearchResult {
                        title: "Test".to_string(),
                        url: "https://example.com".to_string(),
                        snippet: "A test".to_string(),
                    },
                    query_text: "test query".to_string(),
                    language: "en".to_string(),
                },
            ],
            total_queries_executed: 1,
            failed_queries: 0,
        };
        assert_eq!(results.results.len(), 1);
        assert_eq!(results.failed_queries, 0);
    }

    #[test]
    fn test_url_dedup_logic() {
        let mut seen = std::collections::HashSet::new();
        assert!(seen.insert("https://a.com".to_string()));
        assert!(seen.insert("https://b.com".to_string()));
        assert!(!seen.insert("https://a.com".to_string())); // duplicate
    }
}
