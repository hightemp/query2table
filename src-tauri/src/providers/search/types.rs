use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for search operations.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Search request failed: {0}")]
    RequestFailed(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limited, retry after {retry_after_secs:?}s")]
    RateLimited { retry_after_secs: Option<u64> },

    #[error("Search provider not configured: {0}")]
    NotConfigured(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// A single search result from a search provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Request parameters for a web search.
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub num_results: u32,
    pub language: Option<String>,
    pub country: Option<String>,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>, num_results: u32) -> Self {
        Self {
            query: query.into(),
            num_results,
            language: None,
            country: None,
        }
    }
}

/// Trait for web search providers.
#[async_trait]
pub trait SearchProvider: Send + Sync {
    /// Execute a web search and return results.
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError>;

    /// Get the provider name.
    fn provider_name(&self) -> &str;

    /// Check if the provider is accessible.
    async fn health_check(&self) -> Result<(), SearchError>;
}

/// A single image search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSearchResult {
    pub image_url: String,
    pub thumbnail_url: String,
    pub title: String,
    pub source_url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Trait for image search providers.
#[async_trait]
pub trait ImageSearchProvider: Send + Sync {
    /// Execute an image search and return results.
    async fn search_images(&self, query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_new() {
        let query = SearchQuery::new("rust programming", 10);
        assert_eq!(query.query, "rust programming");
        assert_eq!(query.num_results, 10);
        assert!(query.language.is_none());
    }

    #[test]
    fn test_search_result_serialize() {
        let result = SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "A test result".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("example.com"));
    }

    #[test]
    fn test_image_search_result_serialize() {
        let result = ImageSearchResult {
            image_url: "https://example.com/img.jpg".to_string(),
            thumbnail_url: "https://example.com/thumb.jpg".to_string(),
            title: "Test Image".to_string(),
            source_url: "https://example.com".to_string(),
            width: Some(800),
            height: Some(600),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("img.jpg"));
        assert!(json.contains("800"));
    }
}
