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
use crate::orchestrator::events::EventPublisher;
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
    stop_conditions: Option<StopConditions>,
) -> Result<StartRunResponse, String> {
    let run_id = new_id();

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

    let (pipeline, cmd_tx) = Pipeline::new(
        run_id.clone(),
        query.clone(),
        config,
        repo,
        events,
    );

    // Store the command sender
    {
        let mut active = controller.active.lock().await;
        active.insert(run_id.clone(), cmd_tx);
    }

    let rid = run_id.clone();
    let controller_handle = controller.active.clone();

    // Spawn pipeline on background task
    tokio::spawn(async move {
        let result = pipeline.run().await;
        match &result {
            Ok(state) => info!(run_id = %rid, state = ?state, "Pipeline finished"),
            Err(e) => error!(run_id = %rid, error = %e, "Pipeline failed"),
        }
        // Remove from active runs
        let mut active = controller_handle.lock().await;
        active.remove(&rid);
    });

    info!(run_id = %run_id, query = %query, "Run started");

    Ok(StartRunResponse { run_id })
}

#[tauri::command]
pub async fn cancel_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    let active = controller.active.lock().await;
    if let Some(tx) = active.get(&run_id) {
        tx.send(PipelineCommand::Cancel).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Run {} is not active", run_id))
    }
}

#[tauri::command]
pub async fn pause_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    let active = controller.active.lock().await;
    if let Some(tx) = active.get(&run_id) {
        tx.send(PipelineCommand::Pause).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Run {} is not active", run_id))
    }
}

#[tauri::command]
pub async fn resume_run(
    controller: State<'_, RunController>,
    run_id: String,
) -> Result<(), String> {
    let active = controller.active.lock().await;
    if let Some(tx) = active.get(&run_id) {
        tx.send(PipelineCommand::Resume).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Run {} is not active", run_id))
    }
}

#[tauri::command]
pub async fn confirm_schema(
    controller: State<'_, RunController>,
    run_id: String,
    columns: Vec<SchemaColumn>,
) -> Result<(), String> {
    let active = controller.active.lock().await;
    if let Some(tx) = active.get(&run_id) {
        tx.send(PipelineCommand::ConfirmSchema(columns)).await.map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err(format!("Run {} is not active", run_id))
    }
}

#[derive(Debug, Serialize)]
pub struct RunInfo {
    pub id: String,
    pub query: String,
    pub status: String,
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
