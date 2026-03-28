use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use super::types::*;
use super::brave::BraveSearchProvider;
use super::serper::SerperProvider;
use crate::utils::retry::{retry_with_backoff, RetryAction, RetryConfig};

/// No-op image provider for testing with `with_providers`.
struct NoopImageProvider;

#[async_trait::async_trait]
impl ImageSearchProvider for NoopImageProvider {
    async fn search_images(&self, _query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError> {
        Ok(vec![])
    }
}

/// Which search backend to use.
#[derive(Debug, Clone, PartialEq)]
pub enum SearchBackend {
    Brave,
    Serper,
}

/// Configuration for the search manager.
#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub primary: SearchBackend,
    pub brave_api_key: String,
    pub serper_api_key: String,
    pub num_results: u32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            primary: SearchBackend::Brave,
            brave_api_key: String::new(),
            serper_api_key: String::new(),
            num_results: 10,
        }
    }
}

/// Manages search providers with primary/fallback routing.
pub struct SearchManager {
    primary: Arc<dyn SearchProvider>,
    fallback: Option<Arc<dyn SearchProvider>>,
    image_primary: Arc<dyn ImageSearchProvider>,
    image_fallback: Option<Arc<dyn ImageSearchProvider>>,
    config: SearchConfig,
}

impl SearchManager {
    pub fn from_config(config: SearchConfig) -> Result<Self, SearchError> {
        let (primary, fallback, image_primary, image_fallback) = match config.primary {
            SearchBackend::Brave => {
                if config.brave_api_key.is_empty() {
                    return Err(SearchError::NotConfigured(
                        "Brave Search API key is required".to_string()
                    ));
                }
                let brave = Arc::new(BraveSearchProvider::new(config.brave_api_key.clone()));
                let primary: Arc<dyn SearchProvider> = brave.clone();
                let image_primary: Arc<dyn ImageSearchProvider> = brave;
                let (fallback, image_fallback): (Option<Arc<dyn SearchProvider>>, Option<Arc<dyn ImageSearchProvider>>) =
                    if !config.serper_api_key.is_empty() {
                        let serper = Arc::new(SerperProvider::new(config.serper_api_key.clone()));
                        (Some(serper.clone() as Arc<dyn SearchProvider>), Some(serper as Arc<dyn ImageSearchProvider>))
                    } else {
                        (None, None)
                    };
                (primary, fallback, image_primary, image_fallback)
            }
            SearchBackend::Serper => {
                if config.serper_api_key.is_empty() {
                    return Err(SearchError::NotConfigured(
                        "Serper API key is required".to_string()
                    ));
                }
                let serper = Arc::new(SerperProvider::new(config.serper_api_key.clone()));
                let primary: Arc<dyn SearchProvider> = serper.clone();
                let image_primary: Arc<dyn ImageSearchProvider> = serper;
                let (fallback, image_fallback): (Option<Arc<dyn SearchProvider>>, Option<Arc<dyn ImageSearchProvider>>) =
                    if !config.brave_api_key.is_empty() {
                        let brave = Arc::new(BraveSearchProvider::new(config.brave_api_key.clone()));
                        (Some(brave.clone() as Arc<dyn SearchProvider>), Some(brave as Arc<dyn ImageSearchProvider>))
                    } else {
                        (None, None)
                    };
                (primary, fallback, image_primary, image_fallback)
            }
        };

        info!(
            primary = primary.provider_name(),
            has_fallback = fallback.is_some(),
            "Search manager initialized"
        );

        Ok(Self { primary, fallback, image_primary, image_fallback, config })
    }

    /// Execute a search, with retry on transient errors, then falling back to secondary provider.
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        self.search_provider_with_retry(query, self.config.num_results).await
    }

    /// Execute a search with custom result count.
    pub async fn search_with_count(
        &self,
        query: &str,
        num_results: u32,
    ) -> Result<Vec<SearchResult>, SearchError> {
        self.search_provider_with_retry(query, num_results).await
    }

    /// Internal: search with retry on the primary, then fallback.
    async fn search_provider_with_retry(
        &self,
        query: &str,
        num_results: u32,
    ) -> Result<Vec<SearchResult>, SearchError> {
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        };

        let primary = self.primary.clone();
        let q = query.to_string();

        let primary_result = retry_with_backoff(&retry_config, "search_primary", || {
            let primary = primary.clone();
            let q = q.clone();
            async move {
                let search_query = SearchQuery::new(&q, num_results);
                match primary.search(search_query).await {
                    Ok(results) => (Ok(results), RetryAction::Success, None),
                    Err(SearchError::RateLimited { retry_after_secs }) => {
                        let hint = retry_after_secs.map(Duration::from_secs);
                        (Err(SearchError::RateLimited { retry_after_secs }), RetryAction::Retry, hint)
                    }
                    Err(SearchError::ConnectionError(msg)) => {
                        (Err(SearchError::ConnectionError(msg)), RetryAction::Retry, None)
                    }
                    Err(e) => (Err(e), RetryAction::Fail, None),
                }
            }
        }).await;

        match primary_result {
            Ok(results) => Ok(results),
            Err(e) => {
                if let Some(ref fallback) = self.fallback {
                    warn!(
                        primary = self.primary.provider_name(),
                        fallback = fallback.provider_name(),
                        error = %e,
                        "Primary search failed after retries, trying fallback"
                    );
                    let fallback_query = SearchQuery::new(query, num_results);
                    fallback.search(fallback_query).await
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Create a SearchManager with custom providers (for testing).
    pub fn with_providers(
        primary: Arc<dyn SearchProvider>,
        fallback: Option<Arc<dyn SearchProvider>>,
        config: SearchConfig,
    ) -> Self {
        // For testing: use dummy image providers (no-op)
        let image_primary: Arc<dyn ImageSearchProvider> = Arc::new(NoopImageProvider);
        Self { primary, fallback, image_primary, image_fallback: None, config }
    }

    /// Execute an image search, with retry on transient errors, then falling back.
    pub async fn search_images(&self, query: &str) -> Result<Vec<ImageSearchResult>, SearchError> {
        self.search_images_with_count(query, self.config.num_results).await
    }

    /// Execute an image search with custom result count.
    pub async fn search_images_with_count(
        &self,
        query: &str,
        num_results: u32,
    ) -> Result<Vec<ImageSearchResult>, SearchError> {
        let retry_config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        };

        let image_primary = self.image_primary.clone();
        let q = query.to_string();

        let primary_result = retry_with_backoff(&retry_config, "image_search_primary", || {
            let image_primary = image_primary.clone();
            let q = q.clone();
            async move {
                let search_query = SearchQuery::new(&q, num_results);
                match image_primary.search_images(search_query).await {
                    Ok(results) => (Ok(results), RetryAction::Success, None),
                    Err(SearchError::RateLimited { retry_after_secs }) => {
                        let hint = retry_after_secs.map(Duration::from_secs);
                        (Err(SearchError::RateLimited { retry_after_secs }), RetryAction::Retry, hint)
                    }
                    Err(SearchError::ConnectionError(msg)) => {
                        (Err(SearchError::ConnectionError(msg)), RetryAction::Retry, None)
                    }
                    Err(e) => (Err(e), RetryAction::Fail, None),
                }
            }
        }).await;

        match primary_result {
            Ok(results) => Ok(results),
            Err(e) => {
                if let Some(ref fallback) = self.image_fallback {
                    warn!(
                        error = %e,
                        "Primary image search failed after retries, trying fallback"
                    );
                    let fallback_query = SearchQuery::new(query, num_results);
                    fallback.search_images(fallback_query).await
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn primary_name(&self) -> &str {
        self.primary.provider_name()
    }

    pub fn config(&self) -> &SearchConfig {
        &self.config
    }

    /// Build SearchConfig from settings stored in the database.
    pub fn config_from_settings(settings: &std::collections::HashMap<String, String>) -> SearchConfig {
        let primary = match settings.get("search_provider").map(|s| s.as_str()) {
            Some("serper") => SearchBackend::Serper,
            _ => SearchBackend::Brave,
        };

        SearchConfig {
            primary,
            brave_api_key: settings.get("brave_api_key").cloned().unwrap_or_default(),
            serper_api_key: settings.get("serper_api_key").cloned().unwrap_or_default(),
            num_results: settings.get("search_results_per_query")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_default_config() {
        let config = SearchConfig::default();
        assert_eq!(config.primary, SearchBackend::Brave);
        assert_eq!(config.num_results, 10);
    }

    #[test]
    fn test_config_from_settings() {
        let mut settings = HashMap::new();
        settings.insert("search_provider".to_string(), "serper".to_string());
        settings.insert("serper_api_key".to_string(), "key123".to_string());
        settings.insert("search_results_per_query".to_string(), "20".to_string());

        let config = SearchManager::config_from_settings(&settings);
        assert_eq!(config.primary, SearchBackend::Serper);
        assert_eq!(config.serper_api_key, "key123");
        assert_eq!(config.num_results, 20);
    }

    #[test]
    fn test_manager_requires_brave_key() {
        let config = SearchConfig {
            primary: SearchBackend::Brave,
            brave_api_key: String::new(),
            ..Default::default()
        };
        assert!(SearchManager::from_config(config).is_err());
    }

    #[test]
    fn test_manager_brave_with_serper_fallback() {
        let config = SearchConfig {
            primary: SearchBackend::Brave,
            brave_api_key: "brave-key".to_string(),
            serper_api_key: "serper-key".to_string(),
            num_results: 10,
        };
        let manager = SearchManager::from_config(config).unwrap();
        assert_eq!(manager.primary_name(), "brave");
        assert!(manager.fallback.is_some());
    }

    #[test]
    fn test_manager_serper_no_fallback() {
        let config = SearchConfig {
            primary: SearchBackend::Serper,
            serper_api_key: "serper-key".to_string(),
            brave_api_key: String::new(),
            num_results: 10,
        };
        let manager = SearchManager::from_config(config).unwrap();
        assert_eq!(manager.primary_name(), "serper");
        assert!(manager.fallback.is_none());
    }
}
