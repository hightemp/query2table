use reqwest::Client;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use super::client::FetchError;

/// Cached robots.txt entry.
struct RobotsEntry {
    disallowed_paths: Vec<String>,
    fetched_at: Instant,
}

/// Checks robots.txt for crawl permissions with in-memory caching.
pub struct RobotsChecker {
    client: Client,
    cache: RwLock<HashMap<String, RobotsEntry>>,
    cache_ttl: Duration,
    user_agent_token: String,
}

impl RobotsChecker {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .expect("Failed to build robots.txt client"),
            cache: RwLock::new(HashMap::new()),
            cache_ttl: Duration::from_secs(3600), // 1 hour cache
            user_agent_token: "Query2Table".to_string(),
        }
    }

    /// Check if fetching the given URL is allowed by robots.txt.
    pub async fn is_allowed(&self, url: &str) -> Result<bool, FetchError> {
        let parsed = url::Url::parse(url).map_err(|e| {
            FetchError::RequestFailed(format!("Invalid URL: {}", e))
        })?;

        let origin = format!(
            "{}://{}",
            parsed.scheme(),
            parsed.host_str().unwrap_or("unknown")
        );
        let path = parsed.path();

        // Check cache first
        if let Some(allowed) = self.check_cache(&origin, path) {
            return Ok(allowed);
        }

        // Fetch robots.txt
        let robots_url = format!("{}/robots.txt", origin);
        debug!(url = %robots_url, "Fetching robots.txt");

        let disallowed = match self.client.get(&robots_url).send().await {
            Ok(response) if response.status().is_success() => {
                let body = response.text().await.unwrap_or_default();
                self.parse_disallowed(&body)
            }
            Ok(_) => {
                // Non-success status (404 etc.) = no restrictions
                Vec::new()
            }
            Err(e) => {
                warn!(url = %robots_url, error = %e, "Failed to fetch robots.txt, allowing access");
                Vec::new()
            }
        };

        let allowed = !self.path_blocked(path, &disallowed);

        // Update cache
        if let Ok(mut cache) = self.cache.write() {
            cache.insert(origin, RobotsEntry {
                disallowed_paths: disallowed,
                fetched_at: Instant::now(),
            });
        }

        Ok(allowed)
    }

    fn check_cache(&self, origin: &str, path: &str) -> Option<bool> {
        let cache = self.cache.read().ok()?;
        let entry = cache.get(origin)?;

        if entry.fetched_at.elapsed() > self.cache_ttl {
            return None; // Expired
        }

        Some(!self.path_blocked(path, &entry.disallowed_paths))
    }

    /// Parse robots.txt content for disallowed paths applicable to our user agent.
    fn parse_disallowed(&self, content: &str) -> Vec<String> {
        let mut disallowed = Vec::new();
        let mut applies = false; // whether current user-agent block applies to us
        let ua_lower = self.user_agent_token.to_lowercase();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(agent) = line.strip_prefix("User-agent:").or_else(|| line.strip_prefix("user-agent:")) {
                let agent = agent.trim().to_lowercase();
                applies = agent == "*" || agent == ua_lower;
            } else if applies {
                if let Some(path) = line.strip_prefix("Disallow:").or_else(|| line.strip_prefix("disallow:")) {
                    let path = path.trim();
                    if !path.is_empty() {
                        disallowed.push(path.to_string());
                    }
                }
            }
        }

        disallowed
    }

    fn path_blocked(&self, path: &str, disallowed: &[String]) -> bool {
        disallowed.iter().any(|d| path.starts_with(d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_disallowed_wildcard() {
        let checker = RobotsChecker::new();
        let robots = r#"
User-agent: *
Disallow: /admin
Disallow: /private/
"#;
        let paths = checker.parse_disallowed(robots);
        assert_eq!(paths, vec!["/admin", "/private/"]);
    }

    #[test]
    fn test_parse_disallowed_specific_agent() {
        let checker = RobotsChecker::new();
        let robots = r#"
User-agent: Query2Table
Disallow: /api

User-agent: Googlebot
Disallow: /secret
"#;
        let paths = checker.parse_disallowed(robots);
        // Should only pick up Query2Table and ignore Googlebot
        assert_eq!(paths, vec!["/api"]);
    }

    #[test]
    fn test_parse_empty_disallow() {
        let checker = RobotsChecker::new();
        let robots = r#"
User-agent: *
Disallow:
"#;
        let paths = checker.parse_disallowed(robots);
        assert!(paths.is_empty());
    }

    #[test]
    fn test_path_blocked() {
        let checker = RobotsChecker::new();
        let disallowed = vec!["/admin".to_string(), "/private/".to_string()];
        assert!(checker.path_blocked("/admin/users", &disallowed));
        assert!(checker.path_blocked("/private/data", &disallowed));
        assert!(!checker.path_blocked("/public/page", &disallowed));
    }

    #[test]
    fn test_path_not_blocked_empty() {
        let checker = RobotsChecker::new();
        assert!(!checker.path_blocked("/anything", &[]));
    }
}
