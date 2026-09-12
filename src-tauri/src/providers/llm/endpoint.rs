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

/// Classify transport faults without retaining URL credentials or proxy details.
pub(super) fn transport_error(error: reqwest::Error) -> LlmError {
    if error.is_timeout() {
        LlmError::Timeout
    } else {
        LlmError::ConnectionError("Could not reach the LLM provider. Check the server URL, proxy, and network connection.".into())
    }
}

/// Inspect both HTTP failures and successful responses containing an error
/// envelope. Provider text is used for classification only, never echoed: some
/// servers include request content, credentials, or raw reasoning in errors.
pub(super) async fn response_json(
    response: reqwest::Response,
    model: &str,
) -> Result<serde_json::Value, LlmError> {
    let status = response.status().as_u16();
    let retry_after_ms = response
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(5)
        .saturating_mul(1000);
    let bytes = response.bytes().await.map_err(transport_error)?;
    let parsed = serde_json::from_slice::<serde_json::Value>(&bytes);
    let value = parsed.as_ref().ok();
    let envelope = value
        .and_then(|v| v.get("error").or_else(|| v.pointer("/choices/0/error")))
        .filter(|error| !error.is_null());
    if !(200..300).contains(&status) || envelope.is_some() {
        return Err(classify_error(status, envelope, model, retry_after_ms));
    }
    parsed.map_err(|_| {
        LlmError::ParseError(
            "The provider returned invalid JSON instead of a completion response.".into(),
        )
    })
}

fn classify_error(
    status: u16,
    envelope: Option<&serde_json::Value>,
    model: &str,
    retry_after_ms: u64,
) -> LlmError {
    let details = envelope
        .map(serde_json::Value::to_string)
        .unwrap_or_default()
        .to_lowercase();
    let code_status = envelope
        .and_then(|error| error.get("code"))
        .and_then(|code| code.as_u64().or_else(|| code.as_str()?.parse().ok()))
        .and_then(|code| u16::try_from(code).ok());
    let status = if (200..300).contains(&status) {
        code_status.unwrap_or(status)
    } else {
        status
    };
    if status == 401 {
        return LlmError::AuthError;
    }
    if status == 403 {
        return LlmError::AccessDenied;
    }
    if status == 402
        || [
            "insufficient_quota",
            "quota_exceeded",
            "exceeded your current quota",
            "credit",
            "billing",
            "payment_required",
            "usage limit",
            "insufficient balance",
            "insufficient_balance",
            "daily limit",
            "weekly limit",
            "monthly limit",
        ]
        .iter()
        .any(|term| details.contains(term))
    {
        return LlmError::QuotaExceeded;
    }
    if details.contains("invalid_api_key") || details.contains("authentication_error") {
        return LlmError::AuthError;
    }
    if details.contains("access_denied") || details.contains("permission_denied") {
        return LlmError::AccessDenied;
    }
    if [
        "context_length",
        "context length",
        "context window",
        "context limit",
        "prompt is too long",
        "input is too long",
        "prompt too long",
        "too many input tokens",
    ]
    .iter()
    .any(|term| details.contains(term))
    {
        return LlmError::ContextLimit;
    }
    if status == 429 || details.contains("rate_limit") || details.contains("rate limit") {
        return LlmError::RateLimited { retry_after_ms };
    }
    if status == 408 || status == 504 || details.contains("request_timeout") {
        return LlmError::Timeout;
    }
    if [
        "unsupported",
        "not supported",
        "unknown parameter",
        "unrecognized request argument",
        "invalid parameter",
        "invalid value",
        "must be one of",
    ]
    .iter()
    .any(|term| details.contains(term))
    {
        return LlmError::UnsupportedSetting("The provider rejected a generation setting. Try Provider default for reasoning effort, or check which parameters this model supports.".into());
    }
    if status == 404
        || details.contains("model_not_found")
        || details.contains("no such model")
        || details.contains("not found")
        || details.contains("no model")
    {
        return LlmError::ModelNotFound(model.chars().take(200).collect());
    }
    LlmError::ProviderError {
        message: if (200..300).contains(&status) {
            "The provider returned an error envelope instead of an answer. Check model availability and provider status.".into()
        } else {
            format!("The provider returned HTTP {status}. Check model availability and request settings.")
        },
        retryable: status >= 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn transport_timeout_is_distinct_from_connection_failure() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
            .mount(&server)
            .await;
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(10))
            .build()
            .unwrap();
        let error = client.get(server.uri()).send().await.unwrap_err();
        assert_eq!(transport_error(error).code(), "timeout");
    }

    #[tokio::test]
    async fn transport_diagnostics_do_not_retain_request_urls_or_credentials() {
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let error = client
            .get("http://127.0.0.1:0/?api_key=PRIVATE_TEST_KEY")
            .send()
            .await
            .unwrap_err();
        let error = transport_error(error);
        assert_eq!(error.code(), "connection");
        assert!(!error.to_string().contains("PRIVATE_TEST_KEY"));
    }
}
