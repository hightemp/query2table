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

/// Brave image result: `url` is the page that shows the image, `source` is that page's
/// domain, and `properties.url` is the image file itself.
#[derive(Debug, Deserialize)]
struct BraveImageResult {
    title: String,
    url: String,
    thumbnail: Option<BraveThumbnail>,
    properties: Option<BraveImageProperties>,
}

#[derive(Debug, Deserialize)]
struct BraveThumbnail {
    src: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BraveImageProperties {
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

impl BraveSearchProvider {
    // Endpoint-specific public contracts, rather than the shared Results per Query setting:
    // https://api-dashboard.search.brave.com/api-reference/web/search/get
    // https://api-dashboard.search.brave.com/documentation/services/image-search
    const MAX_WEB_RESULTS: u32 = 20;
    const MAX_IMAGE_RESULTS: u32 = 200;

    fn result_count(requested: u32, maximum: u32, mode: &str) -> u32 {
        let effective = requested.clamp(1, maximum);
        if effective != requested {
            debug!(requested, effective, mode, "[FIX:search-count] Limited Brave result count");
        }
        effective
    }

    pub fn new(api_key: impl Into<String>) -> Self {
        let client = crate::providers::http::apply_proxy(Client::builder())
            .build()
            .expect("Failed to build Brave HTTP client");
        Self {
            client,
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
        let count = Self::result_count(query.num_results, Self::MAX_WEB_RESULTS, "web");

        debug!(query = %query.query, num_results = query.num_results, "Brave search");

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .query(&[
                ("q", query.query.as_str()),
                ("count", &count.to_string()),
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
        let count = Self::result_count(query.num_results, Self::MAX_IMAGE_RESULTS, "images");

        debug!(query = %query.query, num_results = query.num_results, "Brave image search");

        let mut request = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip")
            .header("X-Subscription-Token", &self.api_key)
            .query(&[
                ("q", query.query.as_str()),
                ("count", &count.to_string()),
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
            .filter_map(|r| {
                let image_url = r
                    .properties
                    .as_ref()
                    .and_then(|p| p.url.clone())
                    .or_else(|| r.thumbnail.as_ref().and_then(|t| t.src.clone()))?;
                Some(ImageSearchResult {
                    thumbnail_url: r
                        .thumbnail
                        .and_then(|t| t.src)
                        .unwrap_or_else(|| image_url.clone()),
                    image_url,
                    title: r.title,
                    source_url: r.url,
                    width: r.properties.as_ref().and_then(|p| p.width),
                    height: r.properties.as_ref().and_then(|p| p.height),
                })
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn web_and_image_counts_are_bounded_in_actual_http_requests() {
        use wiremock::{matchers::{method, path, query_param}, Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let mut provider = BraveSearchProvider::new("test-key").with_base_url(server.uri());
        provider.client = Client::builder().no_proxy().build().unwrap();
        let cases = [
            (0, 1, 1), (1, 1, 1), (10, 10, 10), (20, 20, 20), (21, 20, 21),
            (50, 20, 50), (100, 20, 100), (200, 20, 200), (201, 20, 200),
            (u32::MAX, 20, 200),
        ];
        for (requested, web_count, image_count) in cases {
            let query = format!("fixture-{requested}");
            for (endpoint, count) in [("/web/search", web_count), ("/images/search", image_count)] {
                Mock::given(method("GET")).and(path(endpoint))
                    .and(query_param("q", &query)).and(query_param("count", count.to_string()))
                    .respond_with(ResponseTemplate::new(200).set_body_json(
                        serde_json::json!({"web":{"results":[]},"results":[]})
                    )).expect(1).mount(&server).await;
            }
            provider.search(SearchQuery::new(&query, requested)).await.unwrap();
            provider.search_images(SearchQuery::new(&query, requested)).await.unwrap();
        }
        assert_eq!(server.received_requests().await.unwrap().len(), cases.len() * 2);
    }

    #[tokio::test]
    async fn image_results_link_the_image_file_and_the_page_showing_it() {
        use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        let mut provider = BraveSearchProvider::new("test-key").with_base_url(server.uri());
        provider.client = Client::builder().no_proxy().build().unwrap();
        Mock::given(method("GET")).and(path("/images/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"results": [
                {
                    "title": "Tower",
                    "url": "https://news.example/article",
                    "source": "news.example",
                    "thumbnail": {"src": "https://imgs.search.brave.com/thumb"},
                    "properties": {"url": "https://cdn.example/tower.jpg", "width": 1600, "height": 900}
                },
                {"title": "Thumbnail only", "url": "https://other.example/", "thumbnail": {"src": "https://imgs.search.brave.com/t2"}},
                {"title": "Nothing to show", "url": "https://empty.example/"}
            ]}))).mount(&server).await;

        let results = provider.search_images(SearchQuery::new("tower", 3)).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].image_url, "https://cdn.example/tower.jpg");
        assert_eq!(results[0].source_url, "https://news.example/article");
        assert_eq!(results[0].thumbnail_url, "https://imgs.search.brave.com/thumb");
        assert_eq!((results[0].width, results[0].height), (Some(1600), Some(900)));
        assert_eq!(results[1].image_url, "https://imgs.search.brave.com/t2");
        assert_eq!(results[1].source_url, "https://other.example/");
    }

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
