use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use thiserror::Error;
use tracing::debug;

use super::proxy::apply_proxy;
use super::rate_limiter::RateLimiter;

/// Error types for HTTP fetching.
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Timeout fetching {url}")]
    Timeout { url: String },

    #[error("HTTP {status} for {url}")]
    HttpStatus { status: u16, url: String },

    #[error("Blocked by robots.txt: {0}")]
    RobotsBlocked(String),

    #[error("Content too large: {size} bytes (max {max})")]
    ContentTooLarge { size: u64, max: u64 },
}

/// Fetched page content.
#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub url: String,
    pub status: u16,
    pub body: String,
    pub content_type: Option<String>,
    /// Raw bytes for binary content (e.g. PDF). Empty for text content.
    pub body_bytes: Vec<u8>,
}

impl FetchedPage {
    /// Returns true if content-type indicates a PDF document.
    pub fn is_pdf(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.to_lowercase().contains("application/pdf"))
            .unwrap_or(false)
            || self.url.to_lowercase().ends_with(".pdf")
    }
}

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (compatible; Query2Table/1.0; +https://github.com/query2table)",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:128.0) Gecko/20100101 Firefox/128.0",
];

/// HTTP fetcher with rate limiting, User-Agent rotation, and size limits.
pub struct HttpFetcher {
    client: Client,
    rate_limiter: RateLimiter,
    max_body_bytes: u64,
    ua_index: std::sync::atomic::AtomicUsize,
}

impl HttpFetcher {
    pub fn new(rate_limiter: RateLimiter) -> Self {
        let builder = ClientBuilder::new()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true);
        let client = apply_proxy(builder)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            rate_limiter,
            max_body_bytes: 5 * 1024 * 1024, // 5 MB
            ua_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Create a fetcher that bypasses system proxy settings.
    /// Useful for tests that use local mock servers.
    pub fn new_no_proxy(rate_limiter: RateLimiter) -> Self {
        let client = ClientBuilder::new()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(8))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            .no_proxy()
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            rate_limiter,
            max_body_bytes: 5 * 1024 * 1024,
            ua_index: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Set max body size in bytes.
    pub fn with_max_body_bytes(mut self, max: u64) -> Self {
        self.max_body_bytes = max;
        self
    }

    /// Fetch a URL with rate limiting and size protection.
    pub async fn fetch(&self, url: &str) -> Result<FetchedPage, FetchError> {
        let parsed = url::Url::parse(url).map_err(|e| {
            FetchError::RequestFailed(format!("Invalid URL: {}", e))
        })?;

        let domain = parsed.host_str().unwrap_or("unknown").to_string();

        // Wait for rate limiter
        self.rate_limiter.wait(&domain).await;

        let ua = self.next_user_agent();
        debug!(url = %url, user_agent = %ua, "Fetching page");

        let response = self.client
            .get(url)
            .header("User-Agent", ua)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.5")
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    FetchError::Timeout { url: url.to_string() }
                } else if e.is_connect() {
                    FetchError::ConnectionError(e.to_string())
                } else {
                    FetchError::RequestFailed(e.to_string())
                }
            })?;

        let status = response.status();
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Check content-length if available
        if let Some(len) = response.content_length() {
            if len > self.max_body_bytes {
                return Err(FetchError::ContentTooLarge {
                    size: len,
                    max: self.max_body_bytes,
                });
            }
        }

        if !status.is_success() {
            return Err(FetchError::HttpStatus {
                status: status.as_u16(),
                url: url.to_string(),
            });
        }

        let is_pdf = content_type
            .as_ref()
            .map(|ct| ct.to_lowercase().contains("application/pdf"))
            .unwrap_or(false)
            || url.to_lowercase().ends_with(".pdf");

        let raw_bytes = response.bytes().await.map_err(|e| {
            FetchError::RequestFailed(format!("Failed to read response body: {}", e))
        })?;

        if raw_bytes.len() as u64 > self.max_body_bytes {
            return Err(FetchError::ContentTooLarge {
                size: raw_bytes.len() as u64,
                max: self.max_body_bytes,
            });
        }

        let (body, body_bytes) = if is_pdf {
            (String::new(), raw_bytes.to_vec())
        } else {
            (decode_html_body(&raw_bytes, content_type.as_deref()), Vec::new())
        };

        Ok(FetchedPage {
            url: url.to_string(),
            status: status.as_u16(),
            body,
            content_type,
            body_bytes,
        })
    }

    fn next_user_agent(&self) -> &'static str {
        let idx = self.ua_index.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        USER_AGENTS[idx % USER_AGENTS.len()]
    }
}

/// Decode raw HTML bytes into a `String`, detecting the character encoding.
///
/// Many non-English sites (especially Chinese, Japanese, Korean, Cyrillic and
/// older Western pages) are *not* UTF-8 — they use GBK/GB2312, Big5,
/// Shift_JIS, EUC-KR, windows-1251, ISO-8859-*, or UTF-16. Blindly running
/// `from_utf8_lossy` on those bytes produces replacement characters (`�`) and
/// garbled text. We determine the encoding in priority order:
///
/// 1. A BOM at the start of the document (UTF-8 / UTF-16 LE / UTF-16 BE).
/// 2. The `charset` parameter of the `Content-Type` HTTP header.
/// 3. A `<meta charset=...>` / `<meta http-equiv="Content-Type">` tag in the
///    first part of the HTML.
/// 4. UTF-8 as the default.
fn decode_html_body(bytes: &[u8], content_type: Option<&str>) -> String {
    // 1. BOM sniffing — authoritative when present.
    if let Some((enc, bom_len)) = encoding_rs::Encoding::for_bom(bytes) {
        let (text, _, _) = enc.decode(&bytes[bom_len..]);
        return text.into_owned();
    }

    // 2. Charset from the Content-Type header.
    let mut encoding: Option<&'static encoding_rs::Encoding> = content_type
        .and_then(charset_from_content_type)
        .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()));

    // 3. Charset declared inside the HTML <meta> tags.
    if encoding.is_none() {
        encoding = charset_from_meta(bytes)
            .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()));
    }

    // 4. Default to UTF-8.
    let enc = encoding.unwrap_or(encoding_rs::UTF_8);
    let (text, _, _) = enc.decode(bytes);
    text.into_owned()
}

/// Extract the `charset=` value from a Content-Type header value.
fn charset_from_content_type(content_type: &str) -> Option<String> {
    for part in content_type.split(';') {
        let part = part.trim();
        if let Some(rest) = part
            .strip_prefix("charset=")
            .or_else(|| part.strip_prefix("CHARSET="))
            .or_else(|| {
                // Case-insensitive prefix match for "charset=".
                if part.len() >= 8 && part[..8].eq_ignore_ascii_case("charset=") {
                    Some(&part[8..])
                } else {
                    None
                }
            })
        {
            let val = rest.trim().trim_matches('"').trim_matches('\'');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Scan the first portion of an HTML document for a declared charset.
fn charset_from_meta(bytes: &[u8]) -> Option<String> {
    // Only the document head matters; scan at most the first 4 KB as ASCII.
    let limit = bytes.len().min(4096);
    let head = String::from_utf8_lossy(&bytes[..limit]).to_ascii_lowercase();

    // <meta charset="...">
    if let Some(pos) = head.find("charset") {
        let after = &head[pos + "charset".len()..];
        let after = after.trim_start_matches(|c: char| c == '=' || c == ' ' || c == '"' || c == '\'');
        let val: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !val.is_empty() {
            return Some(val);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_utf8_default() {
        let body = decode_html_body("héllo 你好".as_bytes(), Some("text/html"));
        assert_eq!(body, "héllo 你好");
    }

    #[test]
    fn decode_gbk_from_header() {
        // "你好" encoded in GBK.
        let (gbk_bytes, _, _) = encoding_rs::GBK.encode("你好世界");
        let body = decode_html_body(&gbk_bytes, Some("text/html; charset=gbk"));
        assert_eq!(body, "你好世界");
    }

    #[test]
    fn decode_shift_jis_from_meta() {
        let (sjis_body, _, _) = encoding_rs::SHIFT_JIS.encode("こんにちは");
        let mut bytes =
            b"<html><head><meta charset=\"shift_jis\"></head><body>".to_vec();
        bytes.extend_from_slice(&sjis_body);
        let body = decode_html_body(&bytes, None);
        assert!(body.contains("こんにちは"));
    }

    #[test]
    fn decode_utf16_le_bom() {
        let mut bytes = vec![0xFF, 0xFE]; // UTF-16 LE BOM
        for unit in "héllo".encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        let body = decode_html_body(&bytes, None);
        assert_eq!(body, "héllo");
    }

    #[test]
    fn charset_from_content_type_variants() {
        assert_eq!(
            charset_from_content_type("text/html; charset=UTF-8").as_deref(),
            Some("UTF-8")
        );
        assert_eq!(
            charset_from_content_type("text/html;charset=gbk").as_deref(),
            Some("gbk")
        );
        assert_eq!(charset_from_content_type("text/html").as_deref(), None);
    }

    #[test]
    fn test_user_agent_rotation() {
        let rate_limiter = RateLimiter::new(Duration::from_millis(100));
        let fetcher = HttpFetcher::new(rate_limiter);

        let ua1 = fetcher.next_user_agent();
        let ua2 = fetcher.next_user_agent();
        let ua3 = fetcher.next_user_agent();
        let ua4 = fetcher.next_user_agent();

        assert_eq!(ua1, USER_AGENTS[0]);
        assert_eq!(ua2, USER_AGENTS[1]);
        assert_eq!(ua3, USER_AGENTS[2]);
        // wraps around
        assert_eq!(ua4, USER_AGENTS[0]);
    }

    #[test]
    fn test_fetched_page() {
        let page = FetchedPage {
            url: "https://example.com".to_string(),
            status: 200,
            body: "<html>test</html>".to_string(),
            content_type: Some("text/html".to_string()),
            body_bytes: Vec::new(),
        };
        assert_eq!(page.status, 200);
        assert!(!page.is_pdf());
    }

    #[test]
    fn test_fetched_page_pdf_detection() {
        let page_by_ct = FetchedPage {
            url: "https://example.com/doc".to_string(),
            status: 200,
            body: String::new(),
            content_type: Some("application/pdf".to_string()),
            body_bytes: vec![0x25, 0x50, 0x44, 0x46], // %PDF
        };
        assert!(page_by_ct.is_pdf());

        let page_by_url = FetchedPage {
            url: "https://example.com/doc.pdf".to_string(),
            status: 200,
            body: String::new(),
            content_type: None,
            body_bytes: vec![0x25, 0x50, 0x44, 0x46],
        };
        assert!(page_by_url.is_pdf());
    }

    #[test]
    fn test_max_body_bytes_config() {
        let rate_limiter = RateLimiter::new(Duration::from_millis(100));
        let fetcher = HttpFetcher::new(rate_limiter).with_max_body_bytes(1024);
        assert_eq!(fetcher.max_body_bytes, 1024);
    }
}
