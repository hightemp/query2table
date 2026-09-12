use std::fmt::Display;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinSet;
use tracing::{error, info};

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
}

pub(crate) struct RunSupervisor {
    run_id: String,
    repo: Arc<Repository>,
    events: Option<EventPublisher>,
    commands: mpsc::Receiver<PipelineCommand>,
    stages: mpsc::Receiver<StageUpdate>,
    paused: watch::Sender<bool>,
    schema: watch::Sender<Option<Vec<SchemaColumn>>>,
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
        (
            Self {
                stage_tx,
                paused,
                schema,
            },
            RunSupervisor {
                run_id,
                repo,
                events,
                commands,
                stages,
                paused: paused_tx,
                schema: schema_tx,
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

        loop {
            tokio::select! {
                biased;
                command = self.commands.recv(), if commands_open => {
                    match command {
                        Some(_) if matches!(stage.as_str(), "completed" | "failed" | "cancelled") => {}
                        Some(PipelineCommand::Cancel) => {
                            drop(work);
                            self.publish_status("cancelled").await;
                            self.log("INFO", "[FIX:run-control] Cancelled active pipeline and worker requests").await;
                            return Ok(PipelineState::Cancelled);
                        }
                        Some(PipelineCommand::Pause) if !paused => {
                            paused = true;
                            self.paused.send_replace(true);
                            self.publish_status("paused").await;
                            self.log("INFO", &format!("[FIX:run-control] Pipeline paused at {stage}")).await;
                        }
                        Some(PipelineCommand::Resume) if paused => {
                            self.publish_status(&stage).await;
                            paused = false;
                            self.paused.send_replace(false);
                            self.log("INFO", &format!("[FIX:run-control] Pipeline resumed at {stage}")).await;
                        }
                        Some(PipelineCommand::ConfirmSchema(columns)) => {
                            self.schema.send_replace(Some(columns));
                        }
                        None => {
                            commands_open = false;
                            if paused {
                                drop(work);
                                self.publish_status("cancelled").await;
                                return Ok(PipelineState::Cancelled);
                            }
                        }
                        _ => {}
                    }
                }
                Some(update) = self.stages.recv() => {
                    stage = update.status;
                    if !paused {
                        self.publish_status(&stage).await;
                    }
                    let _ = update.acknowledged.send(());
                }
                result = &mut work, if !paused => {
                    drop(work);
                    let failure = match &result {
                        Err(err) => Some(err.to_string()),
                        Ok(PipelineState::Failed(message)) => Some(message.clone()),
                        _ => None,
                    };
                    if let Some(message) = failure {
                        if let Err(e) = self.repo.update_run_error(&self.run_id, &message).await {
                            error!(run_id = %self.run_id, error = %e, "Failed to persist pipeline error");
                        }
                        if let Some(events) = &self.events {
                            events.emit_error(&message);
                            events.emit_status_changed("failed");
                        }
                        self.log("ERROR", &format!("[FIX:run-control] Pipeline failed: {message}")).await;
                    } else if let Ok(state) = &result {
                        if state.as_status_str() != stage {
                            self.publish_status(state.as_status_str()).await;
                        }
                    }
                    return result;
                }
            }
        }
    }

    async fn publish_status(&self, status: &str) {
        if let Err(e) = self.repo.update_run_status(&self.run_id, status).await {
            error!(run_id = %self.run_id, error = %e, "Failed to persist pipeline status");
        }
        if let Some(events) = &self.events {
            events.emit_status_changed(status);
        }
    }

    async fn log(&self, level: &str, message: &str) {
        if let Err(e) = self
            .repo
            .create_run_log(&self.run_id, level, Some("run_control"), message, None)
            .await
        {
            error!(run_id = %self.run_id, error = %e, "Failed to persist run control log");
        }
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
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        let db = Database::with_pool(pool.clone()).await;
        db.migrate().await.unwrap();
        Arc::new(Repository::new(pool))
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
                .unwrap()
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
