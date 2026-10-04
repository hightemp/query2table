use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteConnectOptions};
use std::path::PathBuf;
use std::str::FromStr;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_path = Self::db_path();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let options = SqliteConnectOptions::from_str(&db_url)?
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Create a Database with a custom pool (for testing)
    pub async fn with_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn data_dir() -> PathBuf {
        let data_dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        data_dir.join("query2table")
    }

    pub fn db_path() -> PathBuf {
        Self::data_dir().join("data.db")
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS runs (
                id TEXT PRIMARY KEY,
                query TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                config TEXT NOT NULL DEFAULT '{}',
                stats TEXT,
                error TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                completed_at INTEGER
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS run_schemas (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                columns TEXT NOT NULL DEFAULT '[]',
                confirmed INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS search_queries (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                query_text TEXT NOT NULL,
                language TEXT DEFAULT 'en',
                geo_target TEXT,
                provider TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                result_count INTEGER DEFAULT 0,
                batch_number INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                executed_at INTEGER
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS search_results (
                id TEXT PRIMARY KEY,
                search_query_id TEXT NOT NULL REFERENCES search_queries(id) ON DELETE CASCADE,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                url TEXT NOT NULL,
                title TEXT,
                snippet TEXT,
                rank INTEGER,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS fetched_pages (
                id TEXT PRIMARY KEY,
                search_result_id TEXT NOT NULL REFERENCES search_results(id) ON DELETE CASCADE,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                content_text TEXT,
                content_length INTEGER,
                fetch_duration_ms INTEGER,
                http_status INTEGER,
                fetched_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS entity_rows (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                data TEXT NOT NULL DEFAULT '{}',
                confidence REAL NOT NULL DEFAULT 0.0,
                status TEXT NOT NULL DEFAULT 'raw',
                dedup_group_id TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS row_sources (
                id TEXT PRIMARY KEY,
                entity_row_id TEXT NOT NULL REFERENCES entity_rows(id) ON DELETE CASCADE,
                url TEXT NOT NULL,
                title TEXT,
                snippet TEXT,
                fetched_page_id TEXT REFERENCES fetched_pages(id),
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS run_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_id TEXT REFERENCES runs(id) ON DELETE CASCADE,
                level TEXT NOT NULL,
                role TEXT,
                message TEXT NOT NULL,
                details TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        // Indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_runs_status ON runs(status)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_search_queries_run_id ON search_queries(run_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_search_results_run_id ON search_results(run_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_search_results_status ON search_results(status)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_fetched_pages_run_id ON fetched_pages(run_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entity_rows_run_id ON entity_rows(run_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_entity_rows_status ON entity_rows(status)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_row_sources_entity_row_id ON row_sources(entity_row_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_run_logs_run_id ON run_logs(run_id)")
            .execute(&self.pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_run_logs_level ON run_logs(level)")
            .execute(&self.pool).await?;

        // Image search results table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS image_results (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                image_url TEXT NOT NULL,
                thumbnail_url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                source_url TEXT NOT NULL DEFAULT '',
                width INTEGER,
                height INTEGER,
                relevance_score REAL,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_image_results_run_id ON image_results(run_id)")
            .execute(&self.pool).await?;

        // Link search results table (relevance-filtered web pages)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS link_results (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                url TEXT NOT NULL,
                title TEXT NOT NULL DEFAULT '',
                description TEXT NOT NULL DEFAULT '',
                relevance_score REAL,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_link_results_run_id ON link_results(run_id)")
            .execute(&self.pool).await?;

        // Link review state and the model's match explanation (for existing databases).
        for column in [
            "reason TEXT NOT NULL DEFAULT ''",
            // Scored below the run's relevance threshold: kept, but collapsed and not exported.
            "low_relevance INTEGER NOT NULL DEFAULT 0",
            "hidden INTEGER NOT NULL DEFAULT 0",
            "visited_at INTEGER",
        ] {
            sqlx::query(&format!("ALTER TABLE link_results ADD COLUMN {column}"))
                .execute(&self.pool)
                .await
                .ok(); // ok() — ignore error if column already exists
        }

        // Research agent steps table (search / fetch / think transcript)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS research_steps (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                step_index INTEGER NOT NULL,
                step_type TEXT NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                url TEXT,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_research_steps_run_id ON research_steps(run_id)")
            .execute(&self.pool).await?;

        // Research final answer table (one markdown answer per run)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS research_results (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                answer_markdown TEXT NOT NULL DEFAULT '',
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            )"
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_research_results_run_id ON research_results(run_id)")
            .execute(&self.pool).await?;

        // Add run_type column to runs table (for existing databases)
        sqlx::query(
            "ALTER TABLE runs ADD COLUMN run_type TEXT NOT NULL DEFAULT 'table'"
        )
        .execute(&self.pool)
        .await
        .ok(); // ok() — ignore error if column already exists

        // Research conversations: each question and its answer is a turn of the run.
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS research_turns (
                id TEXT PRIMARY KEY,
                run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
                turn_index INTEGER NOT NULL,
                question TEXT NOT NULL,
                answer_markdown TEXT,
                follow_ups TEXT NOT NULL DEFAULT '[]',
                status TEXT NOT NULL DEFAULT 'running',
                limits TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                UNIQUE (run_id, turn_index)
            )"
        )
        .execute(&self.pool)
        .await?;
        // Final usage and cost of each turn; the run's own accounting describes the latest turn.
        sqlx::query("ALTER TABLE research_turns ADD COLUMN accounting TEXT")
            .execute(&self.pool)
            .await
            .ok(); // ok() — ignore error if column already exists
        sqlx::query("ALTER TABLE research_steps ADD COLUMN turn_index INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await
            .ok(); // ok() — ignore error if column already exists
        // Notices the reader marked as read (see `set_run_dismissed_notices`).
        sqlx::query("ALTER TABLE runs ADD COLUMN dismissed_notices TEXT")
            .execute(&self.pool)
            .await
            .ok(); // ok() — ignore error if column already exists
        // History: a user-given name, pinning, and deletion that can be undone.
        for column in ["title TEXT", "pinned_at INTEGER", "deleted_at INTEGER"] {
            sqlx::query(&format!("ALTER TABLE runs ADD COLUMN {column}"))
                .execute(&self.pool)
                .await
                .ok(); // ok() — ignore error if column already exists
        }
        // Research runs saved before conversations become one-turn conversations.
        sqlx::query(
            "INSERT INTO research_turns (id, run_id, turn_index, question, answer_markdown, status, created_at)
             SELECT lower(hex(randomblob(16))), r.id, 0, r.query,
                    (SELECT rr.answer_markdown FROM research_results rr WHERE rr.run_id = r.id ORDER BY rr.created_at DESC LIMIT 1),
                    r.status, r.created_at
             FROM runs r
             WHERE r.run_type = 'research'
               AND NOT EXISTS (SELECT 1 FROM research_turns t WHERE t.run_id = r.id)"
        )
        .execute(&self.pool)
        .await?;

        // Rename legacy keys before inserting defaults; defaults must not shadow saved values.
        let renames = vec![
            ("default_model", "openrouter_model"),
            ("primary_search_provider", "search_provider"),
            ("ollama_base_url", "ollama_url"),
            ("max_results_per_query", "search_results_per_query"),
        ];
        for (old_key, new_key) in renames {
            sqlx::query(
                "UPDATE OR IGNORE settings SET key = ? WHERE key = ?"
            )
            .bind(new_key)
            .bind(old_key)
            .execute(&self.pool)
            .await?;
            // Clean up old key if rename failed due to new key already existing
            sqlx::query("DELETE FROM settings WHERE key = ?")
                .bind(old_key)
                .execute(&self.pool)
                .await?;
        }
        // Insert default settings if not present
        // Keys must match what backend reads in providers/ and orchestrator/
        let defaults = vec![
            ("theme", "system"),
            ("notifications_enabled", "true"),
            ("show_site_icons", "true"),
            // LLM
            ("llm_provider", "openrouter"),
            ("openrouter_model", "openai/gpt-4.1-mini"),
            ("llm_temperature", "0.7"),
            ("llm_max_tokens", "4096"),
            ("llm_reasoning_effort", "auto"),
            ("llm_pricing_overrides", "{}"),
            ("brave_price_per_1000", "0"),
            ("serper_price_per_1000", "0"),
            ("ollama_url", "http://localhost:11434"),
            ("ollama_model", "llama3"),
            ("ollama_cloud_url", "https://ollama.com"),
            ("ollama_cloud_model", "gpt-oss:120b"),
            ("openai_base_url", "http://localhost:8080/v1"),
            ("openai_model", ""),
            ("openai_json_mode", "true"),
            // Search
            ("search_provider", "brave"),
            ("search_fallback_enabled", "true"),
            ("search_results_per_query", "20"),
            ("max_pages_per_query", "10"),
            // Execution
            ("max_parallel_fetches", "8"),
            ("max_parallel_extractions", "3"),
            ("fetch_timeout_seconds", "15"),
            ("rate_limit_per_domain_ms", "2000"),
            ("respect_robots_txt", "true"),
            ("max_page_size_kb", "5000"),
            // Network / Proxy
            ("proxy_list", "[]"),
            ("active_proxy_url", ""),
            // Content processing
            ("enable_content_truncation", "true"),
            ("max_extraction_text_chars", "12000"),
            ("max_pdf_text_chars", "500000"),
            // Quality
            ("precision_recall", "balanced"),
            ("evidence_strictness", "moderate"),
            ("min_confidence_threshold", "0.5"),
            ("enable_semantic_validation", "true"),
            ("dedup_similarity_threshold", "0.85"),
            // Stop conditions
            ("target_row_count", "50"),
            ("max_budget_usd", "1.00"),
            ("max_duration_seconds", "600"),
            ("saturation_threshold", "0.05"),
            // Export
            ("default_export_format", "csv"),
        ];

        for (key, value) in defaults {
            sqlx::query(
                "INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)"
            )
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await?;
        }
        // Versioned data migrations run once: a later deliberately blank rate must
        // remain unknown on restart, while the former empty defaults become zero.
        let mut tx = self.pool.begin().await?;
        let version: i64 = sqlx::query_scalar("PRAGMA user_version")
            .fetch_one(&mut *tx).await?;
        if version < 1 {
            let changed = sqlx::query(
                "UPDATE settings SET value = '0', updated_at = unixepoch()
                 WHERE key IN ('brave_price_per_1000', 'serper_price_per_1000')
                   AND trim(value) = ''"
            ).execute(&mut *tx).await?.rows_affected();
            sqlx::query("PRAGMA user_version = 1").execute(&mut *tx).await?;
            tracing::debug!(changed, "[FIX:search-pricing] Migrated empty search defaults to zero");
        }
        tx.commit().await?;

        Ok(())
    }

    // --- Settings CRUD ---

    pub async fn get_all_settings(&self) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, (String, String)>(
            "SELECT key, value FROM settings ORDER BY key"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query_scalar::<_, String>(
            "SELECT value FROM settings WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, unixepoch())
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at"
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

fn dirs_next() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local").join("share"))
            })
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn existing_research_runs_become_one_turn_conversations() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        // Tables as written by earlier versions.
        for sql in [
            "CREATE TABLE runs (id TEXT PRIMARY KEY, query TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', config TEXT NOT NULL DEFAULT '{}', stats TEXT, error TEXT, created_at INTEGER NOT NULL DEFAULT 0, updated_at INTEGER NOT NULL DEFAULT 0, completed_at INTEGER, run_type TEXT NOT NULL DEFAULT 'table')",
            "CREATE TABLE research_steps (id TEXT PRIMARY KEY, run_id TEXT NOT NULL, step_index INTEGER NOT NULL, step_type TEXT NOT NULL, content TEXT NOT NULL DEFAULT '', url TEXT, created_at INTEGER NOT NULL DEFAULT 0)",
            "CREATE TABLE research_results (id TEXT PRIMARY KEY, run_id TEXT NOT NULL, answer_markdown TEXT NOT NULL DEFAULT '', created_at INTEGER NOT NULL DEFAULT 0)",
            "INSERT INTO runs (id, query, status, run_type, created_at) VALUES ('done', 'Find proxies', 'completed', 'research', 5), ('stuck', 'C++ memory', 'running', 'research', 6), ('table', 'Companies', 'completed', 'table', 7)",
            "INSERT INTO research_steps (id, run_id, step_index, step_type, content) VALUES ('s1', 'done', 0, 'search', 'q')",
            "INSERT INTO research_results (id, run_id, answer_markdown) VALUES ('r1', 'done', 'Use Proxy-Seller.')",
        ] {
            sqlx::query(sql).execute(&pool).await.unwrap();
        }

        let db = Database::with_pool(pool).await;
        db.migrate().await.unwrap();
        db.migrate().await.unwrap();
        let repo = crate::storage::repository::Repository::new(db.pool().clone());
        let done = repo.get_research_turns("done").await.unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].question, "Find proxies");
        assert_eq!(done[0].answer_markdown.as_deref(), Some("Use Proxy-Seller."));
        assert_eq!(done[0].status, "completed");
        let stuck = repo.get_research_turns("stuck").await.unwrap();
        assert_eq!((stuck[0].status.as_str(), stuck[0].answer_markdown.clone()), ("running", None));
        assert!(repo.get_research_turns("table").await.unwrap().is_empty());
        assert_eq!(repo.get_research_steps("done").await.unwrap()[0].turn_index, 0);
    }

    #[tokio::test]
    async fn link_review_columns_are_added_to_existing_databases() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::query("CREATE TABLE runs (id TEXT PRIMARY KEY, query TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'pending', config TEXT NOT NULL DEFAULT '{}', stats TEXT, error TEXT, created_at INTEGER NOT NULL DEFAULT 0, updated_at INTEGER NOT NULL DEFAULT 0, completed_at INTEGER)")
            .execute(&pool).await.unwrap();
        sqlx::query("CREATE TABLE link_results (id TEXT PRIMARY KEY, run_id TEXT NOT NULL, url TEXT NOT NULL, title TEXT NOT NULL DEFAULT '', description TEXT NOT NULL DEFAULT '', relevance_score REAL, created_at INTEGER NOT NULL DEFAULT 0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO runs (id, query) VALUES ('old', 'q')").execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO link_results (id, run_id, url, description, relevance_score) VALUES ('l1', 'old', 'https://a.example', 'Saved before', 0.8)")
            .execute(&pool).await.unwrap();

        let db = Database::with_pool(pool).await;
        db.migrate().await.unwrap();
        db.migrate().await.unwrap();
        let repo = crate::storage::repository::Repository::new(db.pool().clone());
        let links = repo.get_link_results("old").await.unwrap();
        assert_eq!(links[0].description, "Saved before");
        assert_eq!(links[0].reason, "");
        assert!(!links[0].low_relevance && !links[0].hidden && links[0].visited_at.is_none());
    }

    async fn test_db() -> Database {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let db = Database::with_pool(pool).await;
        db.migrate().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_migrate_creates_tables() {
        let db = test_db().await;
        // Check settings table exists and has default values
        let settings = db.get_all_settings().await.unwrap();
        assert!(!settings.is_empty(), "Default settings should be inserted");
    }

    #[tokio::test]
    async fn test_get_default_setting() {
        let db = test_db().await;
        let theme = db.get_setting("theme").await.unwrap();
        assert_eq!(theme, Some("system".to_string()));
    }

    #[tokio::test]
    async fn search_prices_default_to_zero_and_upgrade_only_once() {
        let db = test_db().await;
        let keys = ["brave_price_per_1000", "serper_price_per_1000"];
        for key in keys {
            assert_eq!(db.get_setting(key).await.unwrap().as_deref(), Some("0"));
            db.set_setting(key, "").await.unwrap();
        }
        // Simulate the previous version with both old empty defaults.
        sqlx::query("PRAGMA user_version = 0").execute(db.pool()).await.unwrap();
        sqlx::query("INSERT INTO runs(id, query, stats) VALUES ('old', 'fixture', '{\"accounting\":{\"unpriced_calls\":50}}')")
            .execute(db.pool()).await.unwrap();
        db.migrate().await.unwrap();
        for key in keys {
            assert_eq!(db.get_setting(key).await.unwrap().as_deref(), Some("0"));
        }
        let stats: String = sqlx::query_scalar("SELECT stats FROM runs WHERE id = 'old'")
            .fetch_one(db.pool()).await.unwrap();
        assert_eq!(stats, "{\"accounting\":{\"unpriced_calls\":50}}");
        // Clearing a price deliberately after upgrading must survive the next startup.
        db.set_setting(keys[0], "").await.unwrap();
        db.set_setting(keys[1], "2.5").await.unwrap();
        db.migrate().await.unwrap();
        assert_eq!(db.get_setting(keys[0]).await.unwrap().as_deref(), Some(""));
        assert_eq!(db.get_setting(keys[1]).await.unwrap().as_deref(), Some("2.5"));
    }

    #[tokio::test]
    async fn search_price_upgrade_preserves_paid_rates_and_fills_missing_defaults() {
        for paid_key in ["brave_price_per_1000", "serper_price_per_1000"] {
            let db = test_db().await;
            let other = if paid_key == "brave_price_per_1000" { "serper_price_per_1000" } else { "brave_price_per_1000" };
            db.set_setting(paid_key, "4.50").await.unwrap();
            sqlx::query("DELETE FROM settings WHERE key = ?").bind(other).execute(db.pool()).await.unwrap();
            sqlx::query("PRAGMA user_version = 0").execute(db.pool()).await.unwrap();
            db.migrate().await.unwrap();
            assert_eq!(db.get_setting(paid_key).await.unwrap().as_deref(), Some("4.50"));
            assert_eq!(db.get_setting(other).await.unwrap().as_deref(), Some("0"));
        }
    }

    #[tokio::test]
    async fn test_set_and_get_setting() {
        let db = test_db().await;
        db.set_setting("theme", "dark").await.unwrap();
        let theme = db.get_setting("theme").await.unwrap();
        assert_eq!(theme, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_get_nonexistent_setting() {
        let db = test_db().await;
        let result = db.get_setting("nonexistent_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_set_new_setting() {
        let db = test_db().await;
        db.set_setting("custom_key", "custom_value").await.unwrap();
        let value = db.get_setting("custom_key").await.unwrap();
        assert_eq!(value, Some("custom_value".to_string()));
    }

    #[tokio::test]
    async fn test_update_existing_setting() {
        let db = test_db().await;
        db.set_setting("theme", "dark").await.unwrap();
        db.set_setting("theme", "light").await.unwrap();
        let theme = db.get_setting("theme").await.unwrap();
        assert_eq!(theme, Some("light".to_string()));
    }

    #[tokio::test]
    async fn test_all_default_settings_present() {
        let db = test_db().await;
        let expected_keys = vec![
            "theme", "llm_provider", "openrouter_model", "search_provider",
            "target_row_count", "max_budget_usd", "max_duration_seconds",
            "ollama_url", "llm_temperature", "llm_max_tokens", "ollama_model",
            "llm_reasoning_effort", "notifications_enabled",
            "search_results_per_query", "max_pages_per_query",
        ];
        for key in expected_keys {
            let val = db.get_setting(key).await.unwrap();
            assert!(val.is_some(), "Default setting '{}' should exist", key);
        }
    }

    #[tokio::test]
    async fn legacy_settings_are_renamed_before_defaults_and_migration_is_repeatable() {
        let db = test_db().await;
        let legacy = [
            ("default_model", "openrouter_model", "saved-model"),
            ("primary_search_provider", "search_provider", "serper"),
            ("ollama_base_url", "ollama_url", "http://saved-server:11434"),
            ("max_results_per_query", "search_results_per_query", "7"),
        ];
        for (old, new, value) in legacy {
            sqlx::query("DELETE FROM settings WHERE key = ?").bind(new).execute(db.pool()).await.unwrap();
            db.set_setting(old, value).await.unwrap();
        }
        sqlx::query("DELETE FROM settings WHERE key = 'llm_reasoning_effort'").execute(db.pool()).await.unwrap();
        db.migrate().await.unwrap();
        for (old, new, value) in legacy {
            assert_eq!(db.get_setting(new).await.unwrap().as_deref(), Some(value));
            assert_eq!(db.get_setting(old).await.unwrap(), None);
        }
        assert_eq!(db.get_setting("llm_reasoning_effort").await.unwrap().as_deref(), Some("auto"));
        db.set_setting("llm_reasoning_effort", "high").await.unwrap();
        let saved = db.get_all_settings().await.unwrap();
        db.migrate().await.unwrap();
        assert_eq!(db.get_all_settings().await.unwrap(), saved);
    }

    #[tokio::test]
    async fn migration_keeps_current_value_when_legacy_and_current_keys_both_exist() {
        let db = test_db().await;
        db.set_setting("default_model", "older-model").await.unwrap();
        db.set_setting("openrouter_model", "current-model").await.unwrap();
        db.migrate().await.unwrap();
        assert_eq!(db.get_setting("openrouter_model").await.unwrap().as_deref(), Some("current-model"));
        assert_eq!(db.get_setting("default_model").await.unwrap(), None);
    }
}
