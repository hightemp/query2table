use governor::{Quota, RateLimiter as GovLimiter, clock::DefaultClock, state::keyed::DefaultKeyedStateStore};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

type KeyedLimiter = GovLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

/// Per-domain rate limiter using the governor crate (token bucket).
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<KeyedLimiter>,
}

impl RateLimiter {
    /// Create a rate limiter with the given minimum interval between requests per domain.
    pub fn new(interval: Duration) -> Self {
        // Convert interval to a rate: 1 request per interval
        let period = interval;
        let quota = Quota::with_period(period)
            .expect("Rate limit period must be non-zero")
            .allow_burst(NonZeroU32::new(1).unwrap());

        Self {
            limiter: Arc::new(GovLimiter::keyed(quota)),
        }
    }

    /// Default rate limiter: 1 request per 2 seconds per domain.
    pub fn default_web() -> Self {
        Self::new(Duration::from_secs(2))
    }

    /// Wait until a request to the given domain is allowed.
    pub async fn wait(&self, domain: &str) {
        debug!(domain = %domain, "Rate limiter: waiting for slot");
        self.limiter
            .until_key_ready(&domain.to_string())
            .await;
    }

    /// Try to acquire a slot immediately. Returns true if allowed.
    pub fn try_acquire(&self, domain: &str) -> bool {
        self.limiter
            .check_key(&domain.to_string())
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(Duration::from_secs(1));
        // First request should be allowed
        assert!(limiter.try_acquire("example.com"));
    }

    #[test]
    fn test_rate_limiter_per_domain() {
        let limiter = RateLimiter::new(Duration::from_secs(1));
        // Different domains should be independent
        assert!(limiter.try_acquire("a.com"));
        assert!(limiter.try_acquire("b.com"));
    }

    #[test]
    fn test_rate_limiter_blocks_rapid() {
        let limiter = RateLimiter::new(Duration::from_secs(10));
        assert!(limiter.try_acquire("example.com"));
        // Second immediate request should be blocked
        assert!(!limiter.try_acquire("example.com"));
    }

    #[test]
    fn test_default_web_limiter() {
        let limiter = RateLimiter::default_web();
        assert!(limiter.try_acquire("test.com"));
    }
}
