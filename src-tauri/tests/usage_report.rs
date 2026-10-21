//! The statistics report: totals of a period and of the one before it, a series by the
//! frontend's (local) day or month boundaries, breakdowns by model, search provider and mode,
//! and the most expensive runs.

use query2table_lib::commands::usage::get_usage_report;
use query2table_lib::storage::{
    db::Database,
    repository::Repository,
    usage::{UsageEvent, UsageRequest, UsageRun},
};
use sqlx::SqlitePool;

async fn setup() -> (SqlitePool, Repository) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    (pool.clone(), Repository::new(pool))
}

fn event(at: i64, run: &str, run_type: &str, kind: &str, provider: &str, model: &str) -> UsageEvent {
    UsageEvent {
        at,
        run_id: Some(run.into()),
        run_type: run_type.into(),
        kind: kind.into(),
        provider: provider.into(),
        model: model.into(),
        calls: 1,
        ..Default::default()
    }
}

fn llm(at: i64, run: &str, model: &str, reported: f64, estimated: f64) -> UsageEvent {
    UsageEvent {
        prompt_tokens: 1000,
        completion_tokens: 100,
        reasoning_tokens: 10,
        cached_tokens: 400,
        cache_write_tokens: 50,
        reported_usd: reported,
        estimated_usd: estimated,
        unpriced_calls: i64::from(reported == 0.0 && estimated == 0.0),
        bytes_sent: 5000,
        bytes_received: 700,
        ..event(at, run, "table", "llm", "openrouter", model)
    }
}

// Three "days" of 100 seconds from 1000, and the period before them from 700.
const DAY: i64 = 100;
fn request() -> UsageRequest {
    UsageRequest { from: 1000, to: 1300, buckets: vec![1000, 1100, 1200], previous_from: 700 }
}

#[tokio::test]
async fn a_period_has_totals_a_series_and_the_period_before() {
    let (_, repo) = setup().await;
    let mut download = event(1150, "t1", "table", "download", "", "");
    download.bytes_received = 30_000;
    let mut search = event(1210, "t1", "table", "search", "brave", "web search");
    search.estimated_usd = 0.005;
    search.bytes_received = 2_000;
    repo.record_usage(&[
        llm(1000, "t1", "a", 0.10, 0.0),
        llm(1050, "t1", "a", 0.0, 0.02),
        llm(1210, "t2", "b", 0.0, 0.0),
        download,
        search,
        // Outside the period: before the previous one, in the previous one, and after.
        llm(600, "t0", "a", 9.0, 0.0),
        llm(800, "t0", "a", 0.5, 0.0),
        llm(1300, "t3", "a", 7.0, 0.0),
    ])
    .await
    .unwrap();

    let report = get_usage_report(&repo, &request()).await.unwrap();
    let t = &report.totals;
    assert!((t.cost_usd - 0.125).abs() < 1e-9, "{}", t.cost_usd);
    assert_eq!((t.reported_usd, t.estimated_usd), (0.10, 0.025));
    assert_eq!((t.llm_calls, t.unpriced_calls, t.searches, t.downloads, t.runs), (3, 1, 1, 1, 2));
    assert_eq!(
        (t.prompt_tokens, t.completion_tokens, t.reasoning_tokens, t.cached_tokens, t.cache_write_tokens),
        (3000, 300, 30, 1200, 150)
    );
    assert_eq!((t.model_bytes_sent, t.model_bytes_received, t.web_bytes), (15_000, 2_100, 32_000));

    assert_eq!(report.previous.cost_usd, 0.5);
    assert_eq!(report.previous.llm_calls, 1);

    // One point per bucket, in order.
    let series: Vec<(i64, f64, i64, i64)> = report.series.iter().map(|p| (p.start, p.totals.cost_usd, p.totals.llm_calls, p.totals.web_bytes)).collect();
    assert_eq!(series.len(), 3);
    assert_eq!((series[0].0, series[0].2, series[0].3), (1000, 2, 0));
    assert!((series[0].1 - 0.12).abs() < 1e-9);
    assert_eq!((series[1].0, series[1].1, series[1].3), (1000 + DAY, 0.0, 30_000));
    assert_eq!((series[2].0, series[2].2, series[2].3), (1000 + 2 * DAY, 1, 2_000));
    assert!((series[2].1 - 0.005).abs() < 1e-9);
}

#[tokio::test]
async fn breakdowns_by_model_search_provider_and_mode() {
    let (_, repo) = setup().await;
    let mut research = llm(1010, "r1", "claude", 0.30, 0.0);
    research.run_type = "research".into();
    research.provider = "anthropic".into();
    let mut serper = event(1020, "t1", "table", "search", "serper", "web search");
    serper.estimated_usd = 0.001;
    repo.record_usage(&[
        llm(1000, "t1", "a", 0.10, 0.0),
        llm(1001, "t1", "a", 0.0, 0.0),
        research,
        event(1030, "t1", "table", "search", "brave", "web search"),
        event(1031, "t1", "table", "search", "brave", "image search"),
        serper,
    ])
    .await
    .unwrap();
    let report = get_usage_report(&repo, &request()).await.unwrap();

    // Most expensive first.
    let models: Vec<(&str, &str, i64, i64, f64, i64)> = report
        .by_model
        .iter()
        .map(|m| (m.provider.as_str(), m.model.as_str(), m.calls, m.prompt_tokens, m.cost_usd, m.unpriced_calls))
        .collect();
    assert_eq!(models, [("anthropic", "claude", 1, 1000, 0.30, 0), ("openrouter", "a", 2, 2000, 0.10, 1)]);

    let searches: Vec<(&str, i64, f64)> = report.by_search.iter().map(|s| (s.provider.as_str(), s.calls, s.cost_usd)).collect();
    assert_eq!(searches, [("brave", 2, 0.0), ("serper", 1, 0.001)]);

    let modes: Vec<(&str, i64, f64, i64)> = report.by_mode.iter().map(|m| (m.run_type.as_str(), m.runs, m.cost_usd, m.llm_calls)).collect();
    assert_eq!(modes, [("research", 1, 0.30, 1), ("table", 1, 0.101, 2)]);
}

#[tokio::test]
async fn the_most_expensive_runs_link_only_to_runs_that_still_exist() {
    let (pool, repo) = setup().await;
    for (id, title) in [("kept", "Robot channels"), ("trashed", "Old idea"), ("gone", "Gone run")] {
        sqlx::query("INSERT INTO runs (id, query, status, config, run_type) VALUES (?, ?, 'completed', '{}', 'table')").bind(id).bind(format!("query {id}")).execute(&pool).await.unwrap();
        repo.remember_usage_run(&UsageRun { run_id: id.into(), run_type: "table".into(), title: format!("query {id}"), started_at: 1000 }).await.unwrap();
        sqlx::query("UPDATE runs SET title = ? WHERE id = ?").bind(title).bind(id).execute(&pool).await.unwrap();
    }
    sqlx::query("UPDATE runs SET deleted_at = 1 WHERE id = 'trashed'").execute(&pool).await.unwrap();
    repo.delete_run("gone").await.unwrap();
    let mut events = vec![llm(1000, "kept", "a", 0.2, 0.0), llm(1001, "trashed", "a", 0.5, 0.0), llm(1002, "gone", "a", 0.3, 0.0)];
    // Twelve cheap runs: only ten runs are listed.
    for i in 0..12 {
        events.push(llm(1100 + i, &format!("cheap{i}"), "a", 0.001, 0.0));
    }
    repo.record_usage(&events).await.unwrap();

    let report = get_usage_report(&repo, &request()).await.unwrap();
    assert_eq!(report.top_runs.len(), 10);
    let top: Vec<(&str, &str, f64, bool)> = report.top_runs.iter().take(3).map(|r| (r.run_id.as_str(), r.title.as_str(), r.cost_usd, r.exists)).collect();
    assert_eq!(top, [("trashed", "Old idea", 0.5, false), ("gone", "Gone run", 0.3, false), ("kept", "Robot channels", 0.2, true)]);
    assert_eq!(report.top_runs[0].run_type, "table");
}

#[tokio::test]
async fn bad_periods_are_refused() {
    let (_, repo) = setup().await;
    for request in [
        UsageRequest { from: 100, to: 100, buckets: vec![100], previous_from: 0 },
        UsageRequest { from: 100, to: 200, buckets: vec![150, 120], previous_from: 0 },
        UsageRequest { from: 100, to: 200, buckets: vec![], previous_from: 0 },
        UsageRequest { from: 100, to: 200, buckets: vec![100], previous_from: 150 },
    ] {
        assert!(get_usage_report(&repo, &request).await.is_err(), "{request:?}");
    }
}

#[tokio::test]
async fn disk_use_counts_the_database_its_log_attachments_and_kept_pictures() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");
    std::fs::write(&db, vec![0u8; 4096]).unwrap();
    std::fs::write(dir.path().join("data.db-wal"), vec![0u8; 1000]).unwrap();
    let attachments = dir.path().join("attachments");
    std::fs::create_dir_all(attachments.join("ab")).unwrap();
    std::fs::write(attachments.join("ab").join("abcdef"), vec![0u8; 300]).unwrap();
    std::fs::write(attachments.join("top"), vec![0u8; 20]).unwrap();

    let (pool, repo) = setup().await;
    sqlx::query("INSERT INTO runs (id, query, status, config, run_type) VALUES ('r1', 'q', 'completed', '{}', 'research')").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO research_images (run_id, url, media_type, width, height, data) VALUES ('r1', 'https://x/a.png', 'image/jpeg', 1, 1, ?)")
        .bind(vec![1u8; 77])
        .execute(&pool)
        .await
        .unwrap();

    let usage = query2table_lib::commands::usage::storage_usage(&repo, &db, &attachments).await.unwrap();
    assert_eq!((usage.database_bytes, usage.attachments_bytes, usage.answer_images_bytes), (5096, 320, 77));
    // A missing folder holds nothing.
    let usage = query2table_lib::commands::usage::storage_usage(&repo, &db, &dir.path().join("none")).await.unwrap();
    assert_eq!(usage.attachments_bytes, 0);
}

#[tokio::test]
async fn calls_without_a_price_are_priced_with_todays_rates() {
    let (_, repo) = setup().await;
    let unpriced = |model: &str, answered: &str| UsageEvent {
        requested_model: model.into(),
        model: answered.into(),
        cached_tokens: 0,
        cache_write_tokens: 0,
        reasoning_tokens: 0,
        ..llm(1000, "t1", model, 0.0, 0.0)
    };
    let mut folded = unpriced("m", "m");
    // A folded row mixing priced and unpriced calls cannot be split, so it stays as it is.
    (folded.calls, folded.unpriced_calls, folded.estimated_usd) = (3, 1, 0.5);
    let mut search = event(1001, "t1", "table", "search", "brave", "web search");
    (search.calls, search.unpriced_calls) = (2, 2);
    let serper = UsageEvent { unpriced_calls: 1, ..event(1002, "t1", "table", "search", "serper", "web search") };
    repo.record_usage(&[unpriced("m", "m-2025-01-01"), unpriced("other", "other"), folded, search, serper]).await.unwrap();

    // Rates are set for the model asked for, at some address of the provider.
    let scope = serde_json::to_string(&("openrouter", "https://openrouter.ai/api/v1", "m")).unwrap();
    let rates = serde_json::json!({(scope): {"input_per_million": 2.0, "output_per_million": 10.0}}).to_string();
    let settings = std::collections::HashMap::from([
        ("llm_pricing_overrides".to_string(), rates),
        ("brave_price_per_1000".to_string(), "5".to_string()),
        ("serper_price_per_1000".to_string(), "".to_string()),
    ]);
    assert_eq!(repo.reprice_usage(&settings).await.unwrap(), 3, "one model call and two searches");

    let rows = repo.usage_events(0, i64::MAX).await.unwrap();
    let priced: Vec<(&str, &str, f64, i64)> = rows.iter().map(|e| (e.kind.as_str(), e.requested_model.as_str(), e.estimated_usd, e.unpriced_calls)).collect();
    assert_eq!(
        priced,
        [
            // 1000 input tokens at $2 and 100 output tokens at $10 per million.
            ("llm", "m", 0.003, 0),
            ("llm", "other", 0.0, 1),
            ("llm", "m", 0.5, 1),
            ("search", "", 0.01, 0),
            ("search", "", 0.0, 1),
        ]
    );
    assert_eq!(repo.reprice_usage(&settings).await.unwrap(), 0, "nothing left to price");
}

#[tokio::test]
async fn the_monthly_budget_warns_once_at_80_and_at_100_percent() {
    use chrono::TimeZone;
    let (pool, repo) = setup().await;
    let zone = chrono::FixedOffset::east_opt(3 * 3600).unwrap();
    let now = zone.with_ymd_and_hms(2026, 10, 20, 12, 0, 0).unwrap();
    let october = zone.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap().timestamp();
    let spend = |at: i64, usd: f64| llm(at, "t1", "a", usd, 0.0);

    // Off until a budget is set.
    repo.record_usage(&[spend(october - 1, 50.0), spend(october + 10, 0.5)]).await.unwrap();
    assert_eq!(repo.budget_alert(&now).await.unwrap(), None);
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES ('monthly_budget_usd', '1')").execute(&pool).await.unwrap();

    // September's spending does not count; half the budget is not a warning yet.
    let budget = repo.monthly_budget(&now).await.unwrap();
    assert_eq!((budget.limit_usd, budget.spent_usd, budget.month_start), (1.0, 0.5, october));
    assert_eq!(repo.budget_alert(&now).await.unwrap(), None);

    repo.record_usage(&[spend(october + 20, 0.35)]).await.unwrap();
    assert_eq!(repo.budget_alert(&now).await.unwrap().map(|(level, b)| (level, b.spent_usd)), Some((80, 0.85)));
    assert_eq!(repo.budget_alert(&now).await.unwrap(), None, "once per threshold");

    repo.record_usage(&[spend(october + 30, 0.2)]).await.unwrap();
    assert_eq!(repo.budget_alert(&now).await.unwrap().map(|(level, _)| level), Some(100));
    assert_eq!(repo.budget_alert(&now).await.unwrap(), None);

    // A new month starts over.
    let november = zone.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).unwrap();
    assert_eq!(repo.monthly_budget(&november).await.unwrap().spent_usd, 0.0);
    repo.record_usage(&[spend(november.timestamp(), 0.9)]).await.unwrap();
    assert_eq!(repo.budget_alert(&november).await.unwrap().map(|(level, _)| level), Some(80));
}

#[tokio::test]
async fn all_time_starts_at_the_first_recorded_call() {
    let (_, repo) = setup().await;
    assert_eq!(repo.usage_first_at().await.unwrap(), None);
    repo.record_usage(&[llm(5000, "t1", "a", 0.1, 0.0), llm(1200, "t1", "a", 0.1, 0.0)]).await.unwrap();
    assert_eq!(repo.usage_first_at().await.unwrap(), Some(1200));
}
