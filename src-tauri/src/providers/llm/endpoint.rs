use super::types::LlmError;

/// Normalize an API base URL, preserving custom path prefixes.
pub(super) fn normalize_base_url(value: &str, default_path: &str) -> Result<String, LlmError> {
    let mut url = url::Url::parse(value.trim())
        .map_err(|_| LlmError::NotConfigured("LLM URL must be an absolute HTTP(S) URL".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(LlmError::NotConfigured(
            "LLM URL must use HTTP(S), without credentials, query parameters, or fragments".into(),
        ));
    }
    if url.path().trim_matches('/').is_empty() {
        url.set_path(default_path);
    }
    Ok(url.as_str().trim_end_matches('/').to_string())
}

pub(super) fn status_error(response: &reqwest::Response) -> Option<LlmError> {
    match response.status() {
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            Some(LlmError::AuthError)
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            let seconds = response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(5);
            Some(LlmError::RateLimited {
                retry_after_ms: seconds.saturating_mul(1000),
            })
        }
        _ => None,
    }
}
