use std::path::Path;
use serde::Serialize;
use super::ExportRow;

/// A JSON-friendly export record.
#[derive(Serialize)]
struct JsonExportRecord {
    #[serde(flatten)]
    data: serde_json::Value,
    confidence: f64,
    sources: Vec<JsonExportSource>,
}

#[derive(Serialize)]
struct JsonExportSource {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snippet: Option<String>,
}

/// Write rows to a JSON file as an array of objects.
pub fn write_json(path: &Path, _columns: &[String], rows: &[ExportRow]) -> Result<(), String> {
    let records: Vec<JsonExportRecord> = rows
        .iter()
        .map(|row| JsonExportRecord {
            data: row.data.clone(),
            confidence: row.confidence,
            sources: row
                .sources
                .iter()
                .map(|s| JsonExportSource {
                    url: s.url.clone(),
                    title: s.title.clone(),
                    snippet: s.snippet.clone(),
                })
                .collect(),
        })
        .collect();

    let json = serde_json::to_string_pretty(&records)
        .map_err(|e| format!("JSON serialization error: {e}"))?;

    std::fs::write(path, json).map_err(|e| format!("Failed to write JSON file: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ExportSource;
    use tempfile::NamedTempFile;

    #[test]
    fn test_json_export_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let columns = vec!["name".to_string()];
        let rows = vec![ExportRow {
            data: serde_json::json!({"name": "Test Corp"}),
            confidence: 0.9,
            sources: vec![ExportSource {
                url: "https://example.com".to_string(),
                title: Some("Example".to_string()),
                snippet: Some("A snippet".to_string()),
            }],
        }];

        write_json(tmp.path(), &columns, &rows).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["name"], "Test Corp");
        assert_eq!(parsed[0]["confidence"], 0.9);
        assert_eq!(parsed[0]["sources"][0]["url"], "https://example.com");
    }

    #[test]
    fn test_json_export_empty() {
        let tmp = NamedTempFile::new().unwrap();
        write_json(tmp.path(), &[], &[]).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_empty());
    }
}
