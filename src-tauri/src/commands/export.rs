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
    /// Research only: export just this turn of the conversation.
    #[serde(default)]
    pub turn_index: Option<i64>,
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

    // Research mode exports the conversation as Markdown (format is ignored).
    if run.run_type == "research" {
        let turns = repo.get_research_turns(&request.run_id).await
            .map_err(|e| format!("Failed to get research conversation: {e}"))?;
        let steps = repo.get_research_steps(&request.run_id).await
            .map_err(|e| format!("Failed to get research steps: {e}"))?;
        std::fs::write(path, conversation_markdown(&turns, &steps, request.turn_index))
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

const READ_COUNT_PREFIX: &str = "Read page (";

/// Markdown of a research conversation: each turn's question, answer and the pages read for it.
/// `only` limits the export to one turn.
fn conversation_markdown(
    turns: &[crate::storage::repository::ResearchTurnRow],
    steps: &[crate::storage::repository::ResearchStepRow],
    only: Option<i64>,
) -> String {
    turns
        .iter()
        .filter(|turn| only.map_or(true, |index| turn.turn_index == index))
        .map(|turn| {
            let mut out = format!("# {}\n\n", turn.question.trim());
            match turn.answer_markdown.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
                Some(answer) => out.push_str(answer),
                None => out.push_str("_No answer was produced._"),
            }
            out.push('\n');
            let pages: Vec<String> = steps
                .iter()
                .filter(|step| step.turn_index == turn.turn_index && step.step_type == "fetch")
                .filter_map(|step| {
                    let url = step.url.as_deref()?;
                    let title = step.content.trim();
                    // Older runs stored a character count or the page text instead of a title.
                    let title = if title.is_empty() || title.starts_with(READ_COUNT_PREFIX) || title.contains('\n') {
                        url::Url::parse(url).ok()?.host_str()?.trim_start_matches("www.").to_string()
                    } else {
                        title.to_string()
                    };
                    Some(format!("[{}]({url})", title.replace('[', "\\[").replace(']', "\\]")))
                })
                .collect();
            if !pages.is_empty() {
                out.push_str("\n**Pages read**\n\n");
                for (i, page) in pages.iter().enumerate() {
                    out.push_str(&format!("{}. {page}\n", i + 1));
                }
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n---\n\n")
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

#[cfg(test)]
mod research_export_tests {
    use super::*;
    use crate::storage::repository::{ResearchStepRow, ResearchTurnRow};

    fn turn(index: i64, question: &str, answer: Option<&str>) -> ResearchTurnRow {
        ResearchTurnRow {
            id: format!("t{index}"),
            run_id: "run".into(),
            turn_index: index,
            question: question.into(),
            answer_markdown: answer.map(String::from),
            follow_ups: "[]".into(),
            status: "completed".into(),
            limits: "{}".into(),
            accounting: None,
            created_at: index,
        }
    }

    fn read(turn_index: i64, url: &str, title: &str) -> ResearchStepRow {
        ResearchStepRow {
            id: format!("{turn_index}{url}"),
            run_id: "run".into(),
            turn_index,
            step_index: 0,
            step_type: "fetch".into(),
            content: title.into(),
            url: Some(url.into()),
            created_at: 0,
        }
    }

    #[test]
    fn a_conversation_exports_every_turn_with_its_read_pages() {
        let turns = [turn(0, "Find proxies", Some("Use Proxy-Seller.")), turn(1, "Cheapest?", Some("Proxy5."))];
        let steps = [read(0, "https://proxy-seller.me/", "Proxy-Seller"), read(1, "https://proxy5.net/", "Read page (10 characters of content).")];
        assert_eq!(
            conversation_markdown(&turns, &steps, None),
            "# Find proxies\n\nUse Proxy-Seller.\n\n**Pages read**\n\n1. [Proxy-Seller](https://proxy-seller.me/)\n\n---\n\n# Cheapest?\n\nProxy5.\n\n**Pages read**\n\n1. [proxy5.net](https://proxy5.net/)\n"
        );
        assert_eq!(
            conversation_markdown(&turns, &steps, Some(1)),
            "# Cheapest?\n\nProxy5.\n\n**Pages read**\n\n1. [proxy5.net](https://proxy5.net/)\n"
        );
    }

    #[test]
    fn unanswered_turns_say_so() {
        let turns = [turn(0, "Find proxies", None)];
        assert_eq!(conversation_markdown(&turns, &[], None), "# Find proxies\n\n_No answer was produced._\n");
    }
}
