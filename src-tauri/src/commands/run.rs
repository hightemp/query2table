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
    schema: Option<Vec<SchemaColumn>>,
) -> Result<StartRunResponse, String> {
    let run_id = new_id();
    let run_type = run_type.unwrap_or_else(|| "table".to_string());

    // Load settings from DB
    let settings_list = state.db.get_all_settings().await.map_err(|e| e.to_string())?;
    let settings: HashMap<String, String> = settings_list.into_iter().collect();

    let mut config = PipelineConfig::from_settings(&settings);
    apply_stop_conditions(&mut config, stop_conditions);
    // Run again: a table run offers the earlier schema for review.
    config.suggested_schema = schema;
    let repo = Arc::new(Repository::new(state.db.pool().clone()));
    let notifications_enabled = settings
        .get("notifications_enabled")
        .map_or(true, |value| value != "false");
    let events = Some(EventPublisher::new(
        app.clone(),
        run_id.clone(),
        notifications_enabled,
    ));

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
    pub dismissed_notices: Option<String>,
    pub title: Option<String>,
    pub pinned_at: Option<i64>,
    pub config: String,
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
        dismissed_notices: r.dismissed_notices,
        title: r.title,
        pinned_at: r.pinned_at,
        config: r.config,
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
        dismissed_notices: r.dismissed_notices,
        title: r.title,
        pinned_at: r.pinned_at,
        config: r.config,
    }).collect())
}

/// A run in the History list.
#[derive(Debug, Serialize)]
pub struct HistoryRun {
    pub id: String,
    pub query: String,
    pub title: Option<String>,
    pub status: String,
    pub run_type: String,
    pub stats: Option<String>,
    pub error: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub pinned_at: Option<i64>,
    pub dismissed_notices: Option<String>,
    pub turn_count: i64,
}

#[derive(Debug, Serialize)]
pub struct HistoryPage {
    pub runs: Vec<HistoryRun>,
    /// Matching runs of each type, ignoring the type filter.
    pub counts: HashMap<String, i64>,
}

#[tauri::command]
pub async fn list_history(
    state: State<'_, AppState>,
    filter: crate::storage::repository::RunListFilter,
) -> Result<HistoryPage, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.list_runs_filtered(&filter).await.map_err(|e| e.to_string())?;
    let counts = repo.count_runs_by_type(&filter).await.map_err(|e| e.to_string())?;
    Ok(HistoryPage {
        runs: rows
            .into_iter()
            .map(|r| HistoryRun {
                id: r.id,
                query: r.query,
                title: r.title,
                status: r.status,
                run_type: r.run_type,
                stats: r.stats,
                error: r.error,
                created_at: r.created_at,
                completed_at: r.completed_at,
                pinned_at: r.pinned_at,
                dismissed_notices: r.dismissed_notices,
                turn_count: r.turn_count,
            })
            .collect(),
        counts: counts.into_iter().collect(),
    })
}

/// Hides runs from History; `restore_runs` undoes it until `purge_runs` or the app closes.
#[tauri::command]
pub async fn delete_runs(
    state: State<'_, AppState>,
    controller: State<'_, RunController>,
    run_ids: Vec<String>,
) -> Result<(), String> {
    let active = controller.active.lock().await;
    if run_ids.iter().any(|id| active.contains_key(id)) {
        return Err("A running run cannot be deleted. Cancel it first.".into());
    }
    drop(active);
    Repository::new(state.db.pool().clone()).soft_delete_runs(&run_ids).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_runs(state: State<'_, AppState>, run_ids: Vec<String>) -> Result<(), String> {
    Repository::new(state.db.pool().clone()).restore_runs(&run_ids).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_runs(state: State<'_, AppState>, run_ids: Vec<String>) -> Result<(), String> {
    Repository::new(state.db.pool().clone()).purge_deleted_runs(Some(&run_ids)).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_run(state: State<'_, AppState>, run_id: String, title: Option<String>) -> Result<(), String> {
    Repository::new(state.db.pool().clone()).rename_run(&run_id, title.as_deref()).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pin_run(state: State<'_, AppState>, run_id: String, pinned: bool) -> Result<(), String> {
    Repository::new(state.db.pool().clone()).set_run_pinned(&run_id, pinned).await.map_err(|e| e.to_string())
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
    Ok(issues_from_details(&run_id, details))
}

/// Stored issues, latest first, as events in the order they happened.
fn issues_from_details(run_id: &str, details: Vec<String>) -> Vec<LlmIssueEvent> {
    let mut issues: Vec<_> = details
        .into_iter()
        .filter_map(|detail| {
            serde_json::from_str(&detail).ok().map(|issue| LlmIssueEvent { run_id: run_id.to_string(), issue })
        })
        .collect();
    issues.reverse();
    issues
}

const MAX_DISMISSED_KINDS: usize = 500;

fn dismissal_json(dismissed: &std::collections::HashMap<String, u32>) -> Result<String, String> {
    if dismissed.len() > MAX_DISMISSED_KINDS || dismissed.keys().any(|key| key.len() > 1000) {
        return Err("Too many notices to dismiss".into());
    }
    let ordered: std::collections::BTreeMap<_, _> = dismissed.iter().collect();
    serde_json::to_string(&ordered).map_err(|e| e.to_string())
}

/// Marks a run's current notices as read; a new kind of notice or a growing count shows them again.
#[tauri::command]
pub async fn dismiss_run_notices(
    state: State<'_, AppState>,
    run_id: String,
    dismissed: std::collections::HashMap<String, u32>,
) -> Result<(), String> {
    let json = dismissal_json(&dismissed)?;
    Repository::new(state.db.pool().clone())
        .set_run_dismissed_notices(&run_id, &json)
        .await
        .map_err(|e| e.to_string())
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
    pub source_count: i64,
}

#[derive(Debug, Serialize)]
pub struct RowSourceInfo {
    pub id: String,
    pub row_id: String,
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
}

async fn load_row_sources(repo: &Repository, row_id: &str) -> Result<Vec<RowSourceInfo>, String> {
    tracing::debug!(row_id, "Loading row sources");
    let sources = repo.get_row_sources(row_id).await.map_err(|error| {
        tracing::warn!(row_id, %error, "Could not load row sources");
        error.to_string()
    })?;
    tracing::debug!(count = sources.len(), "Loaded row sources");
    Ok(sources
        .into_iter()
        .map(|source| RowSourceInfo {
            id: source.id,
            row_id: source.entity_row_id,
            url: source.url,
            title: source.title,
            snippet: source.snippet,
        })
        .collect())
}

#[tauri::command]
pub async fn get_row_sources(
    state: State<'_, AppState>,
    row_id: String,
) -> Result<Vec<RowSourceInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    load_row_sources(&repo, &row_id).await
}

#[tauri::command]
pub async fn get_run_rows(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<Vec<EntityRowInfo>, String> {
    let repo = Repository::new(state.db.pool().clone());
    let rows = repo.get_entity_rows_by_run(&run_id).await.map_err(|e| e.to_string())?;
    let source_counts = repo.count_row_sources_by_run(&run_id).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| {
        let data = serde_json::from_str(&r.data).unwrap_or(serde_json::Value::Object(Default::default()));
        EntityRowInfo {
            source_count: source_counts.get(&r.id).copied().unwrap_or(0),
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
    pub reason: String,
    pub relevance_score: Option<f64>,
    pub low_relevance: bool,
    pub hidden: bool,
    pub visited_at: Option<i64>,
    pub created_at: i64,
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
        reason: r.reason,
        relevance_score: r.relevance_score,
        low_relevance: r.low_relevance,
        hidden: r.hidden,
        visited_at: r.visited_at,
        created_at: r.created_at,
    }).collect())
}

/// Marks a link as opened, or clears the mark.
#[tauri::command]
pub async fn set_link_visited(
    state: State<'_, AppState>,
    link_id: String,
    visited: bool,
) -> Result<(), String> {
    let repo = Repository::new(state.db.pool().clone());
    match repo.set_link_visited(&link_id, visited).await {
        Ok(true) => Ok(()),
        Ok(false) => Err("Link not found".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

/// Hides a link from the run's results and exports, or shows it again.
#[tauri::command]
pub async fn set_link_hidden(
    state: State<'_, AppState>,
    link_id: String,
    hidden: bool,
) -> Result<(), String> {
    let repo = Repository::new(state.db.pool().clone());
    match repo.set_link_hidden(&link_id, hidden).await {
        Ok(true) => Ok(()),
        Ok(false) => Err("Link not found".to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[derive(Debug, Serialize)]
pub struct ResearchStepInfo {
    pub id: String,
    pub turn_index: i64,
    pub step_index: i64,
    pub step_type: String,
    pub content: String,
    pub url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResearchTurnInfo {
    pub turn_index: i64,
    pub question: String,
    pub answer_markdown: Option<String>,
    pub follow_ups: Vec<String>,
    pub status: String,
    pub limits: serde_json::Value,
    pub accounting: Option<serde_json::Value>,
}

impl From<crate::storage::repository::ResearchTurnRow> for ResearchTurnInfo {
    fn from(turn: crate::storage::repository::ResearchTurnRow) -> Self {
        Self {
            turn_index: turn.turn_index,
            question: turn.question,
            answer_markdown: turn.answer_markdown,
            follow_ups: serde_json::from_str(&turn.follow_ups).unwrap_or_default(),
            status: turn.status,
            limits: serde_json::from_str(&turn.limits).unwrap_or_default(),
            accounting: turn.accounting.and_then(|a| serde_json::from_str(&a).ok()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResearchResultInfo {
    /// The latest answer, for views that show one answer.
    pub answer_markdown: Option<String>,
    pub steps: Vec<ResearchStepInfo>,
    pub turns: Vec<ResearchTurnInfo>,
}

#[tauri::command]
pub async fn get_research_result(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<ResearchResultInfo, String> {
    let repo = Repository::new(state.db.pool().clone());
    let steps = repo.get_research_steps(&run_id).await.map_err(|e| e.to_string())?;
    let turns = repo.get_research_turns(&run_id).await.map_err(|e| e.to_string())?;
    Ok(ResearchResultInfo {
        answer_markdown: turns.iter().rev().find_map(|t| t.answer_markdown.clone()),
        steps: steps.into_iter().map(|s| ResearchStepInfo {
            id: s.id,
            turn_index: s.turn_index,
            step_index: s.step_index,
            step_type: s.step_type,
            content: s.content,
            url: s.url,
        }).collect(),
        turns: turns.into_iter().map(ResearchTurnInfo::from).collect(),
    })
}

/// Applies a run's own stop conditions over the defaults from settings.
fn apply_stop_conditions(config: &mut PipelineConfig, stop_conditions: Option<StopConditions>) {
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
}

/// Earlier answered turns as context for a follow-up, with the pages each turn read.
fn research_history(
    turns: &[crate::storage::repository::ResearchTurnRow],
    steps: &[crate::storage::repository::ResearchStepRow],
) -> Vec<crate::roles::research_agent::PriorTurn> {
    turns
        .iter()
        .filter_map(|turn| {
            let answer = turn.answer_markdown.clone()?;
            let sources = steps
                .iter()
                .filter(|s| s.turn_index == turn.turn_index && s.step_type == "fetch")
                .filter_map(|s| {
                    let url = s.url.clone()?;
                    let content = s.content.trim();
                    let first_line = content.lines().next().unwrap_or_default().trim();
                    // Older steps stored a character count or the page text instead of a title.
                    let title = if first_line.is_empty() || first_line.starts_with("Read page (") {
                        url::Url::parse(&url).ok()?.host_str()?.trim_start_matches("www.").to_string()
                    } else {
                        first_line.chars().take(160).collect()
                    };
                    Some((url, title))
                })
                .collect();
            Some(crate::roles::research_agent::PriorTurn { question: turn.question.clone(), answer, sources })
        })
        .collect()
}

/// Asks a follow-up question in a research conversation. The agent searches and reads
/// pages as needed, with the earlier questions, answers and sources as context.
#[tauri::command]
pub async fn ask_follow_up(
    app: AppHandle,
    state: State<'_, AppState>,
    controller: State<'_, RunController>,
    run_id: String,
    question: String,
    stop_conditions: Option<StopConditions>,
) -> Result<(), String> {
    let question = question.trim().to_string();
    if question.is_empty() {
        return Err("Enter a question.".to_string());
    }
    if controller.active.lock().await.contains_key(&run_id) {
        return Err("This conversation is still answering a question.".to_string());
    }
    let repo = Arc::new(Repository::new(state.db.pool().clone()));
    let run = repo
        .get_run(&run_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Run not found".to_string())?;
    if run.run_type != "research" {
        return Err("Follow-up questions are available for research runs only.".to_string());
    }

    let settings_list = state.db.get_all_settings().await.map_err(|e| e.to_string())?;
    let settings: HashMap<String, String> = settings_list.into_iter().collect();
    let mut config = PipelineConfig::from_settings(&settings);
    apply_stop_conditions(&mut config, stop_conditions);

    let turns = repo.get_research_turns(&run_id).await.map_err(|e| e.to_string())?;
    let steps = repo.get_research_steps(&run_id).await.map_err(|e| e.to_string())?;
    let history = research_history(&turns, &steps);
    let turn_index = repo.next_research_turn_index(&run_id).await.map_err(|e| e.to_string())?;

    let notifications_enabled = settings
        .get("notifications_enabled")
        .map_or(true, |value| value != "false");
    let events = Some(EventPublisher::new(app.clone(), run_id.clone(), notifications_enabled));
    let (pipeline, cmd_tx) = ResearchPipeline::follow_up(
        run_id.clone(),
        question,
        turn_index,
        history,
        config,
        repo,
        events,
    );
    controller.active.lock().await.insert(run_id.clone(), cmd_tx);
    let controller_handle = controller.active.clone();
    tokio::spawn(async move {
        let result = pipeline.run().await;
        match &result {
            Ok(state) => info!(run_id = %run_id, state = ?state, "Research follow-up finished"),
            Err(e) => error!(run_id = %run_id, error = %e, "Research follow-up failed"),
        }
        controller_handle.lock().await.remove(&run_id);
    });
    Ok(())
}

#[cfg(test)]
mod control_tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn row_sources_preserve_nullable_fields_and_stay_scoped_to_the_row() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1).connect("sqlite::memory:").await.unwrap();
        let db = crate::storage::db::Database::with_pool(pool.clone()).await;
        db.migrate().await.unwrap();
        let repo = Repository::new(pool);
        repo.create_run("source-test", "Test sources", "{}").await.unwrap();
        let first = repo.create_entity_row("source-test", "{}", 0.8, "final").await.unwrap();
        let other = repo.create_entity_row("source-test", "{}", 0.9, "final").await.unwrap();
        repo.create_row_source(&first, "https://example.com/first", None, None, None).await.unwrap();
        repo.create_row_source(&other, "https://example.com/other", Some("Other"), Some("Evidence"), None).await.unwrap();
        let sources = load_row_sources(&repo, &first).await.unwrap();
        assert_eq!(sources.len(), 1);
        let json = serde_json::to_value(&sources[0]).unwrap();
        assert_eq!(json["row_id"], first);
        assert_eq!(json["url"], "https://example.com/first");
        assert!(json["title"].is_null() && json["snippet"].is_null());
        assert!(load_row_sources(&repo, "missing").await.unwrap().is_empty());
    }

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

#[cfg(test)]
mod notice_tests {
    use super::*;

    #[test]
    fn issues_are_returned_oldest_first_with_unreadable_rows_skipped() {
        // The repository returns the latest issues first.
        let details = vec![
            r#"{"code":"timeout","provider":"p","model":"m","stage":null,"message":"second","max_tokens":1,"prompt_tokens":null,"completion_tokens":null,"reasoning_tokens":null,"retry_after_ms":null,"attempt":2,"max_attempts":4,"will_retry":false}"#.to_string(),
            "not json".to_string(),
            r#"{"code":"timeout","provider":"p","model":"m","stage":null,"message":"first","max_tokens":1,"prompt_tokens":null,"completion_tokens":null,"reasoning_tokens":null,"retry_after_ms":null,"attempt":1,"max_attempts":4,"will_retry":true}"#.to_string(),
        ];
        let issues = issues_from_details("run", details);
        let messages: Vec<_> = issues.iter().map(|i| i.issue.message.as_str()).collect();
        assert_eq!(messages, ["first", "second"]);
    }

    #[test]
    fn dismissals_are_validated_before_saving() {
        assert_eq!(
            dismissal_json(&std::collections::HashMap::from([("cost:unknown".to_string(), 2u32)])).unwrap(),
            r#"{"cost:unknown":2}"#
        );
        let huge: std::collections::HashMap<String, u32> =
            (0..600).map(|i| (format!("llm:{i}"), 1)).collect();
        assert!(dismissal_json(&huge).is_err());
    }
}

#[cfg(test)]
mod research_conversation_tests {
    use super::*;
    use crate::storage::repository::{ResearchStepRow, ResearchTurnRow};

    fn turn(index: i64, question: &str, answer: Option<&str>, status: &str) -> ResearchTurnRow {
        ResearchTurnRow {
            id: format!("t{index}"),
            run_id: "run".into(),
            turn_index: index,
            question: question.into(),
            answer_markdown: answer.map(String::from),
            follow_ups: r#"["Next?"]"#.into(),
            status: status.into(),
            limits: r#"{"target_row_count":16}"#.into(),
            accounting: None,
            created_at: 0,
        }
    }

    fn step(turn_index: i64, kind: &str, content: &str, url: Option<&str>) -> ResearchStepRow {
        ResearchStepRow {
            id: format!("{turn_index}{content}"),
            run_id: "run".into(),
            turn_index,
            step_index: 0,
            step_type: kind.into(),
            content: content.into(),
            url: url.map(String::from),
            created_at: 0,
        }
    }

    #[test]
    fn history_includes_answered_turns_and_the_pages_they_read() {
        let turns = [
            turn(0, "Find proxies", Some("Use Proxy-Seller."), "completed"),
            turn(1, "Cancelled question", None, "cancelled"),
            turn(2, "Cheapest?", Some("Proxy5."), "completed"),
        ];
        let steps = [
            step(0, "fetch", "Proxy-Seller", Some("https://proxy-seller.me/")),
            step(0, "fetch", "Read page (10 characters of content).", Some("https://www.froxy.com/")),
            step(0, "search", "proxies", None),
            step(2, "fetch", "Line one\nraw page text", Some("https://proxy5.net/")),
        ];
        let history = research_history(&turns, &steps);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].question, "Find proxies");
        assert_eq!(
            history[0].sources,
            vec![
                ("https://proxy-seller.me/".to_string(), "Proxy-Seller".to_string()),
                ("https://www.froxy.com/".to_string(), "froxy.com".to_string()),
            ]
        );
        assert_eq!(history[1].sources, vec![("https://proxy5.net/".to_string(), "Line one".to_string())]);
    }

    #[test]
    fn stop_conditions_override_the_settings() {
        let mut config = PipelineConfig::from_settings(&HashMap::new());
        apply_stop_conditions(
            &mut config,
            Some(StopConditions { target_row_count: Some(12), max_budget_usd: Some(0.5), max_duration_seconds: Some(300) }),
        );
        assert_eq!(config.stop.target_row_count, 12);
        assert_eq!((config.max_budget_usd, config.stop.max_budget_usd), (0.5, 0.5));
        assert_eq!(config.stop.max_duration_secs, 300);
    }

    #[test]
    fn turns_are_sent_with_parsed_follow_ups_limits_and_cost() {
        let mut answered = turn(0, "Find proxies", Some("Use Proxy-Seller."), "completed");
        answered.accounting = Some(r#"{"spent_usd":0.25}"#.into());
        let info = ResearchTurnInfo::from(answered);
        assert_eq!(info.follow_ups, vec!["Next?".to_string()]);
        assert_eq!(info.limits["target_row_count"], 16);
        assert_eq!(info.accounting.unwrap()["spent_usd"], 0.25);
    }
}
