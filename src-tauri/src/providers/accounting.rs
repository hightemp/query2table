//! Provider-neutral accounting. Observers own run state; clients only report attempts.
use super::llm::types::LlmUsage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceQuote {
    pub input_per_million: f64,
    pub output_per_million: f64,
    #[serde(default)]
    pub cached_input_per_million: Option<f64>,
    #[serde(default)]
    pub cache_write_per_million: Option<f64>,
    #[serde(default)]
    pub per_request: f64,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub credit_based: bool,
}
impl PriceQuote {
    pub fn valid(&self) -> bool {
        [
            Some(self.input_per_million),
            Some(self.output_per_million),
            Some(self.per_request),
            self.cached_input_per_million,
            self.cache_write_per_million,
        ]
        .into_iter()
        .flatten()
        .all(|value| value.is_finite() && value >= 0.0)
    }
    pub fn estimate(&self, usage: &LlmUsage) -> Option<f64> {
        if !self.valid() {
            return None;
        }
        if self.input_per_million == 0.0
            && self.output_per_million == 0.0
            && self.cached_input_per_million.unwrap_or(0.0) == 0.0
            && self.cache_write_per_million.unwrap_or(0.0) == 0.0
        {
            return Some(self.per_request);
        }
        let input = usage.prompt_tokens?;
        let output = usage.completion_tokens?;
        let cached = usage.cached_prompt_tokens.unwrap_or(0);
        let written = usage.cache_write_tokens.unwrap_or(0);
        let ordinary = input.checked_sub(cached)?.checked_sub(written)?;
        let result = (f64::from(ordinary) * self.input_per_million
            + f64::from(cached)
                * self
                    .cached_input_per_million
                    .unwrap_or(self.input_per_million)
            + f64::from(written)
                * self
                    .cache_write_per_million
                    .unwrap_or(self.input_per_million)
            + f64::from(output) * self.output_per_million)
            / 1_000_000.0
            + self.per_request;
        (result.is_finite() && result >= 0.0).then_some(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Llm,
    Search,
}
#[derive(Debug, Clone)]
pub struct RequestInfo {
    pub kind: RequestKind,
    pub provider: String,
    pub model: String,
}

pub trait UsageObserver: Send + Sync {
    fn exceeded(&self) -> bool;
    fn begin(&self, request: RequestInfo) -> Result<u64, BudgetExceeded>;
    fn capture(&self, id: u64, usage: LlmUsage);
    fn finish(&self, id: u64, pricing: Option<PriceQuote>);
}
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Run spending limit reached. Increase the run's Max Cost to send more requests.")]
pub struct BudgetExceeded;

/// Dropped HTTP futures still leave an explicitly unknown charge, never a fake zero.
pub struct UsageGuard {
    observer: Option<Arc<dyn UsageObserver>>,
    id: u64,
}
impl UsageGuard {
    pub fn begin(
        observer: Option<Arc<dyn UsageObserver>>,
        request: RequestInfo,
    ) -> Result<Self, BudgetExceeded> {
        let id = if let Some(observer) = &observer {
            observer.begin(request)?
        } else {
            0
        };
        Ok(Self { observer, id })
    }
    pub fn capture(&self, usage: LlmUsage) {
        if let Some(observer) = &self.observer {
            observer.capture(self.id, usage);
        }
    }
    pub fn finish(mut self, pricing: Option<PriceQuote>) {
        if let Some(observer) = self.observer.take() {
            observer.finish(self.id, pricing);
        }
    }
}
impl Drop for UsageGuard {
    fn drop(&mut self) {
        if let Some(observer) = self.observer.take() {
            observer.finish(self.id, None);
        }
    }
}

pub fn validate_pricing_setting(key: &str, value: &str) -> Result<(), String> {
    let invalid = || {
        "Pricing settings are invalid. Complete the input and output rates with finite non-negative numbers, or disable custom rates.".to_string()
    };
    match key {
        "llm_pricing_overrides" => {
            let quotes: std::collections::HashMap<String, PriceQuote> =
                serde_json::from_str(value).map_err(|_| invalid())?;
            for (scope, quote) in quotes {
                let (provider, endpoint, model): (String, String, String) =
                    serde_json::from_str(&scope).map_err(|_| invalid())?;
                if !["openrouter", "ollama", "ollama_cloud", "openai_compatible"]
                    .contains(&provider.as_str())
                    || endpoint.trim().is_empty()
                    || model.trim().is_empty()
                    || !quote.valid()
                {
                    return Err(invalid());
                }
            }
        }
        "brave_price_per_1000" | "serper_price_per_1000" if !value.trim().is_empty() => {
            let rate: f64 = value.parse().map_err(|_| invalid())?;
            if !rate.is_finite() || rate < 0.0 {
                return Err(invalid());
            }
        }
        _ => {}
    }
    Ok(())
}
