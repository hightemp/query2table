// Export implementations — CSV, JSON, XLSX.

pub mod csv_export;
pub mod json_export;
pub mod xlsx_export;

use serde::Serialize;

/// A flattened row for export, combining entity data with source info.
#[derive(Debug, Clone, Serialize)]
pub struct ExportRow {
    pub data: serde_json::Value,
    pub confidence: f64,
    pub sources: Vec<ExportSource>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExportSource {
    pub url: String,
    pub title: Option<String>,
    pub snippet: Option<String>,
}

/// Supported export formats.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Csv,
    Json,
    Xlsx,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "csv" => Some(Self::Csv),
            "json" => Some(Self::Json),
            "xlsx" => Some(Self::Xlsx),
            _ => None,
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Xlsx => "xlsx",
        }
    }
}

/// Export rows to the given path in the specified format.
pub fn export_to_file(
    path: &std::path::Path,
    columns: &[String],
    rows: &[ExportRow],
    format: ExportFormat,
) -> Result<(), String> {
    match format {
        ExportFormat::Csv => csv_export::write_csv(path, columns, rows),
        ExportFormat::Json => json_export::write_json(path, columns, rows),
        ExportFormat::Xlsx => xlsx_export::write_xlsx(path, columns, rows),
    }
}
