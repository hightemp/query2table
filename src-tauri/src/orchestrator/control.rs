use std::collections::VecDeque;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinSet;
use tracing::{error, info};

use crate::providers::llm::types::LlmIssue;
use crate::storage::models::SchemaColumn;
use crate::storage::repository::Repository;

use super::events::EventPublisher;
use super::pipeline::{PipelineCommand, PipelineState};

struct StageUpdate {
    status: String,
    acknowledged: oneshot::Sender<()>,
}

/// The pipeline reports its stage; the supervisor owns status writes so a
/// suspended database write cannot overwrite a user's pause or cancellation.
#[derive(Clone)]
pub(crate) struct RunControl {
    stage_tx: mpsc::Sender<StageUpdate>,
    paused: watch::Receiver<bool>,
    schema: watch::Receiver<Option<Vec<SchemaColumn>>>,
    issues: mpsc::UnboundedSender<LlmIssue>,
}

pub(crate) struct RunSupervisor {
    run_id: String,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    commands: mpsc::Receiver<PipelineCommand>,
    stages: mpsc::Receiver<StageUpdate>,
    paused: watch::Sender<bool>,
    schema: watch::Sender<Option<Vec<SchemaColumn>>>,
    issues: mpsc::UnboundedReceiver<LlmIssue>,
}

impl RunControl {
    pub(crate) fn new(
        run_id: String,
        repo: Arc<Repository>,
        events: Option<EventPublisher>,
        commands: mpsc::Receiver<PipelineCommand>,
    ) -> (Self, RunSupervisor) {
        let (stage_tx, stages) = mpsc::channel(1);
        let (paused_tx, paused) = watch::channel(false);
        let (schema_tx, schema) = watch::channel(None);
        let (issue_tx, issues) = mpsc::unbounded_channel();
        (
            Self {
                stage_tx,
                paused,
                schema,
                issues: issue_tx,
            },
            RunSupervisor {
                run_id,
                repo,
                events,
                commands,
                stages,
                paused: paused_tx,
                schema: schema_tx,
                issues,
            },
        )
    }

    pub(crate) async fn set_status(&self, status: &str) {
        let (acknowledged, ack) = oneshot::channel();
        if self
            .stage_tx
            .send(StageUpdate {
                status: status.to_string(),
                acknowledged,
            })
            .await
            .is_ok()
        {
            let _ = ack.await;
        }
    }

    pub(crate) fn pause_signal(&self) -> watch::Receiver<bool> {
        self.paused.clone()
    }

    pub(crate) fn issue_sender(&self) -> mpsc::UnboundedSender<LlmIssue> {
        self.issues.clone()
    }

    pub(crate) async fn confirmed_schema(&self) -> Option<Vec<SchemaColumn>> {
        let mut schema = self.schema.clone();
        loop {
            if let Some(columns) = schema.borrow_and_update().clone() {
                return Some(columns);
            }
            if schema.changed().await.is_err() {
                return None;
            }
        }
    }

    /// A pause resets the schema review's idle timeout. Its wall-clock timer
    /// must not auto-confirm immediately when a long-paused run is resumed.
    pub(crate) async fn schema_idle_timeout(&self, duration: Duration) {
        let mut paused = self.paused.clone();
        let delay = tokio::time::sleep(duration);
        tokio::pin!(delay);
        loop {
            tokio::select! {
                biased;
                changed = paused.changed() => {
                    if changed.is_err() {
                        return;
                    }
                    delay.as_mut().reset(tokio::time::Instant::now() + duration);
                }
                () = &mut delay => return,
            }
        }
    }
}

/// Persistence progresses alongside the pipeline, never inline in a command
/// handler: a suspended SQLx future may still own the pool's only connection.
struct ControlWriter {
    run_id: String,
    repo: Arc<Repository>,
    queue: VecDeque<ControlWrite>,
    active: Option<Pin<Box<dyn Future<Output = ()> + Send>>>,
}

enum ControlWrite {
    Status {
        status: String,
        acknowledged: Option<oneshot::Sender<()>>,
    },
    Failure(String),
    Log {
        level: &'static str,
        role: &'static str,
        message: String,
        details: Option<String>,
    },
}

impl ControlWriter {
    fn new(run_id: String, repo: Arc<Repository>) -> Self {
        Self {
            run_id,
            repo,
            queue: VecDeque::new(),
            active: None,
        }
    }

    fn pending(&self) -> bool {
        self.active.is_some() || !self.queue.is_empty()
    }

    async fn progress(&mut self) {
        if self.active.is_none() {
            let Some(write) = self.queue.pop_front() else {
                return;
            };
            let repo = self.repo.clone();
            let run_id = self.run_id.clone();
            // The future lives in the writer, so losing a select! branch does
            // not cancel a database operation or discard a queued write.
            self.active = Some(Box::pin(async move {
                let (result, acknowledged) = match write {
                    ControlWrite::Status {
                        status,
                        acknowledged,
                    } => (repo.update_run_status(&run_id, &status).await, acknowledged),
                    ControlWrite::Failure(message) => {
                        (repo.update_run_error(&run_id, &message).await, None)
                    }
                    ControlWrite::Log {
                        level,
                        role,
                        message,
                        details,
                    } => (
                        repo.create_run_log(
                            &run_id,
                            level,
                            Some(role),
                            &message,
                            details.as_deref(),
                        )
                        .await
                        .map(|_| ()),
                        None,
                    ),
                };
                if let Err(e) = result {
                    error!(run_id = %run_id, error = %e, "Failed to persist run control update");
                }
                if let Some(acknowledged) = acknowledged {
                    let _ = acknowledged.send(());
                }
            }));
        }
        if let Some(active) = &mut self.active {
            active.await;
        }
        self.active = None;
    }

    async fn flush(&mut self) {
        while self.pending() {
            self.progress().await;
        }
    }
}

impl RunSupervisor {
    /// Keep the same pinned future across pause/resume. In-flight requests are
    /// retained; cancelling drops them and the scoped worker pools immediately.
    pub(crate) async fn run<F, E>(mut self, work: F) -> Result<PipelineState, E>
    where
        F: Future<Output = Result<PipelineState, E>>,
        E: Display,
    {
        let mut work = Box::pin(work);
        let mut paused = false;
        let mut commands_open = true;
        let mut stage = "pending".to_string();
        let mut writes = ControlWriter::new(self.run_id.clone(), self.repo.clone());

        loop {
            tokio::select! {
                biased;
                command = self.commands.recv(), if commands_open => {
                    match command {
                        Some(_) if !paused && matches!(stage.as_str(), "completed" | "failed" | "cancelled") => {}
                        Some(PipelineCommand::Cancel) => {
                            drop(work);
                            self.drain_issues(&mut writes);
                            self.publish_status("cancelled", &mut writes, None);
                            self.log("INFO", "[FIX:run-control] Cancelled active pipeline and worker requests", &mut writes);
                            writes.flush().await;
                            return Ok(PipelineState::Cancelled);
                        }
                        Some(PipelineCommand::Pause) if !paused => {
                            paused = true;
                            self.paused.send_replace(true);
                            self.publish_status("paused", &mut writes, None);
                            self.log("INFO", &format!("[FIX:run-control] Pipeline paused at {stage}"), &mut writes);
                        }
                        Some(PipelineCommand::Resume) if paused => {
                            if !matches!(stage.as_str(), "completed" | "failed" | "cancelled") {
                                self.publish_status(&stage, &mut writes, None);
                            }
                            paused = false;
                            self.paused.send_replace(false);
                            self.log("INFO", &format!("[FIX:run-control] Pipeline resumed at {stage}"), &mut writes);
                        }
                        Some(PipelineCommand::ConfirmSchema(columns)) => {
                            self.schema.send_replace(Some(columns));
                        }
                        None => {
                            commands_open = false;
                            if paused {
                                drop(work);
                                self.drain_issues(&mut writes);
                                self.publish_status("cancelled", &mut writes, None);
                                writes.flush().await;
                                return Ok(PipelineState::Cancelled);
                            }
                        }
                        _ => {}
                    }
                }
                Some(update) = self.stages.recv() => {
                    stage = update.status;
                    // Terminal status is published only after the work returns
                    // and all diagnostics have been drained. Schema and other
                    // active stages still acknowledge their persisted status.
                    if !paused && !matches!(stage.as_str(), "completed" | "failed" | "cancelled") {
                        self.publish_status(&stage, &mut writes, Some(update.acknowledged));
                    } else {
                        let _ = update.acknowledged.send(());
                    }
                }
                Some(issue) = self.issues.recv() => self.publish_issue(issue, &mut writes),
                () = writes.progress(), if writes.pending() => {}
                result = &mut work, if !paused => {
                    drop(work);
                    self.drain_issues(&mut writes);
                    let failure = match &result {
                        Err(err) => Some(err.to_string()),
                        Ok(PipelineState::Failed(message)) => Some(message.clone()),
                        _ => None,
                    };
                    if let Some(message) = failure {
                        writes.queue.push_back(ControlWrite::Failure(message.clone()));
                        if let Some(events) = &self.events {
                            events.emit_error(&message);
                            events.emit_status_changed("failed");
                        }
                        self.log("ERROR", &format!("[FIX:run-control] Pipeline failed: {message}"), &mut writes);
                    } else if let Ok(state) = &result {
                        self.publish_status(state.as_status_str(), &mut writes, None);
                    }
                    // Work has been dropped, so it can no longer retain a DB
                    // connection. Ordered writes make the terminal status last.
                    writes.flush().await;
                    return result;
                }
            }
        }
    }

    fn publish_status(
        &self,
        status: &str,
        writes: &mut ControlWriter,
        acknowledged: Option<oneshot::Sender<()>>,
    ) {
        writes.queue.push_back(ControlWrite::Status {
            status: status.to_string(),
            acknowledged,
        });
        if let Some(events) = &self.events {
            events.emit_status_changed(status);
        }
    }

    fn drain_issues(&mut self, writes: &mut ControlWriter) {
        while let Ok(issue) = self.issues.try_recv() {
            self.publish_issue(issue, writes);
        }
    }

    fn publish_issue(&self, issue: LlmIssue, writes: &mut ControlWriter) {
        let details = match serde_json::to_string(&issue) {
            Ok(details) => details,
            Err(e) => {
                error!(error = %e, "Failed to serialize LLM issue");
                return;
            }
        };
        let message = format!("{} / {}: {}", issue.provider, issue.model, issue.message);
        writes.queue.push_back(ControlWrite::Log {
            level: "WARN",
            role: "llm_issue",
            message: message.clone(),
            details: Some(details),
        });
        if let Some(events) = &self.events {
            events.emit_llm_issue(&issue);
            events.emit_log("WARN", "llm_issue", &message);
        }
    }

    fn log(&self, level: &'static str, message: &str, writes: &mut ControlWriter) {
        writes.queue.push_back(ControlWrite::Log {
            level,
            role: "run_control",
            message: message.to_string(),
            details: None,
        });
        if let Some(events) = &self.events {
            events.emit_log(level, "run_control", message);
        }
        if level == "ERROR" {
            error!(run_id = %self.run_id, "{message}");
        } else {
            info!(run_id = %self.run_id, "{message}");
        }
    }
}

/// A result receiver owns its workers. Dropping it aborts network requests and
/// blocked job submission, including early stops and errors as well as Cancel.
pub struct WorkerResults<T> {
    receiver: mpsc::UnboundedReceiver<T>,
    tasks: JoinSet<()>,
}

impl<T> WorkerResults<T> {
    pub(crate) fn new(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self {
            receiver,
            tasks: JoinSet::new(),
        }
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }

    pub(crate) fn spawn<F>(&mut self, mut paused: watch::Receiver<bool>, work: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.tasks.spawn(async move {
            tokio::pin!(work);
            loop {
                let is_paused = *paused.borrow_and_update();
                tokio::select! {
                    biased;
                    changed = paused.changed() => {
                        if changed.is_err() {
                            return;
                        }
                    }
                    () = &mut work, if !is_paused => return,
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use crate::orchestrator::image_pipeline::ImagePipeline;
    use crate::orchestrator::link_pipeline::LinkPipeline;
    use crate::orchestrator::pipeline::{Pipeline, PipelineConfig};
    use crate::orchestrator::research_pipeline::ResearchPipeline;
    use crate::providers::llm::manager::LlmBackend;
    use crate::storage::db::Database;
    use sqlx::sqlite::SqlitePoolOptions;
    use tokio::sync::Notify;
    use tokio::time::{sleep, timeout};
    use wiremock::{matchers::path, Mock, MockServer, ResponseTemplate};

    async fn test_repo() -> Arc<Repository> {
        test_repo_with_pool().await.0
    }

    async fn test_repo_with_pool() -> (Arc<Repository>, sqlx::SqlitePool) {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let db = Database::with_pool(pool.clone()).await;
        db.migrate().await.unwrap();
        (Arc::new(Repository::new(pool.clone())), pool)
    }

    fn output_limit_issue() -> LlmIssue {
        LlmIssue {
            code: "output_limit".into(),
            provider: "ollama_cloud".into(),
            model: "test-model".into(),
            stage: Some("extractor".into()),
            message: "Model reached its output token limit".into(),
            max_tokens: 4096,
            prompt_tokens: Some(800),
            completion_tokens: Some(4096),
            reasoning_tokens: None,
            retry_after_ms: None,
            attempt: 1,
            max_attempts: 1,
            will_retry: false,
        }
    }

    #[tokio::test]
    async fn issue_persistence_does_not_block_work_holding_the_only_database_connection() {
        let (repo, pool) = test_repo_with_pool().await;
        repo.create_run("db-progress", "query", "{}").await.unwrap();
        let (_tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("db-progress".into(), repo.clone(), None, rx);
        let task = supervisor.run(async move {
            let mut connection = pool.acquire().await.unwrap();
            control.issue_sender().send(output_limit_issue()).unwrap();
            // The supervisor sees the issue before this future can release the
            // connection. Awaiting diagnostic persistence inline deadlocks.
            tokio::task::yield_now().await;
            sqlx::query("SELECT 1")
                .execute(&mut *connection)
                .await
                .unwrap();
            drop(connection);
            Ok::<_, String>(PipelineState::Completed)
        });
        assert_eq!(
            timeout(Duration::from_secs(1), task)
                .await
                .unwrap()
                .unwrap(),
            PipelineState::Completed
        );
        assert_eq!(
            repo.get_run_llm_issue_details("db-progress")
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn pause_allows_resume_or_cancel_while_work_holds_a_database_connection() {
        for cancel in [false, true] {
            let (repo, pool) = test_repo_with_pool().await;
            repo.create_run("db-control", "query", "{}").await.unwrap();
            let (tx, rx) = mpsc::channel(16);
            let (control, supervisor) =
                RunControl::new("db-control".into(), repo.clone(), None, rx);
            let mut paused = control.pause_signal();
            let (started_tx, started_rx) = oneshot::channel();
            let (release_tx, release_rx) = oneshot::channel();
            let task = tokio::spawn(supervisor.run(async move {
                control.set_status("running").await;
                let connection = pool.acquire().await.unwrap();
                control.issue_sender().send(output_limit_issue()).unwrap();
                started_tx.send(()).unwrap();
                let _ = release_rx.await;
                drop(connection);
                control.set_status("completed").await;
                Ok::<_, String>(PipelineState::Completed)
            }));
            started_rx.await.unwrap();
            tx.send(PipelineCommand::Pause).await.unwrap();
            timeout(Duration::from_secs(1), paused.wait_for(|value| *value))
                .await
                .unwrap()
                .unwrap();
            assert!(!task.is_finished());
            let expected = if cancel {
                tx.send(PipelineCommand::Cancel).await.unwrap();
                PipelineState::Cancelled
            } else {
                release_tx.send(()).unwrap();
                tx.send(PipelineCommand::Resume).await.unwrap();
                PipelineState::Completed
            };
            assert_eq!(
                timeout(Duration::from_secs(1), task)
                    .await
                    .unwrap()
                    .unwrap()
                    .unwrap(),
                expected
            );
            assert_eq!(
                repo.get_run("db-control").await.unwrap().unwrap().status,
                expected.as_status_str()
            );
            assert_eq!(
                repo.get_run_llm_issue_details("db-control")
                    .await
                    .unwrap()
                    .len(),
                1
            );
        }
    }

    #[tokio::test]
    async fn swallowed_llm_limit_is_persisted_even_when_run_completes_in_the_same_poll() {
        let repo = test_repo().await;
        repo.create_run("partial", "query", "{}").await.unwrap();
        let (_tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("partial".into(), repo.clone(), None, rx);
        let result = supervisor
            .run(async move {
                control.issue_sender().send(output_limit_issue()).unwrap();
                Ok::<_, String>(PipelineState::Completed)
            })
            .await
            .unwrap();
        assert_eq!(result, PipelineState::Completed);
        let details = repo.get_run_llm_issue_details("partial").await.unwrap();
        assert_eq!(details.len(), 1);
        let issue: LlmIssue = serde_json::from_str(&details[0]).unwrap();
        assert_eq!(issue.code, "output_limit");
        assert_eq!(issue.stage.as_deref(), Some("extractor"));
        assert_eq!(issue.completion_tokens, Some(4096));
        assert_eq!(issue.max_tokens, 4096);
    }

    #[tokio::test]
    async fn fatal_llm_limit_retains_diagnostics_and_failed_status() {
        let repo = test_repo().await;
        repo.create_run("limited", "query", "{}").await.unwrap();
        let (_tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("limited".into(), repo.clone(), None, rx);
        let result = supervisor
            .run(async move {
                control.issue_sender().send(output_limit_issue()).unwrap();
                Err::<PipelineState, _>("Token limit reached".to_string())
            })
            .await;
        assert!(result.is_err());
        repo.create_run_log(
            "limited",
            "ERROR",
            Some("other"),
            "unrelated error",
            Some("not JSON"),
        )
        .await
        .unwrap();
        assert_eq!(
            repo.get_run_llm_issue_details("limited")
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            repo.get_run("limited").await.unwrap().unwrap().status,
            "failed"
        );
    }

    async fn wait_status(repo: &Repository, id: &str, expected: &str) {
        timeout(Duration::from_secs(2), async {
            loop {
                if repo
                    .get_run(id)
                    .await
                    .unwrap()
                    .is_some_and(|run| run.status == expected)
                {
                    return;
                }
                sleep(Duration::from_millis(5)).await;
            }
        })
        .await
        .unwrap_or_else(|_| panic!("Run {id} did not become {expected}"));
    }

    struct NotifyOnDrop(Arc<Notify>);
    impl Drop for NotifyOnDrop {
        fn drop(&mut self) {
            self.0.notify_one();
        }
    }

    #[tokio::test]
    async fn cancel_drops_pending_request_and_persists_terminal_status() {
        let repo = test_repo().await;
        repo.create_run("cancel", "query", "{}").await.unwrap();
        let (tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("cancel".into(), repo.clone(), None, rx);
        let dropped = Arc::new(Notify::new());
        let guard = NotifyOnDrop(dropped.clone());
        let task = tokio::spawn(supervisor.run(async move {
            let _guard = guard;
            control.set_status("running").await;
            std::future::pending::<Result<PipelineState, String>>().await
        }));
        wait_status(&repo, "cancel", "running").await;
        tx.send(PipelineCommand::Cancel).await.unwrap();
        let result = timeout(Duration::from_secs(1), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        assert_eq!(result, PipelineState::Cancelled);
        timeout(Duration::from_secs(1), dropped.notified())
            .await
            .unwrap();
        let run = repo.get_run("cancel").await.unwrap().unwrap();
        assert_eq!(run.status, "cancelled");
        assert!(run.completed_at.is_some());
    }

    #[tokio::test]
    async fn pause_keeps_schema_stage_and_buffers_confirmation_until_resume() {
        let repo = test_repo().await;
        repo.create_run("schema", "query", "{}").await.unwrap();
        let (tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("schema".into(), repo.clone(), None, rx);
        let stages = Arc::new(AtomicUsize::new(0));
        let counted = stages.clone();
        let (schema_seen_tx, schema_seen_rx) = oneshot::channel();
        let (finish_tx, finish_rx) = oneshot::channel();
        let expected = vec![SchemaColumn {
            name: "channel".into(),
            col_type: "text".into(),
            description: "YouTube channel".into(),
            required: true,
        }];
        let task = tokio::spawn(supervisor.run(async move {
            counted.fetch_add(1, Ordering::SeqCst);
            control.set_status("schema_review").await;
            let schema = control.confirmed_schema().await.unwrap();
            schema_seen_tx.send(schema).unwrap();
            finish_rx.await.unwrap();
            Ok::<_, String>(PipelineState::Completed)
        }));
        wait_status(&repo, "schema", "schema_review").await;
        tx.send(PipelineCommand::Pause).await.unwrap();
        wait_status(&repo, "schema", "paused").await;
        tx.send(PipelineCommand::ConfirmSchema(expected.clone()))
            .await
            .unwrap();
        sleep(Duration::from_millis(30)).await;
        assert_eq!(
            repo.get_run("schema").await.unwrap().unwrap().status,
            "paused"
        );
        assert!(!task.is_finished());
        tx.send(PipelineCommand::Resume).await.unwrap();
        wait_status(&repo, "schema", "schema_review").await;
        assert_eq!(
            timeout(Duration::from_secs(1), schema_seen_rx)
                .await
                .unwrap()
                .unwrap(),
            expected
        );
        finish_tx.send(()).unwrap();
        assert_eq!(task.await.unwrap().unwrap(), PipelineState::Completed);
        assert_eq!(stages.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn schema_idle_timeout_restarts_after_a_long_pause() {
        let repo = test_repo().await;
        repo.create_run("idle", "query", "{}").await.unwrap();
        let (tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("idle".into(), repo.clone(), None, rx);
        let (started_tx, started_rx) = oneshot::channel();
        let task = tokio::spawn(supervisor.run(async move {
            control.set_status("schema_review").await;
            started_tx.send(()).unwrap();
            tokio::select! {
                _ = control.schema_idle_timeout(Duration::from_millis(100)) => {
                    Err("Schema timed out".to_string())
                }
                _ = control.confirmed_schema() => Ok(PipelineState::Completed),
            }
        }));
        started_rx.await.unwrap();
        tx.send(PipelineCommand::Pause).await.unwrap();
        wait_status(&repo, "idle", "paused").await;
        sleep(Duration::from_millis(150)).await;
        tx.send(PipelineCommand::Resume).await.unwrap();
        wait_status(&repo, "idle", "schema_review").await;
        sleep(Duration::from_millis(20)).await;
        assert!(
            !task.is_finished(),
            "A long pause consumed the schema review timeout"
        );
        tx.send(PipelineCommand::ConfirmSchema(vec![]))
            .await
            .unwrap();
        assert_eq!(task.await.unwrap().unwrap(), PipelineState::Completed);
    }

    #[tokio::test]
    async fn errors_are_persisted_instead_of_leaving_run_running() {
        let repo = test_repo().await;
        repo.create_run("failure", "query", "{}").await.unwrap();
        let (_tx, rx) = mpsc::channel(16);
        let (control, supervisor) = RunControl::new("failure".into(), repo.clone(), None, rx);
        let message = "QueryExpander: empty model response";
        let result = supervisor
            .run(async move {
                control.set_status("running").await;
                Err::<PipelineState, _>(message.to_string())
            })
            .await;
        assert_eq!(result.unwrap_err(), message);
        let run = repo.get_run("failure").await.unwrap().unwrap();
        assert_eq!(run.status, "failed");
        assert_eq!(run.error.as_deref(), Some(message));
        assert!(run.completed_at.is_some());
    }

    fn spawn_pipeline(
        mode: &str,
        config: PipelineConfig,
        repo: Arc<Repository>,
    ) -> (
        mpsc::Sender<PipelineCommand>,
        tokio::task::JoinHandle<Result<PipelineState, String>>,
    ) {
        let id = mode.to_string();
        let query = "Find YouTube channels about robots".to_string();
        match mode {
            "table" => {
                let (pipeline, tx) = Pipeline::new(id, query, config, repo, None);
                (
                    tx,
                    tokio::spawn(async move { pipeline.run().await.map_err(|e| e.to_string()) }),
                )
            }
            "images" => {
                let (pipeline, tx) = ImagePipeline::new(id, query, config, repo, None);
                (tx, tokio::spawn(pipeline.run()))
            }
            "links" => {
                let (pipeline, tx) = LinkPipeline::new(id, query, config, repo, None);
                (tx, tokio::spawn(pipeline.run()))
            }
            "research" => {
                let (pipeline, tx) = ResearchPipeline::new(id, query, config, repo, None);
                (tx, tokio::spawn(pipeline.run()))
            }
            _ => panic!("Unknown test mode"),
        }
    }

    #[tokio::test]
    async fn all_modes_pause_resume_and_cancel_during_pending_llm_http_request() {
        for mode in ["table", "images", "links", "research"] {
            let server = MockServer::start().await;
            Mock::given(path("/api/chat"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_delay(Duration::from_secs(60))
                        .set_body_json(serde_json::json!({"message": {"content": "{}"}})),
                )
                .mount(&server)
                .await;
            let repo = test_repo().await;
            let mut config = PipelineConfig::from_settings(&Default::default());
            config.llm.backend = LlmBackend::Ollama;
            config.llm.ollama_url = server.uri();
            config.search.brave_api_key = "unused-local-test-key".into();
            let (tx, task) = spawn_pipeline(mode, config, repo.clone());

            timeout(Duration::from_secs(2), async {
                while server.received_requests().await.unwrap().is_empty() {
                    sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap_or_else(|_| panic!("{mode}: model request did not start"));
            tx.send(PipelineCommand::Pause).await.unwrap();
            wait_status(&repo, mode, "paused").await;
            assert!(!task.is_finished(), "{mode}: pause must retain work");
            tx.send(PipelineCommand::Resume).await.unwrap();
            wait_status(&repo, mode, "running").await;
            assert_eq!(
                server.received_requests().await.unwrap().len(),
                1,
                "{mode}: resume restarted request"
            );
            tx.send(PipelineCommand::Pause).await.unwrap();
            wait_status(&repo, mode, "paused").await;
            tx.send(PipelineCommand::Cancel).await.unwrap();
            let result = timeout(Duration::from_secs(1), task)
                .await
                .unwrap_or_else(|_| panic!("{mode}: cancel waited for HTTP response"))
                .unwrap()
                .unwrap();
            assert_eq!(result, PipelineState::Cancelled);
            assert_eq!(
                repo.get_run(mode).await.unwrap().unwrap().status,
                "cancelled"
            );
        }
    }

    #[tokio::test]
    async fn all_modes_persist_failures_from_provider_initialization() {
        for mode in ["table", "images", "links", "research"] {
            let repo = test_repo().await;
            let config = PipelineConfig::from_settings(&Default::default());
            let (_tx, task) = spawn_pipeline(mode, config, repo.clone());
            let _ = timeout(Duration::from_secs(2), task)
                .await
                .unwrap_or_else(|_| panic!("{mode}: failed provider setup did not finish"))
                .unwrap();
            let run = repo.get_run(mode).await.unwrap().unwrap();
            assert_eq!(
                run.status, "failed",
                "{mode}: failed setup left an active status"
            );
            assert!(run.error.is_some());
        }
    }

    #[tokio::test]
    async fn dropping_worker_results_aborts_pending_work() {
        let (_paused_tx, paused) = watch::channel(false);
        let (_tx, rx) = mpsc::unbounded_channel::<()>();
        let mut results = WorkerResults::new(rx);
        let dropped = Arc::new(Notify::new());
        let guard = NotifyOnDrop(dropped.clone());
        let (started_tx, started_rx) = oneshot::channel();
        results.spawn(paused, async move {
            let _guard = guard;
            started_tx.send(()).unwrap();
            std::future::pending::<()>().await;
        });
        started_rx.await.unwrap();
        drop(results);
        timeout(Duration::from_secs(1), dropped.notified())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn paused_worker_retains_work_and_processes_result_once_after_resume() {
        let (paused_tx, paused) = watch::channel(false);
        let (tx, rx) = mpsc::unbounded_channel();
        let mut results = WorkerResults::new(rx);
        let started = Arc::new(AtomicUsize::new(0));
        let counted = started.clone();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();
        results.spawn(paused, async move {
            counted.fetch_add(1, Ordering::SeqCst);
            started_tx.send(()).unwrap();
            release_rx.await.unwrap();
            tx.send("done").unwrap();
        });
        started_rx.await.unwrap();
        paused_tx.send_replace(true);
        release_tx.send(()).unwrap();
        assert!(timeout(Duration::from_millis(50), results.recv())
            .await
            .is_err());
        paused_tx.send_replace(false);
        assert_eq!(
            timeout(Duration::from_secs(1), results.recv())
                .await
                .unwrap(),
            Some("done")
        );
        assert_eq!(started.load(Ordering::SeqCst), 1);
    }
}
