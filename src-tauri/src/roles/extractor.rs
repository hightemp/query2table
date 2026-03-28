use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::providers::llm::{LlmManager, LlmError, Message};
use crate::storage::models::SchemaColumn;
use super::document_parser::ParsedDocument;

/// A single extracted entity row from a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRow {
    pub data: serde_json::Value,
    pub confidence: f64,
    pub source_url: String,
    pub source_title: String,
}

/// Result of extraction from a single document.
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub rows: Vec<ExtractedRow>,
    pub page_url: String,
}

fn build_system_prompt(columns: &[SchemaColumn]) -> String {
    let col_desc: Vec<String> = columns.iter().map(|c| {
        format!("- {} ({}{}): {}", c.name, c.col_type,
            if c.required { ", required" } else { "" },
            c.description)
    }).collect();

    format!(
        r#"You are a data extraction specialist. Given a web page text and a target table schema, extract all matching entities.

Table schema columns:
{}

Rules:
1. Extract ALL entities matching the schema from the text. There may be 0, 1, or many.
2. Each entity is a JSON object with column names as keys.
3. Only include values you can directly find or infer from the text.
4. For required columns, if the value is not available, skip the entity.
5. Assign a confidence score (0.0-1.0) to each entity based on how much data was found.
6. Return empty rows array if no matching entities are found.

Respond with valid JSON: {{"rows": [{{"data": {{...}}, "confidence": 0.9}}, ...]}}. No markdown, no explanation."#,
        col_desc.join("\n")
    )
}

/// Extracts structured entity rows from parsed documents using LLM.
pub struct Extractor;

impl Extractor {
    /// Extract entities from a single parsed document.
    /// If `max_text_chars` is `Some(n)`, the document text is truncated to `n` characters before sending to LLM.
    pub async fn extract(
        document: &ParsedDocument,
        columns: &[SchemaColumn],
        llm: &LlmManager,
        max_text_chars: Option<usize>,
    ) -> Result<ExtractionResult, LlmError> {
        debug!(url = %document.url, text_len = document.text.len(), "Extracting entities");

        // Truncate text to avoid exceeding token limits
        let text = if let Some(max) = max_text_chars {
            if document.text.len() > max {
                &document.text[..max]
            } else {
                &document.text
            }
        } else {
            &document.text
        };

        let system_prompt = build_system_prompt(columns);

        let messages = vec![
            Message::system(system_prompt),
            Message::user(format!(
                "Extract entities from this web page:\n\nTitle: {}\nURL: {}\n\nContent:\n{}",
                document.title, document.url, text
            )),
        ];

        let response = llm.complete(messages, true).await?;

        #[derive(Deserialize)]
        struct LlmResponse {
            rows: Vec<LlmRow>,
        }

        #[derive(Deserialize)]
        struct LlmRow {
            data: serde_json::Value,
            confidence: Option<f64>,
        }

        let parsed: LlmResponse = serde_json::from_str(&response.content)
            .map_err(|e| LlmError::ParseError(format!(
                "Failed to parse extraction result: {}. Response: {}",
                e, response.content
            )))?;

        let rows: Vec<ExtractedRow> = parsed.rows.into_iter().map(|r| {
            ExtractedRow {
                data: r.data,
                confidence: r.confidence.unwrap_or(0.5),
                source_url: document.url.clone(),
                source_title: document.title.clone(),
            }
        }).collect();

        debug!(url = %document.url, rows_found = rows.len(), "Extraction complete");

        Ok(ExtractionResult {
            rows,
            page_url: document.url.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extracted_row_serialize() {
        let row = ExtractedRow {
            data: serde_json::json!({"name": "Acme Corp", "website": "https://acme.com"}),
            confidence: 0.9,
            source_url: "https://example.com/list".to_string(),
            source_title: "Top Companies".to_string(),
        };

        let json = serde_json::to_string(&row).unwrap();
        assert!(json.contains("Acme Corp"));
    }

    #[test]
    fn test_extraction_response_deserialize() {
        let json = r#"{
            "rows": [
                {"data": {"name": "Company A", "employees": 100}, "confidence": 0.95},
                {"data": {"name": "Company B", "employees": 50}, "confidence": 0.7}
            ]
        }"#;

        #[derive(serde::Deserialize)]
        struct LlmResponse { rows: Vec<LlmRow> }
        #[derive(serde::Deserialize)]
        struct LlmRow { data: serde_json::Value, confidence: Option<f64> }

        let parsed: LlmResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.rows.len(), 2);
        assert_eq!(parsed.rows[0].confidence, Some(0.95));
    }

    #[test]
    fn test_build_system_prompt() {
        let columns = vec![
            SchemaColumn {
                name: "name".to_string(),
                col_type: "text".to_string(),
                description: "Company name".to_string(),
                required: true,
            },
        ];
        let prompt = build_system_prompt(&columns);
        assert!(prompt.contains("name (text, required)"));
    }

    #[test]
    fn test_text_truncation_with_limit() {
        let long_text = "a".repeat(15000);
        let max = Some(12000usize);
        let truncated = if let Some(m) = max {
            if long_text.len() > m { &long_text[..m] } else { &long_text }
        } else {
            &long_text
        };
        assert_eq!(truncated.len(), 12000);
    }

    #[test]
    fn test_text_truncation_disabled() {
        let long_text = "a".repeat(15000);
        let max: Option<usize> = None;
        let truncated = if let Some(m) = max {
            if long_text.len() > m { &long_text[..m] } else { &long_text }
        } else {
            &long_text
        };
        assert_eq!(truncated.len(), 15000);
    }
}
