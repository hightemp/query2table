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

    // Check run type
    let run = repo.get_run(&request.run_id).await
        .map_err(|e| format!("Failed to get run: {e}"))?
        .ok_or_else(|| "Run not found".to_string())?;

    let path = std::path::Path::new(&request.path);

    if run.run_type == "images" {
        // Export image results
        let image_rows = repo.get_image_results(&request.run_id).await
            .map_err(|e| format!("Failed to get image results: {e}"))?;

        let columns = vec![
            "image_url".to_string(),
            "thumbnail_url".to_string(),
            "title".to_string(),
            "source_url".to_string(),
            "width".to_string(),
            "height".to_string(),
            "relevance_score".to_string(),
        ];

        let export_rows: Vec<ExportRow> = image_rows.iter().map(|img| {
            let data = serde_json::json!({
                "image_url": img.image_url,
                "thumbnail_url": img.thumbnail_url,
                "title": img.title,
                "source_url": img.source_url,
                "width": img.width.map(|w| w.to_string()).unwrap_or_default(),
                "height": img.height.map(|h| h.to_string()).unwrap_or_default(),
                "relevance_score": img.relevance_score.map(|s| format!("{:.2}", s)).unwrap_or_default(),
            });
            ExportRow {
                data,
                confidence: img.relevance_score.unwrap_or(0.0),
                sources: vec![],
            }
        }).collect();

        export_to_file(path, &columns, &export_rows, format)
    } else {
        // Export table results (original logic)
        let schema = repo.get_run_schema(&request.run_id).await
            .map_err(|e| format!("Failed to get schema: {e}"))?
            .ok_or_else(|| "No schema found for this run".to_string())?;

        let columns: Vec<SchemaColumn> = serde_json::from_str(&schema.columns)
            .map_err(|e| format!("Failed to parse schema columns: {e}"))?;
        let column_names: Vec<String> = columns.iter().map(|c| c.name.clone()).collect();

        let entity_rows = repo.get_entity_rows_by_run(&request.run_id).await
            .map_err(|e| format!("Failed to get entity rows: {e}"))?;

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

        export_to_file(path, &column_names, &export_rows, format)
    }
}
