use tracing::debug;

use crate::storage::models::SchemaColumn;
use super::extractor::ExtractedRow;

/// Result of validating a single row.
#[derive(Debug, Clone)]
pub struct ValidatedRow {
    pub row: ExtractedRow,
    pub is_valid: bool,
    pub issues: Vec<String>,
}

/// Validates extracted rows against the schema using deterministic checks.
pub struct Validator;

impl Validator {
    /// Validate a batch of extracted rows against the schema.
    pub fn validate(
        rows: &[ExtractedRow],
        columns: &[SchemaColumn],
        min_confidence: f64,
    ) -> Vec<ValidatedRow> {
        debug!(row_count = rows.len(), min_confidence = %min_confidence, "Validating rows");

        rows.iter().map(|row| {
            let mut issues = Vec::new();

            // Check confidence threshold
            if row.confidence < min_confidence {
                issues.push(format!(
                    "Confidence {:.2} below threshold {:.2}",
                    row.confidence, min_confidence
                ));
            }

            // Check required columns
            if let Some(obj) = row.data.as_object() {
                for col in columns {
                    if col.required {
                        let has_value = obj.get(&col.name).map_or(false, |v| {
                            !v.is_null() && v.as_str().map_or(true, |s| !s.is_empty())
                        });
                        if !has_value {
                            issues.push(format!("Required column '{}' is missing or empty", col.name));
                        }
                    }
                }

                // Type validation
                for col in columns {
                    if let Some(value) = obj.get(&col.name) {
                        if !value.is_null() {
                            if let Some(issue) = Self::validate_type(value, &col.col_type, &col.name) {
                                issues.push(issue);
                            }
                        }
                    }
                }
            } else {
                issues.push("Row data is not a JSON object".to_string());
            }

            let is_valid = issues.is_empty();

            ValidatedRow {
                row: row.clone(),
                is_valid,
                issues,
            }
        }).collect()
    }

    fn validate_type(value: &serde_json::Value, expected_type: &str, col_name: &str) -> Option<String> {
        match expected_type {
            "number" => {
                if let Some(s) = value.as_str() {
                    if s.parse::<f64>().is_err() {
                        return Some(format!("Column '{}' expected number, got text '{}'", col_name, s));
                    }
                } else if !value.is_number() {
                    return Some(format!("Column '{}' expected number", col_name));
                }
                None
            }
            "url" => {
                if let Some(s) = value.as_str() {
                    if !s.starts_with("http://") && !s.starts_with("https://") {
                        return Some(format!("Column '{}' has invalid URL: '{}'", col_name, s));
                    }
                }
                None
            }
            "boolean" => {
                if let Some(s) = value.as_str() {
                    let lower = s.to_lowercase();
                    if !["true", "false", "yes", "no", "1", "0"].contains(&lower.as_str()) {
                        return Some(format!("Column '{}' has invalid boolean: '{}'", col_name, s));
                    }
                }
                None
            }
            "email" => {
                if let Some(s) = value.as_str() {
                    if !s.contains('@') || !s.contains('.') {
                        return Some(format!("Column '{}' has invalid email: '{}'", col_name, s));
                    }
                }
                None
            }
            _ => None, // "text", "date", etc. — accept anything
        }
    }

    /// Filter only valid rows from validation results.
    pub fn filter_valid(results: &[ValidatedRow]) -> Vec<ExtractedRow> {
        results.iter()
            .filter(|v| v.is_valid)
            .map(|v| v.row.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_columns() -> Vec<SchemaColumn> {
        vec![
            SchemaColumn {
                name: "name".to_string(),
                col_type: "text".to_string(),
                description: "Name".to_string(),
                required: true,
            },
            SchemaColumn {
                name: "website".to_string(),
                col_type: "url".to_string(),
                description: "Website".to_string(),
                required: false,
            },
            SchemaColumn {
                name: "employees".to_string(),
                col_type: "number".to_string(),
                description: "Count".to_string(),
                required: false,
            },
        ]
    }

    #[test]
    fn test_valid_row() {
        let row = ExtractedRow {
            data: json!({"name": "Acme", "website": "https://acme.com", "employees": 100}),
            confidence: 0.9,
            source_url: "https://example.com".to_string(),
            source_title: "Test".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(results[0].is_valid);
        assert!(results[0].issues.is_empty());
    }

    #[test]
    fn test_missing_required_column() {
        let row = ExtractedRow {
            data: json!({"website": "https://acme.com"}),
            confidence: 0.9,
            source_url: "https://example.com".to_string(),
            source_title: "Test".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(!results[0].is_valid);
        assert!(results[0].issues.iter().any(|i| i.contains("name")));
    }

    #[test]
    fn test_low_confidence() {
        let row = ExtractedRow {
            data: json!({"name": "Acme"}),
            confidence: 0.3,
            source_url: "https://example.com".to_string(),
            source_title: "Test".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(!results[0].is_valid);
        assert!(results[0].issues.iter().any(|i| i.contains("Confidence")));
    }

    #[test]
    fn test_invalid_url() {
        let row = ExtractedRow {
            data: json!({"name": "Acme", "website": "not-a-url"}),
            confidence: 0.9,
            source_url: "https://example.com".to_string(),
            source_title: "Test".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(!results[0].is_valid);
        assert!(results[0].issues.iter().any(|i| i.contains("invalid URL")));
    }

    #[test]
    fn test_filter_valid() {
        let rows = vec![
            ExtractedRow {
                data: json!({"name": "Good"}),
                confidence: 0.9,
                source_url: "a".to_string(),
                source_title: "a".to_string(),
            },
            ExtractedRow {
                data: json!({"website": "only"}),
                confidence: 0.1,
                source_url: "b".to_string(),
                source_title: "b".to_string(),
            },
        ];

        let validated = Validator::validate(&rows, &test_columns(), 0.5);
        let valid = Validator::filter_valid(&validated);
        assert_eq!(valid.len(), 1);
        assert_eq!(valid[0].data["name"], "Good");
    }

    #[test]
    fn test_number_as_string_valid() {
        let row = ExtractedRow {
            data: json!({"name": "Acme", "employees": "500"}),
            confidence: 0.9,
            source_url: "a".to_string(),
            source_title: "a".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(results[0].is_valid);
    }

    #[test]
    fn test_number_as_string_invalid() {
        let row = ExtractedRow {
            data: json!({"name": "Acme", "employees": "many"}),
            confidence: 0.9,
            source_url: "a".to_string(),
            source_title: "a".to_string(),
        };

        let results = Validator::validate(&[row], &test_columns(), 0.5);
        assert!(!results[0].is_valid);
    }
}
