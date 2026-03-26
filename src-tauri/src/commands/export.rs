use serde::Deserialize;
use tauri::State;

use crate::AppState;
use crate::export::{ExportFormat, ExportRow, ExportSource, export_to_file};
use crate::storage::models::SchemaColumn;
use crate::storage::repository::Repository;

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub run_id: String,
    pub format: String,
    pub path: String,
}

#[tauri::command]
pub async fn export_run(
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<(), String> {
    let format = ExportFormat::from_str(&request.format)
        .ok_or_else(|| format!("Unknown export format: {}", request.format))?;

    let repo = Repository::new(state.db.pool().clone());

    // Get schema columns for column ordering
    let schema = repo.get_run_schema(&request.run_id).await
        .map_err(|e| format!("Failed to get schema: {e}"))?
        .ok_or_else(|| "No schema found for this run".to_string())?;

    let columns: Vec<SchemaColumn> = serde_json::from_str(&schema.columns)
        .map_err(|e| format!("Failed to parse schema columns: {e}"))?;
    let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

    // Get entity rows
    let entity_rows = repo.get_entity_rows_by_run(&request.run_id).await
        .map_err(|e| format!("Failed to get entity rows: {e}"))?;

    // Build export rows with sources
    let mut export_rows = Vec::with_capacity(entity_rows.len());
    for er in &entity_rows {
        let sources_rows = repo.get_row_sources(&er.id).await
            .map_err(|e| format!("Failed to get sources: {e}"))?;

        let sources: Vec<ExportSource> = sources_rows
            .into_iter()
            .map(|s| ExportSource {
                url: s.url,
                title: s.title,
                snippet: s.snippet,
            })
            .collect();

        let data: serde_json::Value = serde_json::from_str(&er.data)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        export_rows.push(ExportRow {
            data,
            confidence: er.confidence,
            sources,
        });
    }

    let path = std::path::Path::new(&request.path);
    export_to_file(path, &column_names, &export_rows, format)
}
