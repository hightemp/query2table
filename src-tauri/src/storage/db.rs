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

    fn db_path() -> PathBuf {
        let data_dir = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        data_dir.join("query2table").join("data.db")
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

        // Insert default settings if not present
        // Keys must match what backend reads in providers/ and orchestrator/
        let defaults = vec![
            ("theme", "system"),
            // LLM
            ("llm_provider", "openrouter"),
            ("openrouter_model", "openai/gpt-4.1-mini"),
            ("llm_temperature", "0.7"),
            ("llm_max_tokens", "4096"),
            ("ollama_url", "http://localhost:11434"),
            ("ollama_model", "llama3"),
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

        // Rename legacy keys from older databases
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
            "search_results_per_query", "max_pages_per_query",
        ];
        for key in expected_keys {
            let val = db.get_setting(key).await.unwrap();
            assert!(val.is_some(), "Default setting '{}' should exist", key);
        }
    }
}
