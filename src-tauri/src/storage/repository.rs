use sqlx::SqlitePool;
use tracing::debug;

use crate::utils::id::new_id;

/// Repository provides CRUD operations for all run-related tables.
pub struct Repository {
    pool: SqlitePool,
}

pub struct EntityMerge {
    pub member_ids: Vec<String>,
    pub group_id: String,
    pub data: serde_json::Value,
    pub confidence: f64,
}

impl Repository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // --- Runs ---

    pub async fn create_run(&self, id: &str, query: &str, config: &str) -> Result<(), sqlx::Error> {
        self.create_run_with_type(id, query, config, "table").await
    }

    pub async fn create_run_with_type(&self, id: &str, query: &str, config: &str, run_type: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO runs (id, query, status, config, run_type, created_at, updated_at) VALUES (?, ?, 'pending', ?, ?, unixepoch(), unixepoch())"
        )
        .bind(id)
        .bind(query)
        .bind(config)
        .bind(run_type)
        .execute(&self.pool)
        .await?;
        debug!(run_id = id, run_type, "Created run");
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

    pub async fn merge_run_stats(&self, run_id: &str, patch_json: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE runs SET stats = json_patch(CASE WHEN json_valid(stats) THEN stats ELSE '{}' END, json(?)), updated_at = unixepoch() WHERE id = ?")
            .bind(patch_json).bind(run_id).execute(&self.pool).await?;
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
            "SELECT id, query, status, config, stats, error, run_type, created_at, updated_at, completed_at FROM runs WHERE id = ?"
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    pub async fn list_runs(&self, limit: i64, offset: i64) -> Result<Vec<RunRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, RunRow>(
            "SELECT id, query, status, config, stats, error, run_type, created_at, updated_at, completed_at FROM runs ORDER BY created_at DESC LIMIT ? OFFSET ?"
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

    /// Apply a complete merge batch atomically, keeping the first member's ID.
    /// Every source record is moved before a duplicate entity is removed.
    pub async fn merge_entity_rows(
        &self,
        run_id: &str,
        groups: &[EntityMerge],
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let mut seen = std::collections::HashSet::new();
        for group in groups {
            if group.member_ids.len() < 2 {
                return Err(sqlx::Error::Protocol(
                    "A merge needs at least two members".into(),
                ));
            }
            for id in &group.member_ids {
                if !seen.insert(id) {
                    return Err(sqlx::Error::Protocol(
                        "An entity belongs to multiple merge groups".into(),
                    ));
                }
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM entity_rows WHERE id = ? AND run_id = ?)",
                )
                .bind(id)
                .bind(run_id)
                .fetch_one(&mut *tx)
                .await?;
                if !exists {
                    return Err(sqlx::Error::Protocol(
                        "Merge member is missing or belongs to another run".into(),
                    ));
                }
            }
            let representative = &group.member_ids[0];
            sqlx::query("UPDATE entity_rows SET data = ?, confidence = ?, dedup_group_id = ?, status = 'deduplicated' WHERE id = ? AND run_id = ?")
                .bind(group.data.to_string()).bind(group.confidence).bind(&group.group_id)
                .bind(representative).bind(run_id).execute(&mut *tx).await?;
            for duplicate in &group.member_ids[1..] {
                sqlx::query("UPDATE row_sources SET entity_row_id = ? WHERE entity_row_id = ?")
                    .bind(representative)
                    .bind(duplicate)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("DELETE FROM entity_rows WHERE id = ? AND run_id = ?")
                    .bind(duplicate)
                    .bind(run_id)
                    .execute(&mut *tx)
                    .await?;
            }
        }
        tx.commit().await?;
        debug!(
            run_id,
            groups = groups.len(),
            "[FIX:dedup] Committed entity merges and preserved source records"
        );
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

    /// Number of saved sources per entity row of a run. Rows without sources are absent.
    pub async fn count_row_sources_by_run(
        &self,
        run_id: &str,
    ) -> Result<std::collections::HashMap<String, i64>, sqlx::Error> {
        let counts: Vec<(String, i64)> = sqlx::query_as(
            "SELECT s.entity_row_id, COUNT(*) FROM row_sources s \
             JOIN entity_rows r ON r.id = s.entity_row_id \
             WHERE r.run_id = ? GROUP BY s.entity_row_id",
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(counts.into_iter().collect())
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

    /// Read only structured LLM diagnostics, independent of the volume of ordinary logs.
    pub async fn get_run_llm_issue_details(&self, run_id: &str) -> Result<Vec<String>, sqlx::Error> {
        let mut details = sqlx::query_scalar::<_, String>(
            "SELECT details FROM run_logs WHERE run_id = ? AND role = 'llm_issue' AND details IS NOT NULL ORDER BY id DESC LIMIT 100"
        ).bind(run_id).fetch_all(&self.pool).await?;
        details.reverse();
        Ok(details)
    }

    pub async fn delete_run(&self, run_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM runs WHERE id = ?")
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        debug!(run_id, "Deleted run");
        Ok(())
    }

    // --- Image Results ---

    pub async fn create_image_result(
        &self,
        run_id: &str,
        image_url: &str,
        thumbnail_url: &str,
        title: &str,
        source_url: &str,
        width: Option<u32>,
        height: Option<u32>,
        relevance_score: Option<f64>,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO image_results (id, run_id, image_url, thumbnail_url, title, source_url, width, height, relevance_score, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(image_url)
        .bind(thumbnail_url)
        .bind(title)
        .bind(source_url)
        .bind(width.map(|v| v as i64))
        .bind(height.map(|v| v as i64))
        .bind(relevance_score)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_image_results(&self, run_id: &str) -> Result<Vec<ImageResultRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ImageResultRow>(
            "SELECT id, run_id, image_url, thumbnail_url, title, source_url, width, height, relevance_score, created_at FROM image_results WHERE run_id = ? ORDER BY relevance_score DESC NULLS LAST, created_at"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn count_image_results(&self, run_id: &str) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM image_results WHERE run_id = ?"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    // --- Link Results ---

    #[allow(clippy::too_many_arguments)]
    pub async fn create_link_result(
        &self,
        run_id: &str,
        url: &str,
        title: &str,
        description: &str,
        reason: &str,
        relevance_score: Option<f64>,
        low_relevance: bool,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO link_results (id, run_id, url, title, description, reason, relevance_score, low_relevance, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(url)
        .bind(title)
        .bind(description)
        .bind(reason)
        .bind(relevance_score)
        .bind(low_relevance)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Marks a link as opened (or clears the mark). Returns false when the link does not exist.
    pub async fn set_link_visited(&self, link_id: &str, visited: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "UPDATE link_results SET visited_at = CASE WHEN ? THEN COALESCE(visited_at, unixepoch()) ELSE NULL END WHERE id = ?"
        )
        .bind(visited)
        .bind(link_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Hides a link from the run's results and exports (or shows it again).
    pub async fn set_link_hidden(&self, link_id: &str, hidden: bool) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("UPDATE link_results SET hidden = ? WHERE id = ?")
            .bind(hidden)
            .bind(link_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_link_results(&self, run_id: &str) -> Result<Vec<LinkResultRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, LinkResultRow>(
            "SELECT id, run_id, url, title, description, reason, relevance_score, low_relevance, hidden, visited_at, created_at FROM link_results WHERE run_id = ? ORDER BY relevance_score DESC NULLS LAST, created_at"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn count_link_results(&self, run_id: &str) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM link_results WHERE run_id = ?"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    // --- Research Steps & Results ---

    pub async fn create_research_step(
        &self,
        run_id: &str,
        turn_index: i64,
        step_index: i64,
        step_type: &str,
        content: &str,
        url: Option<&str>,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO research_steps (id, run_id, turn_index, step_index, step_type, content, url, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(turn_index)
        .bind(step_index)
        .bind(step_type)
        .bind(content)
        .bind(url)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_research_steps(&self, run_id: &str) -> Result<Vec<ResearchStepRow>, sqlx::Error> {
        let rows = sqlx::query_as::<_, ResearchStepRow>(
            "SELECT id, run_id, turn_index, step_index, step_type, content, url, created_at FROM research_steps WHERE run_id = ? ORDER BY turn_index, step_index"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Starts a turn of a research conversation. `limits` is the turn's stop conditions as JSON.
    pub async fn create_research_turn(
        &self,
        run_id: &str,
        turn_index: i64,
        question: &str,
        limits: &str,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO research_turns (id, run_id, turn_index, question, limits, status, created_at) VALUES (?, ?, ?, ?, ?, 'running', unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(turn_index)
        .bind(question)
        .bind(limits)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    /// Records how a turn ended: its answer (if any), suggested follow-ups and status.
    pub async fn finish_research_turn(
        &self,
        run_id: &str,
        turn_index: i64,
        answer_markdown: Option<&str>,
        follow_ups: &[String],
        status: &str,
    ) -> Result<(), sqlx::Error> {
        let follow_ups = serde_json::to_string(follow_ups).unwrap_or_else(|_| "[]".into());
        sqlx::query(
            "UPDATE research_turns SET answer_markdown = COALESCE(?, answer_markdown), follow_ups = ?, status = ? WHERE run_id = ? AND turn_index = ?"
        )
        .bind(answer_markdown)
        .bind(follow_ups)
        .bind(status)
        .bind(run_id)
        .bind(turn_index)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_research_turns(&self, run_id: &str) -> Result<Vec<ResearchTurnRow>, sqlx::Error> {
        sqlx::query_as::<_, ResearchTurnRow>(
            "SELECT id, run_id, turn_index, question, answer_markdown, follow_ups, status, limits, accounting, created_at FROM research_turns WHERE run_id = ? ORDER BY turn_index"
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await
    }

    /// Stores a turn's final usage and cost snapshot (JSON).
    pub async fn set_research_turn_accounting(
        &self,
        run_id: &str,
        turn_index: i64,
        accounting: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE research_turns SET accounting = ? WHERE run_id = ? AND turn_index = ?")
            .bind(accounting)
            .bind(run_id)
            .bind(turn_index)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn next_research_turn_index(&self, run_id: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar::<_, i64>(
            "SELECT COALESCE(MAX(turn_index) + 1, 0) FROM research_turns WHERE run_id = ?"
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await
    }

    pub async fn create_research_result(
        &self,
        run_id: &str,
        answer_markdown: &str,
    ) -> Result<String, sqlx::Error> {
        let id = new_id();
        sqlx::query(
            "INSERT INTO research_results (id, run_id, answer_markdown, created_at) VALUES (?, ?, ?, unixepoch())"
        )
        .bind(&id)
        .bind(run_id)
        .bind(answer_markdown)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn get_research_result(&self, run_id: &str) -> Result<Option<ResearchResultRow>, sqlx::Error> {
        let row = sqlx::query_as::<_, ResearchResultRow>(
            "SELECT id, run_id, answer_markdown, created_at FROM research_results WHERE run_id = ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
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
    pub run_type: String,
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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ImageResultRow {
    pub id: String,
    pub run_id: String,
    pub image_url: String,
    pub thumbnail_url: String,
    pub title: String,
    pub source_url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub relevance_score: Option<f64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LinkResultRow {
    pub id: String,
    pub run_id: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub reason: String,
    pub relevance_score: Option<f64>,
    pub low_relevance: bool,
    pub hidden: bool,
    pub visited_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchStepRow {
    pub id: String,
    pub run_id: String,
    pub turn_index: i64,
    pub step_index: i64,
    pub step_type: String,
    pub content: String,
    pub url: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchTurnRow {
    pub id: String,
    pub run_id: String,
    pub turn_index: i64,
    pub question: String,
    pub answer_markdown: Option<String>,
    /// JSON array of suggested follow-up questions.
    pub follow_ups: String,
    pub status: String,
    /// JSON stop conditions the turn ran with.
    pub limits: String,
    /// JSON usage and cost snapshot when the turn ended.
    pub accounting: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchResultRow {
    pub id: String,
    pub run_id: String,
    pub answer_markdown: String,
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
    async fn dedup_merges_exact_members_and_preserves_all_source_metadata() {
        let (repo, _db) = test_repo().await;
        repo.create_run("merge", "test", "{}").await.unwrap();
        repo.create_run("other", "test", "{}").await.unwrap();
        let query = repo
            .create_search_query("merge", "query", "en", None, "brave", 0)
            .await
            .unwrap();
        let result = repo
            .create_search_result(
                &query,
                "merge",
                "https://example.com/shared",
                "Title",
                "Snippet",
                1,
            )
            .await
            .unwrap();
        let page = repo
            .create_fetched_page(
                &result,
                "merge",
                "https://example.com/shared",
                "success",
                Some("Evidence"),
                Some(8),
                Some(1),
                Some(200),
            )
            .await
            .unwrap();
        let mut ids = Vec::new();
        let mut source_ids = Vec::new();
        for (index, name) in ["Alpha", "Beta", "Alpha", "Gamma", "Beta"]
            .iter()
            .enumerate()
        {
            let id = repo
                .create_entity_row(
                    "merge",
                    &serde_json::json!({"name":name}).to_string(),
                    0.8,
                    "validated",
                )
                .await
                .unwrap();
            let source = repo
                .create_row_source(
                    &id,
                    "https://example.com/shared",
                    Some(&format!("Title {index}")),
                    Some(&format!("Evidence {index}")),
                    Some(&page),
                )
                .await
                .unwrap();
            ids.push(id);
            source_ids.push(source);
        }
        let unrelated = repo
            .create_entity_row("other", r#"{"name":"Elsewhere"}"#, 0.9, "validated")
            .await
            .unwrap();
        let groups = vec![
            EntityMerge {
                member_ids: vec![ids[0].clone(), ids[2].clone()],
                group_id: "alpha".into(),
                data: serde_json::json!({"name":"Alpha","count":0,"enabled":false}),
                confidence: 0.95,
            },
            EntityMerge {
                member_ids: vec![ids[1].clone(), ids[4].clone()],
                group_id: "beta".into(),
                data: serde_json::json!({"name":"Beta"}),
                confidence: 0.9,
            },
        ];
        repo.merge_entity_rows("merge", &groups).await.unwrap();
        let rows = repo.get_entity_rows_by_run("merge").await.unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(repo.count_entity_rows("merge").await.unwrap(), 3);
        let alpha = rows.iter().find(|row| row.id == ids[0]).unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&alpha.data).unwrap(),
            groups[0].data
        );
        assert!(rows
            .iter()
            .any(|row| row.id == ids[3] && row.status == "validated"));
        for (representative, members) in [(&ids[0], vec![0, 2]), (&ids[1], vec![1, 4])] {
            let sources = repo.get_row_sources(representative).await.unwrap();
            assert_eq!(sources.len(), 2);
            for index in members {
                let source = sources
                    .iter()
                    .find(|source| source.id == source_ids[index])
                    .unwrap();
                assert_eq!(
                    source.title.as_deref(),
                    Some(format!("Title {index}").as_str())
                );
                assert_eq!(
                    source.snippet.as_deref(),
                    Some(format!("Evidence {index}").as_str())
                );
                assert_eq!(source.fetched_page_id.as_deref(), Some(page.as_str()));
            }
        }
        assert_eq!(
            repo.get_entity_rows_by_run("other").await.unwrap()[0].id,
            unrelated
        );
        assert!(repo.get_row_sources(&ids[2]).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn invalid_later_merge_rolls_back_every_prior_change() {
        let (repo, _db) = test_repo().await;
        repo.create_run("merge", "test", "{}").await.unwrap();
        let first = repo
            .create_entity_row("merge", r#"{"name":"Alpha"}"#, 0.8, "validated")
            .await
            .unwrap();
        let second = repo
            .create_entity_row("merge", r#"{"name":"Alpha"}"#, 0.8, "validated")
            .await
            .unwrap();
        let source = repo
            .create_row_source(
                &second,
                "https://example.com",
                Some("Title"),
                Some("Snippet"),
                None,
            )
            .await
            .unwrap();
        let groups = vec![
            EntityMerge {
                member_ids: vec![first.clone(), second.clone()],
                group_id: "valid".into(),
                data: serde_json::json!({"name":"Changed"}),
                confidence: 1.0,
            },
            EntityMerge {
                member_ids: vec!["missing".into(), "missing2".into()],
                group_id: "invalid".into(),
                data: serde_json::json!({}),
                confidence: 1.0,
            },
        ];
        assert!(repo.merge_entity_rows("merge", &groups).await.is_err());
        assert_eq!(repo.count_entity_rows("merge").await.unwrap(), 2);
        assert_eq!(repo.get_row_sources(&second).await.unwrap()[0].id, source);
        assert!(repo
            .get_entity_rows_by_run("merge")
            .await
            .unwrap()
            .iter()
            .all(|row| row.status == "validated" && row.data.contains("Alpha")));
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
    async fn research_turns_keep_questions_answers_and_their_steps() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "Find proxies", "{}").await.unwrap();
        assert_eq!(repo.next_research_turn_index("run-1").await.unwrap(), 0);
        repo.create_research_turn("run-1", 0, "Find proxies", r#"{"target_row_count":16}"#).await.unwrap();
        repo.create_research_step("run-1", 0, 0, "search", "proxies", None).await.unwrap();
        repo.finish_research_turn("run-1", 0, Some("Use Proxy-Seller."), &["Cheapest?".to_string()], "completed")
            .await
            .unwrap();
        assert_eq!(repo.next_research_turn_index("run-1").await.unwrap(), 1);
        repo.create_research_turn("run-1", 1, "Which are cheapest?", "{}").await.unwrap();
        repo.create_research_step("run-1", 1, 0, "fetch", "Prices", Some("https://p.example")).await.unwrap();
        repo.finish_research_turn("run-1", 1, None, &[], "cancelled").await.unwrap();

        let turns = repo.get_research_turns("run-1").await.unwrap();
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].question, "Find proxies");
        assert_eq!(turns[0].answer_markdown.as_deref(), Some("Use Proxy-Seller."));
        assert_eq!(turns[0].follow_ups, r#"["Cheapest?"]"#);
        assert_eq!(turns[0].status, "completed");
        assert_eq!(turns[0].limits, r#"{"target_row_count":16}"#);
        assert_eq!((turns[1].status.as_str(), turns[1].answer_markdown.clone()), ("cancelled", None));

        let steps = repo.get_research_steps("run-1").await.unwrap();
        assert_eq!(steps.iter().map(|s| (s.turn_index, s.step_index)).collect::<Vec<_>>(), vec![(0, 0), (1, 0)]);
    }

    #[tokio::test]
    async fn links_keep_reason_relevance_flag_and_review_state() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "links", "{}").await.unwrap();
        let good = repo
            .create_link_result("run-1", "https://a.example", "A", "Docs", "Covers it", Some(0.9), false)
            .await
            .unwrap();
        let weak = repo
            .create_link_result("run-1", "https://b.example", "B", "Blog", "Off topic", Some(0.2), true)
            .await
            .unwrap();

        assert!(repo.set_link_visited(&good, true).await.unwrap());
        let first_visit = repo.get_link_results("run-1").await.unwrap()[0].visited_at;
        assert!(first_visit.is_some());
        // Opening the link again keeps the first visit time.
        repo.set_link_visited(&good, true).await.unwrap();
        assert_eq!(repo.get_link_results("run-1").await.unwrap()[0].visited_at, first_visit);
        assert!(repo.set_link_hidden(&weak, true).await.unwrap());
        assert!(!repo.set_link_hidden("missing", true).await.unwrap());

        let links = repo.get_link_results("run-1").await.unwrap();
        assert_eq!((links[0].id.as_str(), links[0].reason.as_str()), (good.as_str(), "Covers it"));
        assert!(!links[0].low_relevance && !links[0].hidden);
        assert!(links[1].low_relevance && links[1].hidden && links[1].visited_at.is_none());

        repo.set_link_visited(&good, false).await.unwrap();
        repo.set_link_hidden(&weak, false).await.unwrap();
        let links = repo.get_link_results("run-1").await.unwrap();
        assert!(links[0].visited_at.is_none() && !links[1].hidden);
    }

    #[tokio::test]
    async fn counts_sources_per_row_within_one_run() {
        let (repo, _db) = test_repo().await;
        repo.create_run("run-1", "test", "{}").await.unwrap();
        repo.create_run("run-2", "test", "{}").await.unwrap();
        let first = repo.create_entity_row("run-1", "{}", 0.8, "raw").await.unwrap();
        let bare = repo.create_entity_row("run-1", "{}", 0.8, "raw").await.unwrap();
        let other = repo.create_entity_row("run-2", "{}", 0.8, "raw").await.unwrap();
        for url in ["https://a.example", "https://b.example"] {
            repo.create_row_source(&first, url, None, None, None).await.unwrap();
        }
        repo.create_row_source(&other, "https://c.example", None, None, None).await.unwrap();

        let counts = repo.count_row_sources_by_run("run-1").await.unwrap();
        assert_eq!(counts.get(&first), Some(&2));
        assert_eq!(counts.get(&bare), None);
        assert_eq!(counts.len(), 1);
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
