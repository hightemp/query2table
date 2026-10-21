//! The usage journal: every model call, search and download with its time, run, tokens, cost and
//! bytes. It survives deleting runs, imports runs made before it once, and folds old calls into
//! daily totals.

use query2table_lib::storage::{
    db::Database,
    repository::Repository,
    usage::{UsageEvent, UsageRun},
};
use serde_json::json;
use sqlx::SqlitePool;

async fn pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    pool
}

fn llm(at: i64, run: &str, model: &str, usd: f64) -> UsageEvent {
    UsageEvent {
        at,
        run_id: Some(run.into()),
        run_type: "table".into(),
        kind: "llm".into(),
        provider: "openrouter".into(),
        model: model.into(),
        calls: 1,
        prompt_tokens: 100,
        completion_tokens: 20,
        reported_usd: usd,
        bytes_sent: 1000,
        bytes_received: 200,
        ..Default::default()
    }
}

async fn insert_run(pool: &SqlitePool, id: &str, run_type: &str, created_at: i64, stats: serde_json::Value) {
    sqlx::query("INSERT INTO runs (id, query, status, config, run_type, stats, created_at, updated_at) VALUES (?, ?, 'completed', '{}', ?, ?, ?, ?)")
        .bind(id)
        .bind(format!("Query {id}"))
        .bind(run_type)
        .bind(stats.to_string())
        .bind(created_at)
        .bind(created_at)
        .execute(pool)
        .await
        .unwrap();
}

fn accounting(lines: serde_json::Value, fetch_calls: u64) -> serde_json::Value {
    json!({"spent_usd": 1, "fetch_calls": fetch_calls, "breakdown": lines})
}

#[tokio::test]
async fn runs_made_before_the_journal_are_imported_once() {
    let pool = pool().await;
    // As if the journal were new: runs exist, the import has not happened yet.
    sqlx::query("DELETE FROM settings WHERE key = 'usage_journal_since'").execute(&pool).await.unwrap();
    insert_run(
        &pool,
        "t1",
        "table",
        1_700_000_000,
        json!({"rows_found": 3, "accounting": accounting(json!([
            {"provider": "openrouter", "model": "m1", "requested_model": "m1", "calls": 3, "reported_usd": 0.3, "estimated_usd": 0.0, "unpriced_calls": 0, "prompt_tokens": 1000, "completion_tokens": 200},
            {"provider": "brave", "model": "web", "requested_model": "web", "calls": 2, "reported_usd": 0.0, "estimated_usd": 0.01, "unpriced_calls": 0, "prompt_tokens": 0, "completion_tokens": 0}
        ]), 5)}),
    )
    .await;
    // Research keeps the cost of each question with that question.
    insert_run(&pool, "r1", "research", 1_700_100_000, json!({"accounting": accounting(json!([]), 0)})).await;
    for (turn, at, model) in [(0, 1_700_100_000, "m2"), (1, 1_702_000_000, "m3")] {
        let turn_accounting = accounting(
            json!([{"provider": "anthropic", "model": model, "requested_model": model, "calls": 2, "reported_usd": 0.0, "estimated_usd": 0.0, "unpriced_calls": 2, "prompt_tokens": 50, "completion_tokens": 5}]),
            1,
        );
        sqlx::query("INSERT INTO research_turns (id, run_id, turn_index, question, status, accounting, created_at) VALUES (?, 'r1', ?, 'q', 'completed', ?, ?)")
            .bind(format!("turn{turn}"))
            .bind(turn)
            .bind(turn_accounting.to_string())
            .bind(at)
            .execute(&pool)
            .await
            .unwrap();
    }
    // Spending of a run in the trash happened all the same.
    insert_run(&pool, "gone", "links", 1_700_200_000, json!({"accounting": accounting(json!([{"provider": "ollama", "model": "m4", "requested_model": "m4", "calls": 1, "reported_usd": 0.0, "estimated_usd": 0.0, "unpriced_calls": 0, "prompt_tokens": 7, "completion_tokens": 1}]), 0)})).await;
    sqlx::query("UPDATE runs SET deleted_at = 1 WHERE id = 'gone'").execute(&pool).await.unwrap();
    insert_run(&pool, "empty", "table", 1_700_300_000, json!({"rows_found": 0})).await;

    let database = Database::with_pool(pool.clone()).await;
    database.migrate().await.unwrap();
    database.migrate().await.unwrap();

    let repo = Repository::new(pool.clone());
    let rows = repo.usage_events(0, i64::MAX).await.unwrap();
    let brief: Vec<_> = rows
        .iter()
        .map(|e| (e.at, e.run_id.as_deref().unwrap(), e.run_type.as_str(), e.kind.as_str(), e.provider.as_str(), e.model.as_str(), e.calls))
        .collect();
    // These runs are older than a year, so they also arrive folded into daily totals.
    assert_eq!(
        brief,
        [
            (1_700_000_000, "t1", "table", "download", "", "", 5),
            (1_700_000_000, "t1", "table", "llm", "openrouter", "m1", 3),
            (1_700_000_000, "t1", "table", "search", "brave", "web", 2),
            (1_700_100_000, "r1", "research", "download", "", "", 1),
            (1_700_100_000, "r1", "research", "llm", "anthropic", "m2", 2),
            (1_700_200_000, "gone", "links", "llm", "ollama", "m4", 1),
            (1_702_000_000, "r1", "research", "download", "", "", 1),
            (1_702_000_000, "r1", "research", "llm", "anthropic", "m3", 2),
        ]
    );
    let first = &rows[1];
    assert_eq!((first.prompt_tokens, first.completion_tokens, first.reported_usd, first.unpriced_calls), (1000, 200, 0.3, 0));
    assert_eq!((rows[2].estimated_usd, rows[4].unpriced_calls), (0.01, 2));
    assert!(rows.iter().all(|e| e.bytes_sent == 0 && e.bytes_received == 0), "bytes were not counted before");

    let runs = repo.usage_runs().await.unwrap();
    assert_eq!(
        runs.iter().map(|r| (r.run_id.as_str(), r.run_type.as_str(), r.title.as_str())).collect::<Vec<_>>(),
        [("empty", "table", "Query empty"), ("gone", "links", "Query gone"), ("r1", "research", "Query r1"), ("t1", "table", "Query t1")]
    );
}

#[tokio::test]
async fn a_new_database_imports_nothing_later() {
    let pool = pool().await;
    insert_run(&pool, "t1", "table", 1_700_000_000, json!({"accounting": accounting(json!([{"provider": "openrouter", "model": "m", "requested_model": "m", "calls": 1, "reported_usd": 0.1, "estimated_usd": 0.0, "unpriced_calls": 0, "prompt_tokens": 1, "completion_tokens": 1}]), 0)})).await;
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    // Runs made after the journal started record their calls as they happen.
    assert!(Repository::new(pool).usage_events(0, i64::MAX).await.unwrap().is_empty());
}

#[tokio::test]
async fn calls_are_recorded_and_runs_keep_their_title_after_deletion() {
    let pool = pool().await;
    let repo = Repository::new(pool.clone());
    insert_run(&pool, "t1", "table", 100, json!({})).await;
    repo.remember_usage_run(&UsageRun { run_id: "t1".into(), run_type: "table".into(), title: "Robot channels".into(), started_at: 100 }).await.unwrap();
    // A later turn of a conversation does not rename the run or move its start.
    repo.remember_usage_run(&UsageRun { run_id: "t1".into(), run_type: "table".into(), title: "Next question".into(), started_at: 900 }).await.unwrap();
    repo.record_usage(&[llm(200, "t1", "m", 0.5), llm(300, "t1", "m", 0.25)]).await.unwrap();
    // Deleting the run keeps the name it had last, given by the user here.
    sqlx::query("UPDATE runs SET title = 'My robots' WHERE id = 't1'").execute(&pool).await.unwrap();
    repo.delete_run("t1").await.unwrap();

    let rows = repo.usage_events(250, 400).await.unwrap();
    assert_eq!(rows, [llm(300, "t1", "m", 0.25)]);
    assert_eq!(repo.usage_events(0, i64::MAX).await.unwrap().len(), 2);
    assert_eq!(repo.usage_runs().await.unwrap(), [UsageRun { run_id: "t1".into(), run_type: "table".into(), title: "My robots".into(), started_at: 100 }]);
}

#[tokio::test]
async fn old_calls_fold_into_daily_totals() {
    let pool = pool().await;
    let repo = Repository::new(pool);
    // Noon UTC: a minute apart is the same local day in every time zone.
    let noon = 1_600_000_000 - 1_600_000_000 % 86_400 + 43_200;
    let mut other = llm(noon + 60, "t1", "m", 0.25);
    other.reasoning_tokens = 5;
    let events = [llm(noon, "t1", "m", 0.5), other, llm(noon, "t2", "m", 1.0), llm(noon + 3 * 86_400, "t1", "m", 2.0), llm(1_700_000_000, "t1", "m", 4.0)];
    repo.record_usage(&events).await.unwrap();

    repo.fold_usage(1_650_000_000).await.unwrap();
    repo.fold_usage(1_650_000_000).await.unwrap();

    let rows = repo.usage_events(0, i64::MAX).await.unwrap();
    let brief: Vec<_> = rows.iter().map(|e| (e.at, e.run_id.as_deref().unwrap(), e.calls, e.reported_usd, e.prompt_tokens, e.reasoning_tokens, e.bytes_sent)).collect();
    assert_eq!(
        brief,
        [
            (noon, "t1", 2, 0.75, 200, 5, 2000),
            (noon, "t2", 1, 1.0, 100, 0, 1000),
            (noon + 3 * 86_400, "t1", 1, 2.0, 100, 0, 1000),
            (1_700_000_000, "t1", 1, 4.0, 100, 0, 1000),
        ]
    );
}

#[tokio::test]
async fn clearing_removes_the_journal() {
    let pool = pool().await;
    let repo = Repository::new(pool);
    repo.remember_usage_run(&UsageRun { run_id: "t1".into(), run_type: "table".into(), title: "T".into(), started_at: 1 }).await.unwrap();
    repo.record_usage(&[llm(10, "t1", "m", 1.0)]).await.unwrap();
    repo.clear_usage().await.unwrap();
    assert!(repo.usage_events(0, i64::MAX).await.unwrap().is_empty());
    assert!(repo.usage_runs().await.unwrap().is_empty());
}

#[tokio::test]
async fn starting_the_app_folds_calls_older_than_a_year() {
    let pool = pool().await;
    let repo = Repository::new(pool.clone());
    let now = chrono::Utc::now().timestamp();
    let noon = now - 400 * 86_400 - now % 86_400 + 43_200;
    repo.record_usage(&[llm(noon, "t1", "m", 1.0), llm(noon + 60, "t1", "m", 1.0), llm(now - 60, "t1", "m", 1.0), llm(now - 30, "t1", "m", 1.0)]).await.unwrap();
    Database::with_pool(pool).await.migrate().await.unwrap();
    let calls: Vec<i64> = repo.usage_events(0, i64::MAX).await.unwrap().iter().map(|e| e.calls).collect();
    assert_eq!(calls, [2, 1, 1], "only calls older than a year are folded");
}
