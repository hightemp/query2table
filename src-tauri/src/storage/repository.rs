use sqlx::SqlitePool;
use tracing::debug;

use crate::utils::id::new_id;

/// Repository provides CRUD operations for all run-related tables.
pub struct Repository {
    pool: SqlitePool,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // --- Runs ---

    pub async fn create_run(&self, id: &str, query: &str, config: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO runs (id, query, status, config, created_at, updated_at) VALUES (?, ?, 'pending', ?, unixepoch(), unixepoch())"
        )
        .bind(id)
        .bind(query)
        .bind(config)
        .execute(&self.pool)
        .await?;
        debug!(run_id = id, "Created run");
        Ok(())
    }

    pub async fn update_run_status(&self, run_id: &str, status: &str) -> Result<(), sqlx::Error> {
        if status == "completed" || status == "failed" || status == "cancelled" {
            sqlx::query(
                "UPDATE runs SET status = ?, updated_at = unixepoch(), completed_at = unixepoch() WHERE id = ?"
            )
            .bind(status)
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE runs SET status = ?, updated_at = unixepoch() WHERE id = ?"
            )
            .bind(status)
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        }
        debug!(run_id, status, "Updated run status");
        Ok(())
    }

    pub async fn update_run_stats(&self, run_id: &str, stats_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET stats = ?, updated_at = unixepoch() WHERE id = ?")
            .bind(stats_json)
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_run_error(&self, run_id: &str, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE runs SET error = ?, status = 'failed', updated_at = unixepoch(), completed_at = unixepoch() WHERE id = ?"
        )
        .bind(error)
        .bind(run_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_run(
        &self,
        run_id: &str,
    ) -> Result<Option<RunRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, RunRow>(
            "SELECT id, query, status, config, stats, error, created_at, updated_at, completed_at FROM runs WHERE id = ?"
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_runs(&self, limit: i64, offset: i64) -> Result<Vec<RunRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunRow>(
            "SELECT id, query, status, config, stats, error, created_at, updated_at, completed_at FROM runs ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Run Schemas ---

    pub async fn create_run_schema(
        &self,
        run_id: &str,
        columns_json: &str,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO run_schemas (id, run_id, columns, confirmed, created_at) VALUES (?, ?, ?, 0, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(columns_json)
        .execute(&self.pool)
        .await?;
        debug!(schema_id = %id, run_id, "Created run schema");
        Ok(id)
    }

    pub async fn confirm_run_schema(&self, run_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE run_schemas SET confirmed = 1 WHERE run_id = ?")
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_run_schema_columns(
        &self,
        run_id: &str,
        columns_json: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE run_schemas SET columns = ? WHERE run_id = ?")
            .bind(columns_json)
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_run_schema(&self, run_id: &str) -> Result<Option<RunSchemaRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, RunSchemaRow>(
            "SELECT id, run_id, columns, confirmed, created_at FROM run_schemas WHERE run_id = ?"
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    // --- Search Queries ---

    pub async fn create_search_query(
        &self,
        run_id: &str,
        query_text: &str,
        language: &str,
        geo_target: Option<&str>,
        provider: &str,
        batch_number: i64,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO search_queries (id, run_id, query_text, language, geo_target, provider, status, batch_number, created_at) VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(query_text)
        .bind(language)
        .bind(geo_target)
        .bind(provider)
        .bind(batch_number)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_search_query_status(
        &self,
        id: &str,
        status: &str,
        result_count: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE search_queries SET status = ?, result_count = ?, executed_at = unixepoch() WHERE id = ?"
        )
        .bind(status)
        .bind(result_count)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_search_queries_by_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<SearchQueryRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SearchQueryRow>(
            "SELECT id, run_id, query_text, language, geo_target, provider, status, result_count, batch_number, created_at, executed_at FROM search_queries WHERE run_id = ? ORDER BY batch_number, created_at"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Search Results ---

    pub async fn create_search_result(
        &self,
        search_query_id: &str,
        run_id: &str,
        url: &str,
        title: &str,
        snippet: &str,
        rank: i64,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO search_results (id, search_query_id, run_id, url, title, snippet, rank, status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', unixepoch())"
        )
        .bind(&id)
        .bind(search_query_id)
        .bind(run_id)
        .bind(url)
        .bind(title)
        .bind(snippet)
        .bind(rank)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_search_result_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE search_results SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_pending_search_results(
        &self,
        run_id: &str,
    ) -> Result<Vec<SearchResultRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SearchResultRow>(
            "SELECT id, search_query_id, run_id, url, title, snippet, rank, status, created_at FROM search_results WHERE run_id = ? AND status = 'pending' ORDER BY rank"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_search_results_by_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<SearchResultRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, SearchResultRow>(
            "SELECT id, search_query_id, run_id, url, title, snippet, rank, status, created_at FROM search_results WHERE run_id = ? ORDER BY rank"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Fetched Pages ---

    pub async fn create_fetched_page(
        &self,
        search_result_id: &str,
        run_id: &str,
        url: &str,
        status: &str,
        content_text: Option<&str>,
        content_length: Option<i64>,
        fetch_duration_ms: Option<i64>,
        http_status: Option<i64>,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO fetched_pages (id, search_result_id, run_id, url, status, content_text, content_length, fetch_duration_ms, http_status, fetched_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(search_result_id)
        .bind(run_id)
        .bind(url)
        .bind(status)
        .bind(content_text)
        .bind(content_length)
        .bind(fetch_duration_ms)
        .bind(http_status)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_fetched_pages_by_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<FetchedPageRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, FetchedPageRow>(
            "SELECT id, search_result_id, run_id, url, status, content_text, content_length, fetch_duration_ms, http_status, fetched_at FROM fetched_pages WHERE run_id = ? ORDER BY fetched_at"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Entity Rows ---

    pub async fn create_entity_row(
        &self,
        run_id: &str,
        data_json: &str,
        confidence: f64,
        status: &str,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO entity_rows (id, run_id, data, confidence, status, created_at) VALUES (?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(data_json)
        .bind(confidence)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn update_entity_row_status(
        &self,
        id: &str,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE entity_rows SET status = ? WHERE id = ?")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_entity_row_dedup(
        &self,
        id: &str,
        dedup_group_id: &str,
        data_json: &str,
        confidence: f64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE entity_rows SET dedup_group_id = ?, data = ?, confidence = ?, status = 'deduplicated' WHERE id = ?"
        )
        .bind(dedup_group_id)
        .bind(data_json)
        .bind(confidence)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_entity_rows_by_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<EntityRowRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, EntityRowRow>(
            "SELECT id, run_id, data, confidence, status, dedup_group_id, created_at FROM entity_rows WHERE run_id = ? ORDER BY created_at"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn get_entity_rows_by_status(
        &self,
        run_id: &str,
        status: &str,
    ) -> Result<Vec<EntityRowRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, EntityRowRow>(
            "SELECT id, run_id, data, confidence, status, dedup_group_id, created_at FROM entity_rows WHERE run_id = ? AND status = ? ORDER BY created_at"
        )
        .bind(run_id)
        .bind(status)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn count_entity_rows(&self, run_id: &str) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM entity_rows WHERE run_id = ? AND status IN ('validated', 'deduplicated', 'final')"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    // --- Row Sources ---

    pub async fn create_row_source(
        &self,
        entity_row_id: &str,
        url: &str,
        title: Option<&str>,
        snippet: Option<&str>,
        fetched_page_id: Option<&str>,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO row_sources (id, entity_row_id, url, title, snippet, fetched_page_id, created_at) VALUES (?, ?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(entity_row_id)
        .bind(url)
        .bind(title)
        .bind(snippet)
        .bind(fetched_page_id)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_row_sources(
        &self,
        entity_row_id: &str,
    ) -> Result<Vec<RowSourceRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RowSourceRow>(
            "SELECT id, entity_row_id, url, title, snippet, fetched_page_id, created_at FROM row_sources WHERE entity_row_id = ? ORDER BY created_at"
        )
        .bind(entity_row_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Run Logs ---

    pub async fn create_run_log(
        &self,
        run_id: &str,
        level: &str,
        role: Option<&str>,
        message: &str,
        details: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO run_logs (run_id, level, role, message, details, created_at) VALUES (?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(run_id)
        .bind(level)
        .bind(role)
        .bind(message)
        .bind(details)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_run_logs(
        &self,
        run_id: &str,
        limit: i64,
    ) -> Result<Vec<RunLogRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunLogRow>(
            "SELECT id, run_id, level, role, message, details, created_at FROM run_logs WHERE run_id = ? ORDER BY id DESC LIMIT ?"
        )
        .bind(run_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // --- Delete run (cascade) ---

    pub async fn delete_run(&self, run_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM runs WHERE id = ?")
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        debug!(run_id, "Deleted run");
        Ok(())
    }
}

// --- Row types for sqlx::FromRow ---

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RunRow {
    pub id: String,
    pub query: String,
    pub status: String,
    pub config: String,
    pub stats: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RunSchemaRow {
    pub id: String,
    pub run_id: String,
    pub columns: String,
    pub confirmed: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchQueryRow {
    pub id: String,
    pub run_id: String,
    pub query_text: String,
    pub language: String,
    pub geo_target: Option<String>,
    pub provider: Option<String>,
    pub status: String,
    pub result_count: Option<i64>,
    pub batch_number: Option<i64>,
    pub created_at: i64,
    pub executed_at: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SearchResultRow {
    pub id: String,
    pub search_query_id: String,
    pub run_id: String,
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub rank: Option<i64>,
    pub status: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FetchedPageRow {
    pub id: String,
    pub search_result_id: String,
    pub run_id: String,
    pub url: String,
    pub status: String,
    pub content_text: Option<String>,
    pub content_length: Option<i64>,
    pub fetch_duration_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub fetched_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EntityRowRow {
    pub id: String,
    pub run_id: String,
    pub data: String,
    pub confidence: f64,
    pub status: String,
    pub dedup_group_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RowSourceRow {
    pub id: String,
    pub entity_row_id: String,
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
    pub fetched_page_id: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RunLogRow {
    pub id: i64,
    pub run_id: Option<String>,
    pub level: String,
    pub role: Option<String>,
    pub message: String,
    pub details: Option<String>,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use crate::storage::db::Database;

    async fn test_repo() -> (Repository, Database) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let db = Database::with_pool(pool.clone()).await;
        db.migrate().await.unwrap();
        (Repository::new(pool), db)
    }

    #[tokio::test]
    async fn test_create_and_get_run() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "find restaurants", "{}").await.unwrap();
        let run = repo.get_run("run-1").await.unwrap().unwrap();
        assert_eq!(run.query, "find restaurants");
        assert_eq!(run.status, "pending");
    }

    #[tokio::test]
    async fn test_update_run_status() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test query", "{}").await.unwrap();
        repo.update_run_status("run-1", "running").await.unwrap();
        let run = repo.get_run("run-1").await.unwrap().unwrap();
        assert_eq!(run.status, "running");
        assert!(run.completed_at.is_none());

        repo.update_run_status("run-1", "completed").await.unwrap();
        let run = repo.get_run("run-1").await.unwrap().unwrap();
        assert_eq!(run.status, "completed");
        assert!(run.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_list_runs() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "query 1", "{}").await.unwrap();
        repo.create_run("run-2", "query 2", "{}").await.unwrap();
        let runs = repo.list_runs(10, 0).await.unwrap();
        assert_eq!(runs.len(), 2);
    }

    #[tokio::test]
    async fn test_run_schema_crud() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        let schema_id = repo.create_run_schema("run-1", r#"[{"name":"company","type":"text","description":"Name","required":true}]"#).await.unwrap();
        assert!(!schema_id.is_empty());

        let schema = repo.get_run_schema("run-1").await.unwrap().unwrap();
        assert_eq!(schema.confirmed, 0);
        assert!(schema.columns.contains("company"));

        repo.confirm_run_schema("run-1").await.unwrap();
        let schema = repo.get_run_schema("run-1").await.unwrap().unwrap();
        assert_eq!(schema.confirmed, 1);
    }

    #[tokio::test]
    async fn test_search_query_and_results() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();

        let sq_id = repo.create_search_query("run-1", "best restaurants NYC", "en", None, "brave", 0)
            .await.unwrap();
        repo.update_search_query_status(&sq_id, "completed", 5).await.unwrap();

        let queries = repo.get_search_queries_by_run("run-1").await.unwrap();
        assert_eq!(queries.len(), 1);
        assert_eq!(queries[0].status, "completed");

        let sr_id = repo.create_search_result(&sq_id, "run-1", "https://example.com", "Example", "A snippet", 1)
            .await.unwrap();
        let results = repo.get_pending_search_results("run-1").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com");

        repo.update_search_result_status(&sr_id, "fetched").await.unwrap();
        let results = repo.get_pending_search_results("run-1").await.unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_fetched_page_crud() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        let sq_id = repo.create_search_query("run-1", "q", "en", None, "brave", 0).await.unwrap();
        let sr_id = repo.create_search_result(&sq_id, "run-1", "https://example.com", "Title", "", 1).await.unwrap();

        let page_id = repo.create_fetched_page(&sr_id, "run-1", "https://example.com", "success", Some("Hello"), Some(5), Some(200), Some(200))
            .await.unwrap();
        assert!(!page_id.is_empty());

        let pages = repo.get_fetched_pages_by_run("run-1").await.unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].content_text.as_deref(), Some("Hello"));
    }

    #[tokio::test]
    async fn test_entity_row_crud() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();

        let row_id = repo.create_entity_row("run-1", r#"{"name":"Acme"}"#, 0.9, "raw").await.unwrap();
        let rows = repo.get_entity_rows_by_run("run-1").await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].confidence, 0.9);

        repo.update_entity_row_status(&row_id, "validated").await.unwrap();
        let validated = repo.get_entity_rows_by_status("run-1", "validated").await.unwrap();
        assert_eq!(validated.len(), 1);

        let count = repo.count_entity_rows("run-1").await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_row_sources() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        let row_id = repo.create_entity_row("run-1", "{}", 0.8, "raw").await.unwrap();

        let src_id = repo.create_row_source(&row_id, "https://example.com", Some("Example"), None, None)
            .await.unwrap();
        assert!(!src_id.is_empty());

        let sources = repo.get_row_sources(&row_id).await.unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].url, "https://example.com");
    }

    #[tokio::test]
    async fn test_run_logs() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();

        repo.create_run_log("run-1", "INFO", Some("interpreter"), "Parsed query", None)
            .await.unwrap();
        repo.create_run_log("run-1", "DEBUG", Some("planner"), "Generated schema", Some("details"))
            .await.unwrap();

        let logs = repo.get_run_logs("run-1", 10).await.unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_run_cascades() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        repo.create_run_log("run-1", "INFO", None, "msg", None).await.unwrap();
        repo.create_entity_row("run-1", "{}", 0.5, "raw").await.unwrap();

        repo.delete_run("run-1").await.unwrap();
        let run = repo.get_run("run-1").await.unwrap();
        assert!(run.is_none());

        let logs = repo.get_run_logs("run-1", 10).await.unwrap();
        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    async fn test_update_run_error() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        repo.update_run_error("run-1", "Something went wrong").await.unwrap();
        let run = repo.get_run("run-1").await.unwrap().unwrap();
        assert_eq!(run.status, "failed");
        assert_eq!(run.error.as_deref(), Some("Something went wrong"));
        assert!(run.completed_at.is_some());
    }
}
