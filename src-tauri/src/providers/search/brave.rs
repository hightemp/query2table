use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

use super::types::*;

/// Brave Search API client.
pub struct BraveSearchProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveWebResult>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResult {
    title: String,
    url: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveImageResponse {
    results: Option<Vec<BraveImageResult>>,
}

#[derive(Debug, Deserialize)]
struct BraveImageResult {
    title: String,
    url: String,
    source: Option<String>,
    thumbnail: Option<BraveThumbnail>,
    properties: Option<BraveImageProperties>,
}

#[derive(Debug, Deserialize)]
struct BraveThumbnail {
    src: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveImageProperties {
    width: Option<u32>,
    height: Option<u32>,
}

impl BraveSearchProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://api.search.brave.com/res/v1".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl SearchProvider for BraveSearchProvider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let url = format!("{}/web/search", self.base_url);

        debug!(query = %query.query, num_results = query.num_results, "Brave search");

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .query(&[
                ("q", query.query.as_str()),
                ("count", &query.num_results.to_string()),
            ]);

        if let Some(ref lang) = query.language {
            request = request.query(&[("search_lang", lang.as_str())]);
        }
        if let Some(ref country) = query.country {
            request = request.query(&[("country", country.as_str())]);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_connect() {
                SearchError::ConnectionError(e.to_string())
            } else {
                SearchError::RequestFailed(e.to_string())
            }
        })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SearchError::AuthError("Invalid Brave API key".to_string()));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(SearchError::RateLimited { retry_after_secs: retry_after });
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(SearchError::RequestFailed(
                format!("Brave API error {}: {}", status, body)
            ));
        }

        let brave_response: BraveResponse = response.json().await.map_err(|e| {
            SearchError::ParseError(format!("Failed to parse Brave response: {}", e))
        })?;

        let results = brave_response
            .web
            .map(|w| w.results)
            .unwrap_or_default()
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                snippet: r.description.unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }

    fn provider_name(&self) -> &str {
        "brave"
    }

    async fn health_check(&self) -> Result<(), SearchError> {
        let query = SearchQuery::new("test", 1);
        self.search(query).await.map(|_| ())
    }
}

#[async_trait]
impl ImageSearchProvider for BraveSearchProvider {
    async fn search_images(&self, query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError> {
        let url = format!("{}/images/search", self.base_url);

        debug!(query = %query.query, num_results = query.num_results, "Brave image search");

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .query(&[
                ("q", query.query.as_str()),
                ("count", &query.num_results.to_string()),
            ]);

        if let Some(ref lang) = query.language {
            request = request.query(&[("search_lang", lang.as_str())]);
        }
        if let Some(ref country) = query.country {
            request = request.query(&[("country", country.as_str())]);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_connect() {
                SearchError::ConnectionError(e.to_string())
            } else {
                SearchError::RequestFailed(e.to_string())
            }
        })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SearchError::AuthError("Invalid Brave API key".to_string()));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());
            return Err(SearchError::RateLimited { retry_after_secs: retry_after });
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(SearchError::RequestFailed(
                format!("Brave Images API error {}: {}", status, body)
            ));
        }

        let brave_response: BraveImageResponse = response.json().await.map_err(|e| {
            SearchError::ParseError(format!("Failed to parse Brave image response: {}", e))
        })?;

        let results = brave_response
            .results
            .unwrap_or_default()
            .into_iter()
            .map(|r| ImageSearchResult {
                image_url: r.url.clone(),
                thumbnail_url: r.thumbnail.and_then(|t| t.src).unwrap_or_else(|| r.url),
                title: r.title,
                source_url: r.source.unwrap_or_default(),
                width: r.properties.as_ref().and_then(|p| p.width),
                height: r.properties.as_ref().and_then(|p| p.height),
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brave_provider_creation() {
        let provider = BraveSearchProvider::new("test-key");
        assert_eq!(provider.provider_name(), "brave");
        assert_eq!(provider.api_key, "test-key");
    }

    #[test]
    fn test_brave_response_parsing() {
        let json = r#"{
            "web": {
                "results": [
                    {
                        "title": "Rust Lang",
                        "url": "https://rust-lang.org",
                        "description": "A systems language"
                    }
                ]
            }
        }"#;

        let response: BraveResponse = serde_json::from_str(json).unwrap();
        let results = response.web.unwrap().results;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Lang");
    }

    #[test]
    fn test_brave_response_no_web() {
        let json = r#"{}"#;
        let response: BraveResponse = serde_json::from_str(json).unwrap();
        assert!(response.web.is_none());
    }

    #[test]
    fn test_brave_image_response_parsing() {
        let json = r#"{
            "results": [
                {
                    "title": "Cute Cat",
                    "url": "https://example.com/cat.jpg",
                    "source": "https://example.com/cats",
                    "thumbnail": { "src": "https://example.com/cat_thumb.jpg" },
                    "properties": { "width": 1920, "height": 1080 }
                }
            ]
        }"#;

        let response: BraveImageResponse = serde_json::from_str(json).unwrap();
        let results = response.results.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Cute Cat");
        assert_eq!(results[0].url, "https://example.com/cat.jpg");
        assert_eq!(results[0].properties.as_ref().unwrap().width, Some(1920));
    }

    #[test]
    fn test_brave_image_response_empty() {
        let json = r#"{ "results": [] }"#;
        let response: BraveImageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.results.unwrap().len(), 0);
    }
}
