use std::time::Duration;
use tracing::warn;

/// Configuration for retry with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not counting the initial attempt).
    pub max_retries: u32,
    /// Initial delay before the first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Backoff multiplier applied after each retry.
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// Outcome of a retryable operation.
pub enum RetryAction {
    /// Operation succeeded — return the value.
    Success,
    /// Operation failed but is retryable (e.g. rate limit, transient error).
    Retry,
    /// Operation failed with a permanent error — do not retry.
    Fail,
}

/// Execute an async closure with exponential backoff retry.
///
/// The `classify` function inspects the result to decide whether to retry.
/// If a `retry_after_hint` is provided by the operation, it overrides the backoff delay.
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    label: &str,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = (Result<T, E>, RetryAction, Option<Duration>)>,
{
    let mut delay = config.initial_delay;

    for attempt in 0..=config.max_retries {
        let (result, action, retry_hint) = operation().await;

        match action {
            RetryAction::Success => return result,
            RetryAction::Fail => return result,
            RetryAction::Retry => {
                if attempt == config.max_retries {
                    warn!(label, attempt, "Max retries reached, giving up");
                    return result;
                }

                let wait = retry_hint.unwrap_or(delay).min(config.max_delay);
                warn!(label, attempt = attempt + 1, max = config.max_retries, wait_secs = wait.as_secs_f64(), "Retrying after error");
                tokio::time::sleep(wait).await;

                // Exponential backoff for next iteration
                delay = Duration::from_secs_f64(
                    (delay.as_secs_f64() * config.multiplier).min(config.max_delay.as_secs_f64()),
                );
            }
        }
    }

    // Should not be reached, but just in case
    operation().await.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        };

        let result: Result<&str, &str> = retry_with_backoff(&config, "test", || async {
            (Ok("ok"), RetryAction::Success, None)
        }).await;

        assert_eq!(result.unwrap(), "ok");
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_third_attempt() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let result: Result<String, String> = retry_with_backoff(&config, "test", move || {
            let c = c.clone();
            async move {
                let attempt = c.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    (Err("transient".to_string()), RetryAction::Retry, None)
                } else {
                    (Ok("success".to_string()), RetryAction::Success, None)
                }
            }
        }).await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            multiplier: 2.0,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let result: Result<String, String> = retry_with_backoff(&config, "test", move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                (Err("still failing".to_string()), RetryAction::Retry, None)
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3); // initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_permanent_failure_no_retry() {
        let config = RetryConfig::default();

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let result: Result<String, String> = retry_with_backoff(&config, "test", move || {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                (Err("auth failed".to_string()), RetryAction::Fail, None)
            }
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // No retry for permanent failure
    }

    #[tokio::test]
    async fn test_retry_with_hint() {
        let config = RetryConfig {
            max_retries: 1,
            initial_delay: Duration::from_secs(60), // Very long default
            max_delay: Duration::from_secs(120),
            multiplier: 2.0,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let c = counter.clone();

        let start = tokio::time::Instant::now();
        let _result: Result<String, String> = retry_with_backoff(&config, "test", move || {
            let c = c.clone();
            async move {
                let attempt = c.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    // Hint overrides the 60s default
                    (Err("rate limited".to_string()), RetryAction::Retry, Some(Duration::from_millis(50)))
                } else {
                    (Ok("ok".to_string()), RetryAction::Success, None)
                }
            }
        }).await;

        // Should be much less than 60s (the default delay)
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
