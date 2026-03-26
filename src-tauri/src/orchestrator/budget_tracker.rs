use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, warn};

/// Tracks API usage costs across LLM and search calls.
/// Thread-safe via atomics — can be shared across tokio tasks.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    inner: Arc<BudgetInner>,
}

#[derive(Debug)]
struct BudgetInner {
    /// Maximum allowed spend in microdollars (USD * 1_000_000)
    max_budget_micro: AtomicU64,
    /// Current spend in microdollars
    spent_micro: AtomicU64,
    /// Total LLM prompt tokens used
    prompt_tokens: AtomicU64,
    /// Total LLM completion tokens used
    completion_tokens: AtomicU64,
    /// Total search API calls
    search_calls: AtomicU64,
    /// Total LLM API calls
    llm_calls: AtomicU64,
    /// Total fetch requests
    fetch_calls: AtomicU64,
}

impl BudgetTracker {
    pub fn new(max_budget_usd: f64) -> Self {
        let micro = (max_budget_usd * 1_000_000.0) as u64;
        Self {
            inner: Arc::new(BudgetInner {
                max_budget_micro: AtomicU64::new(micro),
                spent_micro: AtomicU64::new(0),
                prompt_tokens: AtomicU64::new(0),
                completion_tokens: AtomicU64::new(0),
                search_calls: AtomicU64::new(0),
                llm_calls: AtomicU64::new(0),
                fetch_calls: AtomicU64::new(0),
            }),
        }
    }

    /// Record an LLM API call with token counts.
    /// Uses a simple cost model: $0.15/1M prompt tokens, $0.60/1M completion tokens
    /// (roughly GPT-4o-mini pricing). Actual costs vary by model.
    pub fn record_llm_call(&self, prompt_tokens: u32, completion_tokens: u32) {
        self.inner.prompt_tokens.fetch_add(prompt_tokens as u64, Ordering::Relaxed);
        self.inner.completion_tokens.fetch_add(completion_tokens as u64, Ordering::Relaxed);
        self.inner.llm_calls.fetch_add(1, Ordering::Relaxed);

        // Cost estimate in microdollars: prompt=$0.15/1M, completion=$0.60/1M
        let prompt_cost_micro = (prompt_tokens as u64) * 150 / 1_000_000;
        let completion_cost_micro = (completion_tokens as u64) * 600 / 1_000_000;
        // Minimum 1 microdollar per call to account for rounding
        let total = (prompt_cost_micro + completion_cost_micro).max(1);

        self.inner.spent_micro.fetch_add(total, Ordering::Relaxed);
        debug!(prompt_tokens, completion_tokens, cost_micro = total, "Recorded LLM call");
    }

    /// Record a search API call. Estimated at $0.005 per call.
    pub fn record_search_call(&self) {
        self.inner.search_calls.fetch_add(1, Ordering::Relaxed);
        // $0.005 = 5000 microdollars
        self.inner.spent_micro.fetch_add(5000, Ordering::Relaxed);
    }

    /// Record a fetch (no cost, just tracking).
    pub fn record_fetch_call(&self) {
        self.inner.fetch_calls.fetch_add(1, Ordering::Relaxed);
    }

    /// Check if the budget has been exceeded.
    pub fn is_exceeded(&self) -> bool {
        let spent = self.inner.spent_micro.load(Ordering::Relaxed);
        let max = self.inner.max_budget_micro.load(Ordering::Relaxed);
        spent >= max
    }

    /// Get estimated total cost in USD.
    pub fn spent_usd(&self) -> f64 {
        self.inner.spent_micro.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    /// Get the max budget in USD.
    pub fn max_budget_usd(&self) -> f64 {
        self.inner.max_budget_micro.load(Ordering::Relaxed) as f64 / 1_000_000.0
    }

    /// Get a snapshot of current stats.
    pub fn snapshot(&self) -> BudgetSnapshot {
        BudgetSnapshot {
            spent_usd: self.spent_usd(),
            max_budget_usd: self.max_budget_usd(),
            prompt_tokens: self.inner.prompt_tokens.load(Ordering::Relaxed),
            completion_tokens: self.inner.completion_tokens.load(Ordering::Relaxed),
            llm_calls: self.inner.llm_calls.load(Ordering::Relaxed),
            search_calls: self.inner.search_calls.load(Ordering::Relaxed),
            fetch_calls: self.inner.fetch_calls.load(Ordering::Relaxed),
        }
    }

    /// Update the budget limit (e.g. from settings).
    pub fn set_max_budget_usd(&self, usd: f64) {
        let micro = (usd * 1_000_000.0) as u64;
        self.inner.max_budget_micro.store(micro, Ordering::Relaxed);
        debug!(max_budget_usd = usd, "Updated budget limit");
    }

    /// Check remaining budget in USD.
    pub fn remaining_usd(&self) -> f64 {
        let max = self.inner.max_budget_micro.load(Ordering::Relaxed) as f64;
        let spent = self.inner.spent_micro.load(Ordering::Relaxed) as f64;
        (max - spent).max(0.0) / 1_000_000.0
    }

    /// Log a warning if budget is nearly exhausted (>80%).
    pub fn check_budget_warning(&self) {
        let spent = self.inner.spent_micro.load(Ordering::Relaxed) as f64;
        let max = self.inner.max_budget_micro.load(Ordering::Relaxed) as f64;
        if max > 0.0 && spent / max > 0.8 {
            warn!(
                spent_usd = spent / 1_000_000.0,
                max_usd = max / 1_000_000.0,
                "Budget usage above 80%"
            );
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BudgetSnapshot {
    pub spent_usd: f64,
    pub max_budget_usd: f64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub llm_calls: u64,
    pub search_calls: u64,
    pub fetch_calls: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let bt = BudgetTracker::new(1.0);
        assert!(!bt.is_exceeded());
        assert_eq!(bt.spent_usd(), 0.0);
        assert_eq!(bt.max_budget_usd(), 1.0);
        assert_eq!(bt.remaining_usd(), 1.0);
    }

    #[test]
    fn test_record_llm_call() {
        let bt = BudgetTracker::new(10.0);
        bt.record_llm_call(1000, 500);
        let snap = bt.snapshot();
        assert_eq!(snap.prompt_tokens, 1000);
        assert_eq!(snap.completion_tokens, 500);
        assert_eq!(snap.llm_calls, 1);
        assert!(snap.spent_usd > 0.0);
    }

    #[test]
    fn test_record_search_call() {
        let bt = BudgetTracker::new(1.0);
        bt.record_search_call();
        let snap = bt.snapshot();
        assert_eq!(snap.search_calls, 1);
        assert_eq!(snap.spent_usd, 0.005);
    }

    #[test]
    fn test_budget_exceeded() {
        let bt = BudgetTracker::new(0.001); // very small budget
        // Record many search calls to exceed budget
        for _ in 0..10 {
            bt.record_search_call();
        }
        // 10 * $0.005 = $0.05 > $0.001
        assert!(bt.is_exceeded());
    }

    #[test]
    fn test_budget_not_exceeded() {
        let bt = BudgetTracker::new(100.0);
        bt.record_search_call();
        bt.record_llm_call(100, 50);
        assert!(!bt.is_exceeded());
    }

    #[test]
    fn test_remaining_usd() {
        let bt = BudgetTracker::new(1.0);
        bt.record_search_call(); // $0.005
        let remaining = bt.remaining_usd();
        assert!((remaining - 0.995).abs() < 0.001);
    }

    #[test]
    fn test_set_max_budget() {
        let bt = BudgetTracker::new(1.0);
        bt.set_max_budget_usd(5.0);
        assert_eq!(bt.max_budget_usd(), 5.0);
    }

    #[test]
    fn test_record_fetch_call() {
        let bt = BudgetTracker::new(1.0);
        bt.record_fetch_call();
        bt.record_fetch_call();
        let snap = bt.snapshot();
        assert_eq!(snap.fetch_calls, 2);
        assert_eq!(snap.spent_usd, 0.0); // fetches are free
    }

    #[test]
    fn test_thread_safe_clone() {
        let bt = BudgetTracker::new(1.0);
        let bt2 = bt.clone();
        bt.record_search_call();
        // Both references share state
        assert_eq!(bt2.snapshot().search_calls, 1);
    }

    #[test]
    fn test_snapshot_serializable() {
        let bt = BudgetTracker::new(1.0);
        bt.record_llm_call(500, 200);
        let snap = bt.snapshot();
        let json = serde_json::to_string(&snap).unwrap();
        assert!(json.contains("spent_usd"));
        assert!(json.contains("prompt_tokens"));
    }
}
