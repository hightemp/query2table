//! Keeps the planned search queries of a run and what happened to each, in the database and
//! live in the interface.

use std::sync::Arc;

use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;

use super::events::EventPublisher;
use crate::roles::search_executor::QueryProgress;
use crate::storage::repository::Repository;

/// A search query as the interface shows it.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SearchQueryInfo {
    pub id: String,
    pub query_text: String,
    pub language: String,
    /// `pending`, `running`, `completed`, `failed` or `skipped`.
    pub status: String,
    pub result_count: i64,
    pub error: Option<String>,
}

pub struct SearchLog {
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    queries: Vec<SearchQueryInfo>,
}

impl SearchLog {
    /// Saves the planned queries (text and language) and shows them as waiting.
    pub async fn plan(
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
        run_id: &str,
        planned: &[(String, String)],
        provider: &str,
    ) -> Result<Self, sqlx::Error> {
        let mut queries = Vec::with_capacity(planned.len());
        for (i, (text, language)) in planned.iter().enumerate() {
            let id = repo.create_search_query(run_id, text, language, None, provider, i as i64 / 5).await?;
            queries.push(SearchQueryInfo {
                id,
                query_text: text.clone(),
                language: language.clone(),
                status: "pending".into(),
                result_count: 0,
                error: None,
            });
        }
        if let Some(events) = &events {
            events.emit_search_queries(&queries);
        }
        Ok(Self { repo, events, queries })
    }

    /// Records progress reports until the searches end.
    pub async fn follow(&mut self, mut progress: UnboundedReceiver<QueryProgress>) {
        while let Some(event) = progress.recv().await {
            let (index, status, count, error) = match event {
                QueryProgress::Started { index } => (index, "running", 0, None),
                QueryProgress::Finished { index, kept, .. } => (index, "completed", kept as i64, None),
                QueryProgress::Failed { index, error } => (index, "failed", 0, Some(error)),
                QueryProgress::Skipped { index } => (index, "skipped", 0, None),
            };
            let Some(query) = self.queries.get_mut(index) else { continue };
            query.status = status.into();
            query.result_count = count;
            query.error = error;
            if let Err(e) = self.repo.update_search_query(&query.id, status, count, query.error.as_deref()).await {
                tracing::warn!(error = %e, "Could not save a search query's progress");
            }
            if let Some(events) = &self.events {
                events.emit_search_query(query);
            }
        }
    }

    pub fn queries(&self) -> &[SearchQueryInfo] {
        &self.queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::db::Database;

    #[tokio::test]
    async fn planned_queries_are_saved_and_follow_their_progress() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
        Database::with_pool(pool.clone()).await.migrate().await.unwrap();
        let repo = Arc::new(Repository::new(pool));
        repo.create_run("r", "q", "{}").await.unwrap();
        let planned = vec![("robots".to_string(), "en".to_string()), ("роботы".to_string(), "ru".to_string()), ("late".to_string(), "en".to_string())];
        let mut log = SearchLog::plan(repo.clone(), None, "r", &planned, "brave").await.unwrap();
        assert!(log.queries().iter().all(|q| q.status == "pending"));

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        for event in [
            QueryProgress::Started { index: 0 },
            QueryProgress::Finished { index: 0, results: 10, kept: 4 },
            QueryProgress::Started { index: 1 },
            QueryProgress::Failed { index: 1, error: "HTTP 429".into() },
            QueryProgress::Skipped { index: 2 },
        ] {
            tx.send(event).unwrap();
        }
        drop(tx);
        log.follow(rx).await;

        let saved = repo.get_run_queries("r").await.unwrap();
        let summary: Vec<(String, String, i64, Option<String>)> =
            saved.iter().map(|q| (q.query_text.clone(), q.status.clone(), q.result_count, q.error.clone())).collect();
        assert_eq!(
            summary,
            vec![
                ("robots".into(), "completed".into(), 4, None),
                ("роботы".into(), "failed".into(), 0, Some("HTTP 429".into())),
                ("late".into(), "skipped".into(), 0, None),
            ]
        );
        assert_eq!(saved[1].language, "ru");
    }
}
