use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error types for search operations.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Run spending limit reached. Increase the run's Max Cost to send more requests.")]
    BudgetExceeded,
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

    /// Bytes of responses received so far, for the usage statistics.
    fn received_bytes(&self) -> u64 {
        0
    }
}

/// Reads a JSON response body, adding its size to `received`.
pub(crate) async fn counted_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    received: &std::sync::atomic::AtomicU64,
) -> Result<T, String> {
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    received.fetch_add(bytes.len() as u64, std::sync::atomic::Ordering::Relaxed);
    serde_json::from_slice(&bytes).map_err(|e| e.to_string())
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

    /// Bytes of responses received so far, for the usage statistics.
    fn received_bytes(&self) -> u64 {
        0
    }
}

/// A video found by a web video search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoSearchResult {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub channel: Option<String>,
    /// The site: `YouTube`, `vimeo.com`…
    pub site: Option<String>,
    /// Seconds.
    pub duration: Option<u64>,
    /// `YYYY-MM-DD` when known, else the provider's words ("2 years ago").
    pub published: Option<String>,
    pub views: Option<u64>,
    pub description: Option<String>,
}

/// Trait for video search providers.
#[async_trait]
pub trait VideoSearchProvider: Send + Sync {
    async fn search_videos(&self, query: SearchQuery) -> Result<Vec<VideoSearchResult>, SearchError>;

    /// Bytes of responses received so far, for the usage statistics.
    fn received_bytes(&self) -> u64 {
        0
    }
}

/// `19:15` or `1:02:03` in seconds.
pub(crate) fn duration_secs(text: &str) -> Option<u64> {
    let text = text.trim();
    if text.is_empty() || text.split(':').count() > 3 {
        return None;
    }
    text.split(':').try_fold(0u64, |total, part| Some(total * 60 + part.trim().parse::<u64>().ok()?))
}

/// `2023-03-14T16:00:00` or `Mar 4, 2021` as `YYYY-MM-DD`; other words are kept as they are.
pub(crate) fn published_date(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some(day) = text.get(..10).filter(|day| chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").is_ok()) {
        return Some(day.to_string());
    }
    for format in ["%b %d, %Y", "%B %d, %Y", "%d %b %Y"] {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(text, format) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }
    Some(text.to_string())
}

/// The host of an address, without `www.`.
pub(crate) fn host_of(url: &str) -> Option<String> {
    url::Url::parse(url).ok()?.host_str().map(|h| h.trim_start_matches("www.").to_string())
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
