use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use serde::Serialize;
use tauri::{AppHandle, State};
use tracing::{info, error};

use serde::Deserialize;
use crate::AppState;
use crate::storage::models::SchemaColumn;
use crate::storage::repository::Repository;
use crate::orchestrator::pipeline::{Pipeline, PipelineConfig, PipelineCommand};
use crate::orchestrator::image_pipeline::ImagePipeline;
use crate::orchestrator::link_pipeline::LinkPipeline;
use crate::orchestrator::research_pipeline::ResearchPipeline;
use crate::orchestrator::events::EventPublisher;
use crate::orchestrator::events::LlmIssueEvent;
use crate::utils::id::new_id;

/// Holds senders for controlling active pipelines.
#[derive(Clone)]
pub struct RunController {
    pub active: Arc<Mutex<HashMap<String, mpsc::Sender<PipelineCommand>>>>,
}

impl RunController {
    pub fn new() -> Self {
        Self {
            active: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn send(&self, run_id: &str, command: PipelineCommand) -> Result<(), String> {
        let tx = self.active.lock().await.get(run_id).cloned()
            .ok_or_else(|| format!("Run {run_id} is not active"))?;
        let action = match &command {
            PipelineCommand::Cancel => "cancel",
            PipelineCommand::Pause => "pause",
            PipelineCommand::Resume => "resume",
            PipelineCommand::ConfirmSchema(_) => "confirm_schema",
        };
        // A full command queue must not lock control/cleanup for every active run.
        tx.send(command).await.map_err(|_| format!("Run {run_id} is no longer active"))?;
        info!(run_id, action, "[FIX:run-control] Command queued");
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct StartRunResponse {
    pub run_id: String,
}

#[derive(Debug, Deserialize)]
pub struct StopConditions {
    pub target_row_count: Option<usize>,
    pub max_budget_usd: Option<f64>,
    pub max_duration_seconds: Option<u64>,
}

#[tauri::command]
pub async fn start_run(
    app: AppHandle,
    state: State<'_, AppState>,
    controller: State<'_, RunController>,
    query: String,
    run_type: Option<String>,
    stop_conditions: Option<StopConditions>,
) -> Result<StartRunResponse, String> {
    let run_id = new_id();
    let run_type = run_type.unwrap_or_else(|| "table".to_string());

    // Load settings from DB
    let settings_list = state.db.get_all_settings().await.map_err(|e| e.to_string())?;
    let settings: HashMap<String, String> = settings_list.into_iter().collect();

    let mut config = PipelineConfig::from_settings(&settings);

    // Override stop conditions with per-query values
    if let Some(sc) = stop_conditions {
        if let Some(v) = sc.target_row_count {
            config.stop.target_row_count = v;
        }
        if let Some(v) = sc.max_budget_usd {
            config.stop.max_budget_usd = v;
            config.max_budget_usd = v;
        }
        if let Some(v) = sc.max_duration_seconds {
            config.stop.max_duration_secs = v;
        }
    }
    let repo = Arc::new(Repository::new(state.db.pool().clone()));
    let events = Some(EventPublisher::new(app.clone(), run_id.clone()));

    let rid = run_id.clone();
    let controller_handle = controller.active.clone();

    if run_type == "images" {
        // Image search pipeline
        let (pipeline, cmd_tx) = ImagePipeline::new(
            run_id.clone(),
            query.clone(),
            config,
            repo,
            events,
        );

        {
            let mut active = controller.active.lock().await;
            active.insert(run_id.clone(), cmd_tx);
        }

        tokio::spawn(async move {
            let result = pipeline.run().await;
            match &result {
                Ok(state) => info!(run_id = %rid, state = ?state, "Image pipeline finished"),
                Err(e) => error!(run_id = %rid, error = %e, "Image pipeline failed"),
            }
            let mut active = controller_handle.lock().await;
            active.remove(&rid);
        });
    } else if run_type == "links" {
        // Link relevance-filter pipeline
        let (pipeline, cmd_tx) = LinkPipeline::new(
            run_id.clone(),
            query.clone(),
            config,
            repo,
            events,
        );

        {
            let mut active = controller.active.lock().await;
            active.insert(run_id.clone(), cmd_tx);
        }

        tokio::spawn(async move {
            let result = pipeline.run().await;
            match &result {
                Ok(state) => info!(run_id = %rid, state = ?state, "Link pipeline finished"),
                Err(e) => error!(run_id = %rid, error = %e, "Link pipeline failed"),
            }
            let mut active = controller_handle.lock().await;
            active.remove(&rid);
        });
    } else if run_type == "research" {
        // Agentic web research pipeline
        let (pipeline, cmd_tx) = ResearchPipeline::new(
            run_id.clone(),
            query.clone(),
            config,
            repo,
            events,
        );

        {
            let mut active = controller.active.lock().await;
            active.insert(run_id.clone(), cmd_tx);
        }

        tokio::spawn(async move {
            let result = pipeline.run().await;
            match &result {
                Ok(state) => info!(run_id = %rid, state = ?state, "Research pipeline finished"),
                Err(e) => error!(run_id = %rid, error = %e, "Research pipeline failed"),
            }
            let mut active = controller_handle.lock().await;
            active.remove(&rid);
        });
    } else {
        // Default table pipeline
        let (pipeline, cmd_tx) = Pipeline::new(
            run_id.clone(),
            query.clone(),
            config,
            repo,
            events,
        );

        {
            let mut active = controller.active.lock().await;
            active.insert(run_id.clone(), cmd_tx);
        }

        tokio::spawn(async move {
            let result = pipeline.run().await;
            match &result {
                Ok(state) => info!(run_id = %rid, state = ?state, "Pipeline finished"),
                Err(e) => error!(run_id = %rid, error = %e, "Pipeline failed"),
            }
            let mut active = controller_handle.lock().await;
            active.remove(&rid);
        });
    }

    info!(run_id = %run_id, query = %query, run_type = %run_type, "Run started");

    Ok(StartRunResponse { run_id })
}

#[tauri::command]
pub async fn cancel_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    controller.send(&run_id, PipelineCommand::Cancel).await
}

#[tauri::command]
pub async fn pause_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    controller.send(&run_id, PipelineCommand::Pause).await
}

#[tauri::command]
pub async fn resume_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    controller.send(&run_id, PipelineCommand::Resume).await
}

#[tauri::command]
pub async fn confirm_schema(
    controller: State<'_, RunController>,
    run_id: String,
    columns: Vec<SchemaColumn>,
) -> Result<(), String> {
    controller.send(&run_id, PipelineCommand::ConfirmSchema(columns)).await
}

#[derive(Debug, Serialize)]
pub struct RunInfo {
    pub id: String,
    pub query: String,
    pub status: String,
    pub run_type: String,
    pub stats: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
}

#[tauri::command]
pub async fn get_run(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Option<RunInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let row = repo.get_run(&run_id).await.map_err(|e| e.to_string())?;
    Ok(row.map(|r| RunInfo {
        id: r.id,
        query: r.query,
        status: r.status,
        run_type: r.run_type,
        stats: r.stats,
        error: r.error,
        created_at: r.created_at,
    }))
}

#[tauri::command]
pub async fn list_runs(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<RunInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.list_runs(limit.unwrap_or(50), offset.unwrap_or(0))
        .await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| RunInfo {
        id: r.id,
        query: r.query,
        status: r.status,
        run_type: r.run_type,
        stats: r.stats,
        error: r.error,
        created_at: r.created_at,
    }).collect())
}

#[tauri::command]
pub async fn delete_run(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<(), String> {
    let repo = Repository::new(state.db.pool().clone());
    repo.delete_run(&run_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_logs(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<RunLogEntry>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let logs = repo.get_run_logs(&run_id, 1000).await.map_err(|e| e.to_string())?;
    Ok(logs.into_iter().map(|l| RunLogEntry {
        id: l.id.to_string(),
        level: l.level,
        role: l.role,
        message: l.message,
        created_at: l.created_at,
    }).collect())
}

#[tauri::command]
pub async fn get_run_issues(state: State<'_, AppState>, run_id: String) -> Result<Vec<LlmIssueEvent>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let details = repo.get_run_llm_issue_details(&run_id).await.map_err(|e| e.to_string())?;
    Ok(details.into_iter().filter_map(|detail| {
        serde_json::from_str(&detail).ok().map(|issue| LlmIssueEvent { run_id: run_id.clone(), issue })
    }).collect())
}

#[derive(Debug, Serialize)]
pub struct RunLogEntry {
    pub id: String,
    pub level: String,
    pub role: Option<String>,
    pub message: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct RunSchemaInfo {
    pub columns: serde_json::Value,
    pub confirmed: bool,
}

#[tauri::command]
pub async fn get_run_schema(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Option<RunSchemaInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let schema = repo.get_run_schema(&run_id).await.map_err(|e| e.to_string())?;
    Ok(schema.map(|s| {
        let columns = serde_json::from_str(&s.columns).unwrap_or(serde_json::Value::Array(vec![]));
        RunSchemaInfo {
            columns,
            confirmed: s.confirmed != 0,
        }
    }))
}

#[derive(Debug, Serialize)]
pub struct EntityRowInfo {
    pub id: String,
    pub data: serde_json::Value,
    pub confidence: f64,
    pub status: String,
}

#[tauri::command]
pub async fn get_run_rows(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<EntityRowInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.get_entity_rows_by_run(&run_id).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| {
        let data = serde_json::from_str(&r.data).unwrap_or(serde_json::Value::Object(Default::default()));
        EntityRowInfo {
            id: r.id,
            data,
            confidence: r.confidence,
            status: r.status,
        }
    }).collect())
}

#[derive(Debug, Serialize)]
pub struct ImageResultInfo {
    pub id: String,
    pub image_url: String,
    pub thumbnail_url: String,
    pub title: String,
    pub source_url: String,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub relevance_score: Option<f64>,
}

#[tauri::command]
pub async fn get_image_results(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<ImageResultInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.get_image_results(&run_id).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| ImageResultInfo {
        id: r.id,
        image_url: r.image_url,
        thumbnail_url: r.thumbnail_url,
        title: r.title,
        source_url: r.source_url,
        width: r.width,
        height: r.height,
        relevance_score: r.relevance_score,
    }).collect())
}

#[derive(Debug, Serialize)]
pub struct LinkResultInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub relevance_score: Option<f64>,
}

#[tauri::command]
pub async fn get_link_results(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<LinkResultInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.get_link_results(&run_id).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| LinkResultInfo {
        id: r.id,
        url: r.url,
        title: r.title,
        description: r.description,
        relevance_score: r.relevance_score,
    }).collect())
}

#[derive(Debug, Serialize)]
pub struct ResearchStepInfo {
    pub id: String,
    pub step_index: i64,
    pub step_type: String,
    pub content: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResearchResultInfo {
    pub answer_markdown: Option<String>,
    pub steps: Vec<ResearchStepInfo>,
}

#[tauri::command]
pub async fn get_research_result(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<ResearchResultInfo, String> {
    let repo = Repository::new(state.db.pool().clone());
    let steps = repo.get_research_steps(&run_id).await.map_err(|e| e.to_string())?;
    let answer = repo.get_research_result(&run_id).await.map_err(|e| e.to_string())?;
    Ok(ResearchResultInfo {
        answer_markdown: answer.map(|a| a.answer_markdown),
        steps: steps.into_iter().map(|s| ResearchStepInfo {
            id: s.id,
            step_index: s.step_index,
            step_type: s.step_type,
            content: s.content,
            url: s.url,
        }).collect(),
    })
}

/// Proxy-fetch an image URL through the backend to avoid hotlink protection.
/// Returns a data URL (data:image/...;base64,...).
#[tauri::command]
pub async fn proxy_image(url: String) -> Result<String, String> {
    use reqwest::Client;
    use base64::Engine;

    let builder = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36");
    let client = crate::providers::http::apply_proxy(builder)
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client.get(&url).send().await
        .map_err(|e| format!("Fetch error: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }

    let content_type = resp.headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    // Only allow image content types
    if !content_type.starts_with("image/") {
        return Err(format!("Not an image: {content_type}"));
    }

    let bytes = resp.bytes().await
        .map_err(|e| format!("Read error: {e}"))?;

    // Limit to 10MB
    if bytes.len() > 10 * 1024 * 1024 {
        return Err("Image too large (>10MB)".to_string());
    }

    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{content_type};base64,{b64}"))
}

#[cfg(test)]
mod control_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn queued_command_does_not_hold_active_runs_mutex() {
        let controller = RunController::new();
        let (tx, mut rx) = mpsc::channel(1);
        tx.send(PipelineCommand::Pause).await.unwrap();
        controller.active.lock().await.insert("run".into(), tx);
        let send = controller.send("run", PipelineCommand::Cancel);
        tokio::pin!(send);
        // Poll the send until it blocks on the full queue.
        assert!(tokio::time::timeout(Duration::from_millis(20), &mut send).await.is_err());
        let guard = tokio::time::timeout(Duration::from_millis(100), controller.active.lock())
            .await.expect("a blocked send must release the registry mutex");
        drop(guard);
        assert_eq!(rx.recv().await, Some(PipelineCommand::Pause));
        send.await.unwrap();
        assert_eq!(rx.recv().await, Some(PipelineCommand::Cancel));
    }

    #[tokio::test]
    async fn inactive_or_finished_runs_return_a_control_error() {
        let controller = RunController::new();
        assert!(controller.send("missing", PipelineCommand::Pause).await.is_err());
        let (tx, rx) = mpsc::channel(1);
        controller.active.lock().await.insert("finished".into(), tx);
        drop(rx);
        assert!(controller.send("finished", PipelineCommand::Cancel).await.is_err());
    }
}
