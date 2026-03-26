use std::path::Path;
use super::ExportRow;

/// Write rows to a CSV file. Includes a "sources" column with semicolon-separated URLs.
pub fn write_csv(path: &Path, columns: &[String], rows: &[ExportRow]) -> Result<(), String> {
    let mut wtr = csv::Writer::from_path(path)
        .map_err(|e| format!("Failed to create CSV file: {e}"))?;

    // Header row: data columns + confidence + sources
    let mut header: Vec<String> = columns.to_vec();
    header.push("confidence".to_string());
    header.push("sources".to_string());
    wtr.write_record(&header).map_err(|e| format!("CSV write error: {e}"))?;

    // Data rows
    for row in rows {
        let mut record: Vec<String> = columns
            .iter()
            .map(|col| {
                row.data
                    .get(col)
                    .and_then(|v| match v {
                        serde_json::Value::String(s) => Some(s.clone()),
                        serde_json::Value::Null => None,
                        other => Some(other.to_string()),
                    })
                    .unwrap_or_default()
            })
            .collect();
        record.push(format!("{:.2}", row.confidence));
        let sources_str: Vec<String> = row.sources.iter().map(|s| s.url.clone()).collect();
        record.push(sources_str.join("; "));
        wtr.write_record(&record).map_err(|e| format!("CSV write error: {e}"))?;
    }

    wtr.flush().map_err(|e| format!("CSV flush error: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::ExportSource;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_export_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let columns = vec!["name".to_string(), "url".to_string()];
        let rows = vec![
            ExportRow {
                data: serde_json::json!({"name": "Acme Corp", "url": "https://acme.com"}),
                confidence: 0.95,
                sources: vec![ExportSource {
                    url: "https://source1.com".to_string(),
                    title: Some("Source 1".to_string()),
                    snippet: None,
                }],
            },
            ExportRow {
                data: serde_json::json!({"name": "Beta Inc", "url": "https://beta.io"}),
                confidence: 0.80,
                sources: vec![],
            },
        ];

        write_csv(tmp.path(), &columns, &rows).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("name,url,confidence,sources"));
        assert!(content.contains("Acme Corp"));
        assert!(content.contains("0.95"));
        assert!(content.contains("https://source1.com"));
        assert!(content.contains("Beta Inc"));
    }

    #[test]
    fn test_csv_export_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let columns = vec!["col1".to_string()];
        write_csv(tmp.path(), &columns, &[]).unwrap();

        let content = std::fs::read_to_string(tmp.path()).unwrap();
        assert!(content.contains("col1,confidence,sources"));
        // Only header, no data rows
        assert_eq!(content.lines().count(), 1);
    }
}
