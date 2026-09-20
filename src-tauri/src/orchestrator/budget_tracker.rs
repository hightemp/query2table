use crate::providers::accounting::{
    BudgetExceeded, PriceQuote, RequestInfo, RequestKind, UsageObserver,
};
use crate::providers::llm::types::LlmUsage;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CostLine {
    pub provider: String,
    pub model: String,
    pub requested_model: String,
    pub calls: u64,
    pub reported_usd: f64,
    pub estimated_usd: f64,
    pub unpriced_calls: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub pricing: Option<PriceQuote>,
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BudgetSnapshot {
    pub spent_usd: f64,
    pub max_budget_usd: f64,
    pub reported_usd: f64,
    pub estimated_usd: f64,
    pub reported_calls: u64,
    pub estimated_calls: u64,
    pub unpriced_calls: u64,
    pub pending_calls: u64,
    pub missing_usage_calls: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub reasoning_tokens: u64,
    pub cached_prompt_tokens: u64,
    pub llm_calls: u64,
    pub search_calls: u64,
    pub fetch_calls: u64,
    pub breakdown: Vec<CostLine>,
}
#[derive(Debug)]
struct Pending {
    request: RequestInfo,
    usage: LlmUsage,
}
#[derive(Debug, Default)]
struct Ledger {
    totals: BudgetSnapshot,
    pending: BTreeMap<u64, Pending>,
    next_id: u64,
    closed: bool,
}
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    inner: Arc<Mutex<Ledger>>,
}
impl BudgetTracker {
    pub fn new(max_budget_usd: f64) -> Self {
        let mut ledger = Ledger::default();
        ledger.totals.max_budget_usd = if max_budget_usd.is_finite() {
            max_budget_usd.max(0.0)
        } else {
            0.0
        };
        Self {
            inner: Arc::new(Mutex::new(ledger)),
        }
    }
    fn ledger(&self) -> MutexGuard<'_, Ledger> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
    pub fn observer(&self) -> Arc<dyn UsageObserver> {
        Arc::new(self.clone())
    }
    pub fn record_fetch_call(&self) {
        self.ledger().totals.fetch_calls += 1;
    }
    pub fn snapshot(&self) -> BudgetSnapshot {
        let ledger = self.ledger();
        let mut result = ledger.totals.clone();
        result.pending_calls = ledger.pending.len() as u64;
        result
    }
    pub fn spent_usd(&self) -> f64 {
        self.ledger().totals.spent_usd
    }
    pub fn max_budget_usd(&self) -> f64 {
        self.ledger().totals.max_budget_usd
    }
    pub fn is_exceeded(&self) -> bool {
        let ledger = self.ledger();
        reached_limit(ledger.totals.spent_usd, ledger.totals.max_budget_usd)
    }
    pub fn remaining_usd(&self) -> f64 {
        let ledger = self.ledger();
        (ledger.totals.max_budget_usd - ledger.totals.spent_usd).max(0.0)
    }
    pub fn set_max_budget_usd(&self, amount: f64) {
        if amount.is_finite() && amount >= 0.0 {
            self.ledger().totals.max_budget_usd = amount;
        }
    }
    pub fn check_budget_warning(&self) {
        let state = self.snapshot();
        if state.spent_usd >= state.max_budget_usd * 0.8 {
            tracing::warn!(
                spent = state.spent_usd,
                limit = state.max_budget_usd,
                "Accounted spending is near its limit"
            );
        }
    }
    /// Worker aborts are asynchronous. Settle remaining attempts exactly once before terminal persistence.
    pub fn finalize_pending(&self) {
        let mut ledger = self.ledger();
        ledger.closed = true;
        for (_, pending) in std::mem::take(&mut ledger.pending) {
            settle(&mut ledger.totals, pending, None);
        }
    }
}
impl UsageObserver for BudgetTracker {
    fn exceeded(&self) -> bool {
        let ledger = self.ledger();
        ledger.closed || reached_limit(ledger.totals.spent_usd, ledger.totals.max_budget_usd)
    }
    fn begin(&self, request: RequestInfo) -> Result<u64, BudgetExceeded> {
        let mut ledger = self.ledger();
        if ledger.closed || reached_limit(ledger.totals.spent_usd, ledger.totals.max_budget_usd) {
            return Err(BudgetExceeded);
        }
        match request.kind {
            RequestKind::Llm => ledger.totals.llm_calls += 1,
            RequestKind::Search => ledger.totals.search_calls += 1,
        }
        ledger.next_id += 1;
        let id = ledger.next_id;
        ledger.pending.insert(
            id,
            Pending {
                request,
                usage: LlmUsage::default(),
            },
        );
        Ok(id)
    }
    fn capture(&self, id: u64, usage: LlmUsage) {
        if let Some(pending) = self.ledger().pending.get_mut(&id) {
            pending.usage = usage;
        }
    }
    fn finish(&self, id: u64, pricing: Option<PriceQuote>) {
        let mut ledger = self.ledger();
        if let Some(pending) = ledger.pending.remove(&id) {
            settle(&mut ledger.totals, pending, pricing);
        }
    }
}
// A decimal USD total can land a few floating-point ULPs below its cap.
// Treat that tiny boundary as reached rather than admitting another paid request.
fn reached_limit(spent: f64, limit: f64) -> bool {
    spent >= limit - limit * 1e-12
}

fn add_money(left: f64, right: f64) -> f64 {
    let sum = left + right;
    if sum.is_finite() {
        sum
    } else {
        f64::MAX
    }
}

fn settle(total: &mut BudgetSnapshot, pending: Pending, pricing: Option<PriceQuote>) {
    let usage = pending.usage;
    let actual_model = usage
        .model
        .clone()
        .unwrap_or_else(|| pending.request.model.clone());
    let reported = usage
        .cost_usd
        .filter(|value| value.is_finite() && *value >= 0.0);
    let pricing = pricing.filter(PriceQuote::valid);
    let estimate = if reported.is_none() {
        pricing
            .as_ref()
            .and_then(|quote| match pending.request.kind {
                RequestKind::Llm => quote.estimate(&usage),
                RequestKind::Search => Some(quote.per_request),
            })
    } else {
        None
    };
    let input = u64::from(usage.prompt_tokens.unwrap_or(0));
    let output = u64::from(usage.completion_tokens.unwrap_or(0));
    total.prompt_tokens = total.prompt_tokens.saturating_add(input);
    total.completion_tokens = total.completion_tokens.saturating_add(output);
    total.reasoning_tokens = total
        .reasoning_tokens
        .saturating_add(u64::from(usage.reasoning_tokens.unwrap_or(0)));
    total.cached_prompt_tokens = total
        .cached_prompt_tokens
        .saturating_add(u64::from(usage.cached_prompt_tokens.unwrap_or(0)));
    if pending.request.kind == RequestKind::Llm
        && (usage.prompt_tokens.is_none() || usage.completion_tokens.is_none())
    {
        total.missing_usage_calls += 1;
    }
    if let Some(cost) = reported {
        total.reported_usd = add_money(total.reported_usd, cost);
        total.reported_calls += 1;
    } else if let Some(cost) = estimate {
        total.estimated_usd = add_money(total.estimated_usd, cost);
        total.estimated_calls += 1;
    } else {
        total.unpriced_calls += 1;
    }
    total.spent_usd = add_money(total.reported_usd, total.estimated_usd);
    let index = total
        .breakdown
        .iter()
        .position(|line| {
            line.provider == pending.request.provider
                && line.model == actual_model
                && line.requested_model == pending.request.model
                && line.pricing == pricing
        })
        .unwrap_or_else(|| {
            let index = total.breakdown.len();
            total.breakdown.push(CostLine {
                provider: pending.request.provider,
                model: actual_model,
                requested_model: pending.request.model,
                pricing,
                ..CostLine::default()
            });
            index
        });
    let line = &mut total.breakdown[index];
    line.calls += 1;
    line.prompt_tokens += input;
    line.completion_tokens += output;
    line.reported_usd = add_money(line.reported_usd, reported.unwrap_or(0.0));
    line.estimated_usd = add_money(line.estimated_usd, estimate.unwrap_or(0.0));
    if reported.is_none() && estimate.is_none() {
        line.unpriced_calls += 1;
    }
    tracing::debug!(
        reported = reported.is_some(),
        estimated = estimate.is_some(),
        unpriced = total.unpriced_calls,
        "[FIX:cost] Accounted one provider attempt"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::accounting::UsageGuard;
    fn attempt(tracker: &BudgetTracker) -> UsageGuard {
        UsageGuard::begin(
            Some(tracker.observer()),
            RequestInfo {
                kind: RequestKind::Llm,
                provider: "test".into(),
                model: "model".into(),
            },
        )
        .unwrap()
    }
    fn quote() -> PriceQuote {
        PriceQuote {
            input_per_million: 2.0,
            output_per_million: 8.0,
            cached_input_per_million: Some(0.5),
            cache_write_per_million: None,
            per_request: 0.0,
            source: "test rate".into(),
            credit_based: false,
        }
    }
    #[test]
    fn reported_cost_wins_over_rates_and_reasoning_is_not_double_counted() {
        let tracker = BudgetTracker::new(10.0);
        let request = attempt(&tracker);
        request.capture(LlmUsage {
            prompt_tokens: Some(1000),
            completion_tokens: Some(500),
            reasoning_tokens: Some(400),
            cost_usd: Some(0.1234567),
            ..Default::default()
        });
        request.finish(Some(quote()));
        let result = tracker.snapshot();
        assert_eq!(result.spent_usd, 0.1234567);
        assert_eq!(result.estimated_calls, 0);
        assert_eq!(
            (
                result.prompt_tokens,
                result.completion_tokens,
                result.reasoning_tokens
            ),
            (1000, 500, 400)
        );
    }
    #[test]
    fn estimates_use_actual_counts_and_cached_input_rate() {
        let tracker = BudgetTracker::new(10.0);
        let request = attempt(&tracker);
        request.capture(LlmUsage {
            prompt_tokens: Some(1_000_000),
            cached_prompt_tokens: Some(200_000),
            completion_tokens: Some(100_000),
            ..Default::default()
        });
        request.finish(Some(quote()));
        assert!((tracker.spent_usd() - 2.5).abs() < 1e-9);
        assert_eq!(tracker.snapshot().estimated_calls, 1);
    }
    #[test]
    fn absent_usage_or_prices_remain_unknown_and_explicit_zero_is_valid() {
        let tracker = BudgetTracker::new(10.0);
        attempt(&tracker).finish(Some(quote()));
        let request = attempt(&tracker);
        request.capture(LlmUsage {
            cost_usd: Some(0.0),
            ..Default::default()
        });
        request.finish(None);
        let result = tracker.snapshot();
        assert_eq!(result.unpriced_calls, 1);
        assert_eq!(result.reported_calls, 1);
        assert_eq!(result.missing_usage_calls, 2);
    }
    #[test]
    fn cancellation_and_terminal_settlement_count_each_attempt_once() {
        let tracker = BudgetTracker::new(10.0);
        let first = attempt(&tracker);
        let second = attempt(&tracker);
        first.capture(LlmUsage {
            prompt_tokens: Some(7),
            cost_usd: Some(0.25),
            ..Default::default()
        });
        tracker.finalize_pending();
        drop(first);
        drop(second);
        assert!(UsageGuard::begin(
            Some(tracker.observer()),
            RequestInfo {
                kind: RequestKind::Llm,
                provider: "test".into(),
                model: "test".into()
            }
        )
        .is_err());
        let result = tracker.snapshot();
        assert_eq!(result.llm_calls, 2);
        assert_eq!(result.pending_calls, 0);
        assert_eq!(result.unpriced_calls, 1);
        assert_eq!(result.spent_usd, 0.25);
        assert_eq!(result.prompt_tokens, 7);
    }
    #[test]
    fn every_new_attempt_is_denied_after_the_accounted_limit() {
        let tracker = BudgetTracker::new(0.01);
        let request = attempt(&tracker);
        request.capture(LlmUsage {
            cost_usd: Some(0.01),
            ..Default::default()
        });
        request.finish(None);
        assert!(UsageGuard::begin(
            Some(tracker.observer()),
            RequestInfo {
                kind: RequestKind::Search,
                provider: "brave".into(),
                model: "search".into()
            }
        )
        .is_err());
        assert_eq!(tracker.snapshot().search_calls, 0);
    }
    #[test]
    fn invalid_cache_counts_or_rates_cannot_become_a_plausible_estimate() {
        let usage = LlmUsage {
            prompt_tokens: Some(5),
            cached_prompt_tokens: Some(9),
            completion_tokens: Some(1),
            ..Default::default()
        };
        assert!(quote().estimate(&usage).is_none());
        let mut invalid = quote();
        invalid.output_per_million = f64::NAN;
        assert!(!invalid.valid());
    }
    #[test]
    fn manual_zero_rates_are_distinct_from_missing_rates() {
        let mut price = quote();
        price.input_per_million = 0.0;
        price.output_per_million = 0.0;
        price.cached_input_per_million = None;
        assert_eq!(price.estimate(&LlmUsage::default()), Some(0.0));
    }
    #[test]
    fn decimal_rounding_does_not_admit_an_extra_paid_call() {
        let tracker = BudgetTracker::new(1.0);
        for _ in 0..10 {
            let request = attempt(&tracker);
            request.capture(LlmUsage {
                cost_usd: Some(0.1),
                ..Default::default()
            });
            request.finish(None);
        }
        assert!(tracker.is_exceeded());
        assert!(UsageGuard::begin(
            Some(tracker.observer()),
            RequestInfo {
                kind: RequestKind::Llm,
                provider: "test".into(),
                model: "test".into()
            }
        )
        .is_err());
        assert_eq!(tracker.snapshot().llm_calls, 10);
    }
    #[test]
    fn totals_remain_serializable_with_extreme_provider_values() {
        let tracker = BudgetTracker::new(f64::MAX);
        let a = attempt(&tracker);
        let b = attempt(&tracker);
        for request in [a, b] {
            request.capture(LlmUsage {
                cost_usd: Some(f64::MAX),
                ..Default::default()
            });
            request.finish(None);
        }
        assert!(tracker.spent_usd().is_finite());
        assert!(serde_json::to_string(&tracker.snapshot()).is_ok());
    }
}
