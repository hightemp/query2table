use std::time::Instant;
use tracing::debug;

/// Configuration for stopping conditions.
#[derive(Debug, Clone)]
pub struct StopConfig {
    pub target_row_count: usize,
    pub max_budget_usd: f64,
    pub max_duration_secs: u64,
    pub saturation_threshold: f64,
}

impl Default for StopConfig {
    fn default() -> Self {
        Self {
            target_row_count: 50,
            max_budget_usd: 1.0,
            max_duration_secs: 600,
            saturation_threshold: 0.05,
        }
    }
}

/// Current pipeline stats for stop condition evaluation.
#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub row_count: usize,
    pub estimated_cost_usd: f64,
    pub start_time: Instant,
    pub last_batch_new_rows: usize,
    pub last_batch_total_rows: usize,
}

/// Reason the pipeline should stop.
#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    TargetReached,
    BudgetExceeded,
    TimeExceeded,
    SearchSaturated,
    Cancelled,
}

impl StopReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TargetReached => "target_reached",
            Self::BudgetExceeded => "budget_exceeded",
            Self::TimeExceeded => "time_exceeded",
            Self::SearchSaturated => "search_saturated",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Evaluates stopping conditions for the pipeline.
pub struct StoppingController;

impl StoppingController {
    /// Check all stop conditions. Returns Some(reason) if pipeline should stop.
    pub fn should_stop(config: &StopConfig, stats: &PipelineStats) -> Option<StopReason> {
        // 1. Target row count reached
        if stats.row_count >= config.target_row_count {
            debug!(
                rows = stats.row_count,
                target = config.target_row_count,
                "Stop: target row count reached"
            );
            return Some(StopReason::TargetReached);
        }

        // 2. Budget exceeded
        if stats.estimated_cost_usd >= config.max_budget_usd {
            debug!(
                cost = %stats.estimated_cost_usd,
                budget = %config.max_budget_usd,
                "Stop: budget exceeded"
            );
            return Some(StopReason::BudgetExceeded);
        }

        // 3. Time exceeded
        let elapsed = stats.start_time.elapsed().as_secs();
        if elapsed >= config.max_duration_secs {
            debug!(
                elapsed_secs = elapsed,
                max_secs = config.max_duration_secs,
                "Stop: time exceeded"
            );
            return Some(StopReason::TimeExceeded);
        }

        // 4. Search saturation — new rows / total rows in last batch below threshold
        if stats.last_batch_total_rows > 0 {
            let saturation_rate = stats.last_batch_new_rows as f64
                / stats.last_batch_total_rows as f64;
            if saturation_rate < config.saturation_threshold && stats.row_count > 0 {
                debug!(
                    saturation = %saturation_rate,
                    threshold = %config.saturation_threshold,
                    "Stop: search saturated"
                );
                return Some(StopReason::SearchSaturated);
            }
        }

        None
    }

    /// Build StopConfig from settings.
    pub fn config_from_settings(settings: &std::collections::HashMap<String, String>) -> StopConfig {
        StopConfig {
            target_row_count: settings.get("target_row_count")
                .and_then(|v| v.parse().ok())
                .unwrap_or(50),
            max_budget_usd: settings.get("max_budget_usd")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1.0),
            max_duration_secs: settings.get("max_duration_seconds")
                .and_then(|v| v.parse().ok())
                .unwrap_or(600),
            saturation_threshold: settings.get("saturation_threshold")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.05),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn make_stats(rows: usize, cost: f64, elapsed_secs: u64, new: usize, total: usize) -> PipelineStats {
        PipelineStats {
            row_count: rows,
            estimated_cost_usd: cost,
            start_time: Instant::now() - Duration::from_secs(elapsed_secs),
            last_batch_new_rows: new,
            last_batch_total_rows: total,
        }
    }

    #[test]
    fn test_no_stop() {
        let config = StopConfig::default();
        let stats = make_stats(10, 0.1, 60, 5, 10);
        assert!(StoppingController::should_stop(&config, &stats).is_none());
    }

    #[test]
    fn test_target_reached() {
        let config = StopConfig { target_row_count: 50, ..Default::default() };
        let stats = make_stats(50, 0.1, 60, 5, 10);
        assert_eq!(
            StoppingController::should_stop(&config, &stats),
            Some(StopReason::TargetReached)
        );
    }

    #[test]
    fn test_budget_exceeded() {
        let config = StopConfig { max_budget_usd: 1.0, ..Default::default() };
        let stats = make_stats(10, 1.5, 60, 5, 10);
        assert_eq!(
            StoppingController::should_stop(&config, &stats),
            Some(StopReason::BudgetExceeded)
        );
    }

    #[test]
    fn test_time_exceeded() {
        let config = StopConfig { max_duration_secs: 300, ..Default::default() };
        let stats = make_stats(10, 0.1, 400, 5, 10);
        assert_eq!(
            StoppingController::should_stop(&config, &stats),
            Some(StopReason::TimeExceeded)
        );
    }

    #[test]
    fn test_search_saturated() {
        let config = StopConfig { saturation_threshold: 0.05, ..Default::default() };
        // 1 new row out of 100 total = 0.01 < 0.05
        let stats = make_stats(30, 0.1, 60, 1, 100);
        assert_eq!(
            StoppingController::should_stop(&config, &stats),
            Some(StopReason::SearchSaturated)
        );
    }

    #[test]
    fn test_saturation_not_triggered_when_no_rows() {
        let config = StopConfig { saturation_threshold: 0.05, ..Default::default() };
        // Even if saturation rate is 0, don't stop if we have no rows yet
        let stats = make_stats(0, 0.1, 60, 0, 10);
        assert!(StoppingController::should_stop(&config, &stats).is_none());
    }

    #[test]
    fn test_config_from_settings() {
        let mut settings = std::collections::HashMap::new();
        settings.insert("target_row_count".to_string(), "100".to_string());
        settings.insert("max_budget_usd".to_string(), "5.0".to_string());
        settings.insert("max_duration_seconds".to_string(), "1200".to_string());
        settings.insert("saturation_threshold".to_string(), "0.1".to_string());

        let config = StoppingController::config_from_settings(&settings);
        assert_eq!(config.target_row_count, 100);
        assert_eq!(config.max_budget_usd, 5.0);
        assert_eq!(config.max_duration_secs, 1200);
        assert_eq!(config.saturation_threshold, 0.1);
    }

    #[test]
    fn test_config_defaults_on_empty_settings() {
        let settings = std::collections::HashMap::new();
        let config = StoppingController::config_from_settings(&settings);
        assert_eq!(config.target_row_count, 50);
        assert_eq!(config.max_budget_usd, 1.0);
    }

    #[test]
    fn test_stop_reason_as_str() {
        assert_eq!(StopReason::TargetReached.as_str(), "target_reached");
        assert_eq!(StopReason::BudgetExceeded.as_str(), "budget_exceeded");
        assert_eq!(StopReason::Cancelled.as_str(), "cancelled");
    }
}
