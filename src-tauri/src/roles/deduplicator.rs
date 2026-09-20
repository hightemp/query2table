use strsim::normalized_damerau_levenshtein;
use tracing::debug;

use super::extractor::ExtractedRow;

/// A group of deduplicated entities.
#[derive(Debug, Clone)]
pub struct DedupGroup {
    pub group_id: String,
    pub merged: ExtractedRow,
    pub sources: Vec<String>,
    pub member_count: usize,
    /// Input positions are retained so persistence never guesses entity identity.
    pub member_indices: Vec<usize>,
}

/// Result of deduplication.
#[derive(Debug, Clone)]
pub struct DedupResult {
    pub groups: Vec<DedupGroup>,
    pub total_input: usize,
    pub unique_entities: usize,
    pub duplicates_merged: usize,
}

/// Deduplicates extracted entity rows using fuzzy string matching.
pub struct Deduplicator;

impl Deduplicator {
    pub fn name_column(columns: &[crate::storage::models::SchemaColumn]) -> &str {
        columns
            .iter()
            .find(|column| column.name.eq_ignore_ascii_case("name"))
            .or_else(|| columns.iter().find(|column| column.col_type == "text"))
            .map(|column| column.name.as_str())
            .unwrap_or("")
    }

    /// Deduplicate rows by comparing their "name" field (or first text column).
    pub fn deduplicate(
        rows: &[ExtractedRow],
        name_column: &str,
        similarity_threshold: f64,
    ) -> DedupResult {
        debug!(
            row_count = rows.len(),
            threshold = %similarity_threshold,
            "Deduplicating rows"
        );

        if rows.is_empty() {
            return DedupResult {
                groups: Vec::new(),
                total_input: 0,
                unique_entities: 0,
                duplicates_merged: 0,
            };
        }

        // Group rows by similarity of their name column value
        let mut groups: Vec<Vec<usize>> = Vec::new();
        let mut assigned = vec![false; rows.len()];

        for i in 0..rows.len() {
            if assigned[i] {
                continue;
            }

            let mut group = vec![i];
            assigned[i] = true;

            let name_i = Self::get_name(&rows[i], name_column);

            for j in (i + 1)..rows.len() {
                if assigned[j] {
                    continue;
                }

                let name_j = Self::get_name(&rows[j], name_column);

                if Self::is_similar(&name_i, &name_j, similarity_threshold) {
                    group.push(j);
                    assigned[j] = true;
                }
            }

            groups.push(group);
        }

        let total_input = rows.len();
        let unique_entities = groups.len();
        let duplicates_merged = total_input - unique_entities;

        let dedup_groups: Vec<DedupGroup> = groups.into_iter().map(|indices| {
            let merged = Self::merge_group(rows, &indices);
            let sources: Vec<String> = indices.iter()
                .map(|&i| rows[i].source_url.clone())
                .collect();
            let group_id = crate::utils::id::new_id();

                DedupGroup {
                    group_id,
                    merged,
                    sources,
                    member_count: indices.len(),
                    member_indices: indices,
                }
            })
            .collect();

        debug!(
            unique = unique_entities,
            merged = duplicates_merged,
            "Deduplication complete"
        );

        DedupResult {
            groups: dedup_groups,
            total_input,
            unique_entities,
            duplicates_merged,
        }
    }

    fn get_name(row: &ExtractedRow, name_column: &str) -> String {
        if name_column.is_empty() {
            return String::new();
        }
        row.data
            .get(name_column)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase()
            .trim()
            .to_string()
    }

    fn is_similar(a: &str, b: &str, threshold: f64) -> bool {
        if a.is_empty() || b.is_empty() {
            return false;
        }

        // Exact match (fast path)
        if a == b {
            return true;
        }

        let similarity = normalized_damerau_levenshtein(a, b);
        similarity >= threshold
    }

    /// Merge a group of rows: take the one with highest confidence, enrich missing fields.
    fn merge_group(rows: &[ExtractedRow], indices: &[usize]) -> ExtractedRow {
        if indices.len() == 1 {
            return rows[indices[0]].clone();
        }

        // Start with the highest-confidence row
        let best_idx = indices.iter()
            .copied()
            .max_by(|&a, &b| rows[a].confidence.partial_cmp(&rows[b].confidence).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();

        let mut merged = rows[best_idx].clone();

        // Fill in missing fields from other rows
        if let Some(merged_obj) = merged.data.as_object_mut() {
            for &idx in indices {
                if idx == best_idx {
                    continue;
                }
                if let Some(other_obj) = rows[idx].data.as_object() {
                    for (key, value) in other_obj {
                        let should_fill = merged_obj.get(key).map_or(true, |v| {
                            v.is_null() || v.as_str().map_or(false, |s| s.is_empty())
                        });
                        if should_fill && !value.is_null() {
                            merged_obj.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_row(name: &str, extra: serde_json::Value, confidence: f64) -> ExtractedRow {
        let mut data = serde_json::Map::new();
        data.insert("name".to_string(), json!(name));
        if let Some(obj) = extra.as_object() {
            for (k, v) in obj {
                data.insert(k.clone(), v.clone());
            }
        }
        ExtractedRow {
            data: serde_json::Value::Object(data),
            confidence,
            source_url: format!("https://source-{}.com", name.replace(' ', "-")),
            source_title: "Test".to_string(),
        }
    }

    #[test]
    fn membership_and_zero_values_survive_high_confidence_merges() {
        let rows = vec![
            make_row("Alpha", json!({"count":0,"enabled":false}), 0.9),
            make_row("Beta", json!({}), 0.8),
            make_row(
                "Alpha",
                json!({"count":10,"enabled":true,"note":"extra"}),
                0.5,
            ),
        ];
        let result = Deduplicator::deduplicate(&rows, "name", 0.9);
        assert_eq!(result.groups[0].member_indices, vec![0, 2]);
        assert_eq!(result.groups[0].merged.data["count"], 0);
        assert_eq!(result.groups[0].merged.data["enabled"], false);
        assert_eq!(result.groups[0].merged.data["note"], "extra");
        assert_eq!(result.groups[1].member_indices, vec![1]);
    }
    #[test]
    fn schema_identity_column_is_explicit_and_missing_names_never_merge() {
        use crate::storage::models::SchemaColumn;
        let column = |name: &str, kind: &str| SchemaColumn {
            name: name.into(),
            col_type: kind.into(),
            description: String::new(),
            required: false,
        };
        assert_eq!(
            Deduplicator::name_column(&[column("Company", "text")]),
            "Company"
        );
        assert_eq!(
            Deduplicator::name_column(&[column("Description", "text"), column("Name", "text")]),
            "Name"
        );
        assert_eq!(Deduplicator::name_column(&[column("Year", "number")]), "");
        let row = ExtractedRow {
            data: json!({"":"same"}),
            confidence: 0.8,
            source_url: String::new(),
            source_title: String::new(),
        };
        assert_eq!(
            Deduplicator::deduplicate(&[row.clone(), row], "", 0.8).unique_entities,
            2
        );
    }

    #[test]
    fn test_no_duplicates() {
        let rows = vec![
            make_row("Apple Inc", json!({}), 0.9),
            make_row("Google LLC", json!({}), 0.8),
            make_row("Microsoft", json!({}), 0.85),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        assert_eq!(result.unique_entities, 3);
        assert_eq!(result.duplicates_merged, 0);
    }

    #[test]
    fn test_exact_duplicates() {
        let rows = vec![
            make_row("Apple Inc", json!({"website": "https://apple.com"}), 0.9),
            make_row("Apple Inc", json!({"employees": 150000}), 0.7),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        assert_eq!(result.unique_entities, 1);
        assert_eq!(result.duplicates_merged, 1);
        // Merged row should have the best confidence entity enriched
        let merged = &result.groups[0].merged;
        assert_eq!(merged.data["website"], "https://apple.com");
    }

    #[test]
    fn test_fuzzy_duplicates() {
        let rows = vec![
            make_row("Apple Inc.", json!({}), 0.9),
            make_row("Apple Inc", json!({}), 0.8),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        assert_eq!(result.unique_entities, 1);
    }

    #[test]
    fn test_different_entities_not_merged() {
        let rows = vec![
            make_row("Apple Inc", json!({}), 0.9),
            make_row("Amazon AWS", json!({}), 0.8),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        assert_eq!(result.unique_entities, 2);
    }

    #[test]
    fn test_empty_rows() {
        let result = Deduplicator::deduplicate(&[], "name", 0.85);
        assert_eq!(result.unique_entities, 0);
        assert_eq!(result.total_input, 0);
    }

    #[test]
    fn test_merge_enriches_fields() {
        let rows = vec![
            make_row("Acme", json!({"website": "https://acme.com"}), 0.9),
            make_row("Acme", json!({"employees": 50, "location": "NYC"}), 0.7),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        let merged = &result.groups[0].merged;
        assert_eq!(merged.data["website"], "https://acme.com");
        assert_eq!(merged.data["employees"], 50);
        assert_eq!(merged.data["location"], "NYC");
    }

    #[test]
    fn test_similarity_function() {
        assert!(Deduplicator::is_similar("apple inc", "apple inc.", 0.85));
        assert!(!Deduplicator::is_similar("apple", "microsoft", 0.85));
        assert!(!Deduplicator::is_similar("", "apple", 0.85));
    }

    #[test]
    fn test_sources_collected() {
        let rows = vec![
            make_row("Acme", json!({}), 0.9),
            make_row("Acme", json!({}), 0.7),
        ];

        let result = Deduplicator::deduplicate(&rows, "name", 0.85);
        assert_eq!(result.groups[0].sources.len(), 2);
        assert_eq!(result.groups[0].member_count, 2);
    }
}
