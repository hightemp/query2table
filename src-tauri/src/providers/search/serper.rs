use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::types::*;

/// Serper.dev Google Search API client.
pub struct SerperProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct SerperRequest {
    q: String,
    num: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    gl: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hl: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SerperResponse {
    organic: Option<Vec<SerperResult>>,
}

#[derive(Debug, Deserialize)]
struct SerperResult {
    title: String,
    link: String,
    snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerperImageResponse {
    images: Option<Vec<SerperImageResult>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SerperImageResult {
    title: String,
    image_url: String,
    image_width: Option<u32>,
    image_height: Option<u32>,
    thumbnail_url: Option<String>,
    source: Option<String>,
    link: Option<String>,
}

impl SerperProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            base_url: "https://google.serper.dev".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl SearchProvider for SerperProvider {
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let url = format!("{}/search", self.base_url);

        debug!(query = %query.query, num_results = query.num_results, "Serper search");

        let body = SerperRequest {
            q: query.query,
            num: query.num_results,
            gl: query.country,
            hl: query.language,
        };

        let response = self.client
            .post(&url)
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    SearchError::ConnectionError(e.to_string())
                } else {
                    SearchError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(SearchError::AuthError("Invalid Serper API key".to_string()));
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
                format!("Serper API error {}: {}", status, body)
            ));
        }

        let serper_response: SerperResponse = response.json().await.map_err(|e| {
            SearchError::ParseError(format!("Failed to parse Serper response: {}", e))
        })?;

        let results = serper_response
            .organic
            .unwrap_or_default()
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.link,
                snippet: r.snippet.unwrap_or_default(),
            })
            .collect();

        Ok(results)
    }

    fn provider_name(&self) -> &str {
        "serper"
    }

    async fn health_check(&self) -> Result<(), SearchError> {
        let query = SearchQuery::new("test", 1);
        self.search(query).await.map(|_| ())
    }
}

#[async_trait]
impl ImageSearchProvider for SerperProvider {
    async fn search_images(&self, query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError> {
        let url = format!("{}/images", self.base_url);

        debug!(query = %query.query, num_results = query.num_results, "Serper image search");

        let body = SerperRequest {
            q: query.query,
            num: query.num_results,
            gl: query.country,
            hl: query.language,
        };

        let response = self.client
            .post(&url)
            .header("X-API-KEY", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    SearchError::ConnectionError(e.to_string())
                } else {
                    SearchError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(SearchError::AuthError("Invalid Serper API key".to_string()));
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
            let body_text = response.text().await.unwrap_or_default();
            return Err(SearchError::RequestFailed(
                format!("Serper Images API error {}: {}", status, body_text)
            ));
        }

        let serper_response: SerperImageResponse = response.json().await.map_err(|e| {
            SearchError::ParseError(format!("Failed to parse Serper image response: {}", e))
        })?;

        let results = serper_response
            .images
            .unwrap_or_default()
            .into_iter()
            .map(|r| ImageSearchResult {
                image_url: r.image_url.clone(),
                thumbnail_url: r.thumbnail_url.unwrap_or_else(|| r.image_url),
                title: r.title,
                source_url: r.link.or(r.source).unwrap_or_default(),
                width: r.image_width,
                height: r.image_height,
            })
            .collect();

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serper_provider_creation() {
        let provider = SerperProvider::new("test-key");
        assert_eq!(provider.provider_name(), "serper");
    }

    #[test]
    fn test_serper_request_serialization() {
        let req = SerperRequest {
            q: "rust lang".to_string(),
            num: 10,
            gl: Some("us".to_string()),
            hl: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("rust lang"));
        assert!(json.contains("\"gl\":\"us\""));
        assert!(!json.contains("hl"));
    }

    #[test]
    fn test_serper_response_parsing() {
        let json = r#"{
            "organic": [
                {
                    "title": "Rust Programming",
                    "link": "https://rust-lang.org",
                    "snippet": "Empowering everyone"
                }
            ]
        }"#;

        let response: SerperResponse = serde_json::from_str(json).unwrap();
        let results = response.organic.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].link, "https://rust-lang.org");
    }

    #[test]
    fn test_serper_response_no_organic() {
        let json = r#"{}"#;
        let response: SerperResponse = serde_json::from_str(json).unwrap();
        assert!(response.organic.is_none());
    }

    #[test]
    fn test_serper_image_response_parsing() {
        let json = r#"{
            "images": [
                {
                    "title": "Beautiful Sunset",
                    "imageUrl": "https://example.com/sunset.jpg",
                    "imageWidth": 1920,
                    "imageHeight": 1080,
                    "thumbnailUrl": "https://example.com/sunset_thumb.jpg",
                    "source": "Example Site",
                    "link": "https://example.com/sunsets"
                }
            ]
        }"#;

        let response: SerperImageResponse = serde_json::from_str(json).unwrap();
        let results = response.images.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Beautiful Sunset");
        assert_eq!(results[0].image_url, "https://example.com/sunset.jpg");
        assert_eq!(results[0].image_width, Some(1920));
    }

    #[test]
    fn test_serper_image_response_empty() {
        let json = r#"{ "images": [] }"#;
        let response: SerperImageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.images.unwrap().len(), 0);
    }
}
