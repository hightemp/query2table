use std::path::Path;
use rust_xlsxwriter::{Workbook, Format, Color};
use super::ExportRow;

/// Write rows to an XLSX file with a header row and formatting.
pub fn write_xlsx(path: &Path, columns: &[String], rows: &[ExportRow]) -> Result<(), String> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Results").map_err(|e| format!("XLSX error: {e}"))?;

    let header_fmt = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0x4472C4))
        .set_font_color(Color::White);

    // Write header: data columns + confidence + sources
    let mut col_idx: u16 = 0;
    for col_name in columns {
        sheet.write_string_with_format(0, col_idx, col_name, &header_fmt)
            .map_err(|e| format!("XLSX write error: {e}"))?;
        sheet.set_column_width(col_idx, 20)
            .map_err(|e| format!("XLSX width error: {e}"))?;
        col_idx += 1;
    }
    sheet.write_string_with_format(0, col_idx, "confidence", &header_fmt)
        .map_err(|e| format!("XLSX write error: {e}"))?;
    sheet.set_column_width(col_idx, 12)
        .map_err(|e| format!("XLSX width error: {e}"))?;
    col_idx += 1;
    sheet.write_string_with_format(0, col_idx, "sources", &header_fmt)
        .map_err(|e| format!("XLSX write error: {e}"))?;
    sheet.set_column_width(col_idx, 40)
        .map_err(|e| format!("XLSX width error: {e}"))?;

    // Write data rows
    for (row_idx, row) in rows.iter().enumerate() {
        let excel_row = (row_idx + 1) as u32;
        let mut ci: u16 = 0;

        for col_name in columns {
            let val = row.data.get(col_name).map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            }).unwrap_or_default();
            sheet.write_string(excel_row, ci, &val)
                .map_err(|e| format!("XLSX write error: {e}"))?;
            ci += 1;
        }

        sheet.write_number(excel_row, ci, row.confidence)
            .map_err(|e| format!("XLSX write error: {e}"))?;
        ci += 1;

        let sources_str: Vec<String> = row.sources.iter().map(|s| s.url.clone()).collect();
        sheet.write_string(excel_row, ci, &sources_str.join("; "))
            .map_err(|e| format!("XLSX write error: {e}"))?;
    }

    workbook.save(path).map_err(|e| format!("Failed to save XLSX file: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::{ExportRow, ExportSource};
    use tempfile::NamedTempFile;

    #[test]
    fn test_xlsx_export_basic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("xlsx");
        let columns = vec!["company".to_string(), "website".to_string()];
        let rows = vec![ExportRow {
            data: serde_json::json!({"company": "Acme", "website": "https://acme.com"}),
            confidence: 0.88,
            sources: vec![ExportSource {
                url: "https://source.com".to_string(),
                title: None,
                snippet: None,
            }],
        }];

        write_xlsx(&path, &columns, &rows).unwrap();
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }

    #[test]
    fn test_xlsx_export_empty() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().with_extension("xlsx");
        let columns = vec!["col1".to_string()];
        write_xlsx(&path, &columns, &[]).unwrap();
        assert!(path.exists());
    }
}
