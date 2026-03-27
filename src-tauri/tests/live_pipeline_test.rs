//! Live pipeline test that uses REAL API keys from the app's SQLite database.
//! Run with: cargo test --test live_pipeline_test -- --nocapture
//!
//! Prerequisites:
//! - API keys configured in the app (Settings page)
//! - Internet connection
//!
//! This test is marked #[ignore] by default. Run explicitly with:
//!   cargo test --test live_pipeline_test -- --ignored --nocapture

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use query2table_lib::orchestrator::pipeline::{Pipeline, PipelineConfig, PipelineState};
use query2table_lib::storage::db::Database;
use query2table_lib::storage::repository::Repository;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use common::setup_test_logs;

/// Load settings from the real app database.
async fn load_real_settings() -> HashMap<String, String> {
    // Resolve data dir the same way the app does
    let data_dir = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".local").join("share"))
        })
        .expect("Cannot determine data directory");

    let db_path = data_dir.join("query2table").join("data.db");

    if !db_path.exists() {
        panic!(
            "App database not found at {}. Run the app first and configure API keys.",
            db_path.display()
        );
    }

    let db_url = format!("sqlite:{}?mode=ro", db_path.display());
    let options = SqliteConnectOptions::from_str(&db_url)
        .expect("Invalid DB URL")
        .read_only(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("Failed to open app database");

    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings ORDER BY key",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to read settings");

    rows.into_iter().collect()
}

/// Create a fresh in-memory DB for the test run (don't pollute the real DB).
async fn setup_fresh_db() -> (Arc<Repository>, Database) {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory SQLite pool");
    let db = Database::with_pool(pool).await;
    db.migrate().await.expect("Failed to run migrations");
    let repo = Arc::new(Repository::new(db.pool().clone()));
    (repo, db)
}

// =============================================================================
// Live test: "Найди все репозитории proxy серверов"
// =============================================================================
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore] // Run explicitly: cargo test --test live_pipeline_test -- --ignored --nocapture
async fn test_live_search_proxy_repos() {
    let _log = setup_test_logs("live_search_proxy_repos");

    // Load real API keys from app database
    let settings = load_real_settings().await;

    // Verify keys are present
    let openrouter_key = settings.get("openrouter_api_key").cloned().unwrap_or_default();
    let search_provider = settings.get("search_provider").cloned().unwrap_or_default();
    let serper_key = settings.get("serper_api_key").cloned().unwrap_or_default();
    let brave_key = settings.get("brave_api_key").cloned().unwrap_or_default();

    assert!(!openrouter_key.is_empty(), "OpenRouter API key not configured in app settings");
    match search_provider.as_str() {
        "serper" => assert!(!serper_key.is_empty(), "Serper API key not configured"),
        "brave" => assert!(!brave_key.is_empty(), "Brave API key not configured"),
        other => panic!("Unknown search provider: {}", other),
    }

    eprintln!("=== Live Pipeline Test ===");
    eprintln!("LLM: {} ({})", settings.get("llm_provider").unwrap_or(&"?".into()), settings.get("openrouter_model").unwrap_or(&"?".into()));
    eprintln!("Search: {}", search_provider);
    eprintln!("Query: Найди все репозитории proxy серверов");
    eprintln!("========================\n");

    // Build config from real settings with lower limits for testing
    let mut tweaked = settings.clone();
    tweaked.insert("target_row_count".into(), "5".into());
    tweaked.insert("max_budget_usd".into(), "0.50".into());
    tweaked.insert("max_duration_seconds".into(), "300".into());
    tweaked.insert("max_parallel_fetches".into(), "4".into());
    tweaked.insert("max_parallel_extractions".into(), "2".into());
    tweaked.insert("search_results_per_query".into(), "5".into());
    tweaked.insert("rate_limit_per_domain_ms".into(), "300".into());

    let config = PipelineConfig::from_settings(&tweaked);

    // Create fresh in-memory DB for the run
    let (repo, _db) = setup_fresh_db().await;
    let run_id = format!("live-test-{}", uuid::Uuid::new_v4());

    eprintln!("Run ID: {}", run_id);
    eprintln!("Config: {:?}\n", config);

    let (pipeline, _cmd_tx) = Pipeline::new(
        run_id.clone(),
        "Найди все репозитории proxy серверов".to_string(),
        config,
        repo.clone(),
        None, // No events → auto-confirm schema
    );

    // Run the pipeline
    let start = std::time::Instant::now();
    let result = pipeline.run().await;
    let elapsed = start.elapsed();

    eprintln!("\n=== Results ===");
    eprintln!("Duration: {:.1}s", elapsed.as_secs_f64());

    match &result {
        Ok(state) => {
            eprintln!("State: {:?}", state);
            assert_eq!(*state, PipelineState::Completed, "Pipeline should complete successfully");
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            panic!("Pipeline failed: {}", e);
        }
    }

    // === Inspect DB results ===

    // Run record
    let run = repo.get_run(&run_id).await.unwrap().expect("Run should exist in DB");
    eprintln!("\nRun status: {}", run.status);
    if let Some(stats) = &run.stats {
        let stats_json: serde_json::Value = serde_json::from_str(stats).unwrap_or_default();
        eprintln!("Stats: {}", serde_json::to_string_pretty(&stats_json).unwrap_or_default());
    }

    // Schema
    let schema = repo.get_run_schema(&run_id).await.unwrap();
    if let Some(schema) = &schema {
        eprintln!("\nSchema (confirmed={}): {}", schema.confirmed, schema.columns);
    }

    // Search queries
    let queries = repo.get_search_queries_by_run(&run_id).await.unwrap();
    eprintln!("\nSearch queries ({}): ", queries.len());
    for q in &queries {
        eprintln!("  - [{}] {}", q.language, q.query_text);
    }

    // Search results
    let search_results = repo.get_search_results_by_run(&run_id).await
        .unwrap_or_default();
    eprintln!("\nSearch results: {} URLs found", search_results.len());
    for (i, sr) in search_results.iter().take(10).enumerate() {
        eprintln!("  {}. {} — {}", i + 1, sr.title.as_deref().unwrap_or("?"), sr.url);
    }

    // Fetched pages
    let pages = repo.get_fetched_pages_by_run(&run_id).await
        .unwrap_or_default();
    let ok_pages = pages.iter().filter(|p| p.status == "success").count();
    let failed_pages = pages.iter().filter(|p| p.status == "failed").count();
    eprintln!("\nFetched pages: {} success, {} failed (total {})", ok_pages, failed_pages, pages.len());

    // Entity rows (the main output)
    let rows = repo.get_entity_rows_by_run(&run_id).await.unwrap();
    eprintln!("\n=== Entity Rows: {} ===", rows.len());
    for (i, row) in rows.iter().enumerate() {
        let data: serde_json::Value = serde_json::from_str(&row.data).unwrap_or_default();
        eprintln!("{}. [conf={:.2}] {}", i + 1, row.confidence, serde_json::to_string(&data).unwrap_or_default());

        // Print sources
        let sources = repo.get_row_sources(&row.id).await.unwrap_or_default();
        for src in &sources {
            eprintln!("   src: {}", src.url);
        }
    }

    // Assertions
    assert_eq!(run.status, "completed", "Run should be completed");
    assert!(schema.is_some(), "Schema should exist");
    assert!(schema.unwrap().confirmed == 1, "Schema should be confirmed");
    assert!(!queries.is_empty(), "Should have search queries");
    assert!(!search_results.is_empty(), "Should have search results");
    assert!(!rows.is_empty(), "Should have entity rows (found {} rows)", rows.len());

    eprintln!("\n=== LIVE TEST PASSED ===");
    eprintln!("Found {} entity rows from {} search results in {:.1}s", rows.len(), search_results.len(), elapsed.as_secs_f64());
}
