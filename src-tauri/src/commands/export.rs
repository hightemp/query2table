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
    let repo = Repository::new(state.db.pool().clone());

    // Check run type
    let run = repo.get_run(&request.run_id).await
        .map_err(|e| format!("Failed to get run: {e}"))?
        .ok_or_else(|| "Run not found".to_string())?;

    let path = std::path::Path::new(&request.path);

    // Research mode exports the Markdown answer directly (format is ignored).
    if run.run_type == "research" {
        let answer = repo.get_research_result(&request.run_id).await
            .map_err(|e| format!("Failed to get research result: {e}"))?
            .map(|r| r.answer_markdown)
            .unwrap_or_default();
        std::fs::write(path, answer)
            .map_err(|e| format!("Failed to write file: {e}"))?;
        return Ok(());
    }

    let format = ExportFormat::from_str(&request.format)
        .ok_or_else(|| format!("Unknown export format: {}", request.format))?;

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
    } else if run.run_type == "links" {
        // Hidden links and links below the relevance threshold are left out.
        let links = repo.get_link_results(&request.run_id).await
            .map_err(|e| format!("Failed to get link results: {e}"))?;
        let (columns, export_rows) = link_export_rows(&links);
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

/// Columns and rows for a links export. Hidden links and links below the relevance
/// threshold are left out, matching what the results view shows by default.
fn link_export_rows(
    links: &[crate::storage::repository::LinkResultRow],
) -> (Vec<String>, Vec<ExportRow>) {
    let columns = ["url", "title", "description", "reason", "relevance_score", "visited"]
        .iter()
        .map(|c| c.to_string())
        .collect();
    let rows = links
        .iter()
        .filter(|link| !link.hidden && !link.low_relevance)
        .map(|link| ExportRow {
            data: serde_json::json!({
                "url": link.url,
                "title": link.title,
                "description": link.description,
                "reason": link.reason,
                "relevance_score": link.relevance_score.map(|s| format!("{:.2}", s)).unwrap_or_default(),
                "visited": if link.visited_at.is_some() { "yes" } else { "" },
            }),
            confidence: link.relevance_score.unwrap_or(0.0),
            sources: vec![],
        })
        .collect();
    (columns, rows)
}

#[cfg(test)]
mod link_export_tests {
    use super::*;
    use crate::storage::repository::LinkResultRow;

    fn link(id: &str, low_relevance: bool, hidden: bool, visited_at: Option<i64>) -> LinkResultRow {
        LinkResultRow {
            id: id.into(),
            run_id: "run".into(),
            url: format!("https://{id}.example/"),
            title: id.into(),
            description: "Docs".into(),
            reason: "Matches".into(),
            relevance_score: Some(0.9),
            low_relevance,
            hidden,
            visited_at,
            created_at: 0,
        }
    }

    #[test]
    fn exports_only_shown_links_with_review_state() {
        let (columns, rows) = link_export_rows(&[
            link("kept", false, false, Some(5)),
            link("weak", true, false, None),
            link("hidden", false, true, None),
        ]);
        assert_eq!(columns, ["url", "title", "description", "reason", "relevance_score", "visited"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].data["url"], "https://kept.example/");
        assert_eq!(rows[0].data["reason"], "Matches");
        assert_eq!(rows[0].data["relevance_score"], "0.90");
        assert_eq!(rows[0].data["visited"], "yes");
    }
}
