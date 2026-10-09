use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::providers::llm::{LlmManager, LlmError, Message};
use crate::storage::models::SchemaColumn;
use super::query_interpreter::QueryIntent;

/// A proposed table schema for a research run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedSchema {
    pub columns: Vec<SchemaColumn>,
}

/// The schema designer's instructions; column names and descriptions are written in `language`.
pub(crate) fn system_prompt(language: &str) -> String {
    format!(
        r#"You are a schema designer for a research data collection tool. Given information about the entity type and desired attributes, design an optimal table schema.

The table is read by someone who writes in {language}: write every column name and description in {language}.

Each column must have:
- name: a short column header in {language}, as a person would title it (e.g. "Company", "Website", "Employees"); unique within the table
- type: one of "text", "number", "url", "date", "boolean", "email"
- description: brief explanation of what this column contains
- required: whether this column must have a value (true/false)

Rules:
1. The first column is the entity's name (type "text", required true).
2. Include all attributes the user asked for.
3. Add a column for the source page at the end (type "url", required true).
4. Keep columns concise and practical — aim for 5-12 columns total.
5. Use appropriate types (urls should be "url" type, counts should be "number", etc.)

Respond with valid JSON: {{"columns": [...]}}. No markdown, no explanation."#
    )
}

/// Plans a table schema based on query intent.
pub struct SchemaPlanner;

impl SchemaPlanner {
    /// Generate a proposed schema from the interpreted query intent.
    pub async fn plan(
        intent: &QueryIntent,
        llm: &LlmManager,
    ) -> Result<ProposedSchema, LlmError> {
        debug!(entity_type = %intent.entity_type, "Planning schema");

        let messages = vec![
            Message::system(system_prompt(&intent.language())),
            Message::user(format!(
                "Design a table schema for collecting data about: {}\n\nDesired attributes: {}\nConstraints: {}\nGeo focus: {}{}",
                intent.entity_type,
                intent.attributes.join(", "),
                intent.constraints.join(", "),
                intent.geo.as_deref().unwrap_or("none"),
                if intent.files_context.is_some() {
                    format!("\n\nAttached files (when they hold the data, base the columns on their fields):{}", intent.files_note(6000))
                } else {
                    String::new()
                },
            )),
        ];

        let response = llm.complete_for_stage("planner", messages, true).await?;

        let schema: ProposedSchema = serde_json::from_str(&response.content)
            .map_err(|e| {
                let message = format!("Failed to parse schema: {e}");
                llm.report_invalid_response("planner", &message, &response, crate::providers::llm::IssueOutcome::Stopped);
                LlmError::ParseError(message)
            })?;

        if schema.columns.is_empty() {
            llm.report_invalid_response("planner", "Schema has no columns", &response, crate::providers::llm::IssueOutcome::Stopped);
            return Err(LlmError::ParseError("Schema has no columns".to_string()));
        }

        debug!(columns = schema.columns.len(), "Schema planned");

        Ok(schema)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proposed_schema_deserialize() {
        let json = r#"{
            "columns": [
                {"name": "company_name", "type": "text", "description": "Company name", "required": true},
                {"name": "website", "type": "url", "description": "Website URL", "required": false},
                {"name": "employee_count", "type": "number", "description": "Number of employees", "required": false}
            ]
        }"#;

        let schema: ProposedSchema = serde_json::from_str(json).unwrap();
        assert_eq!(schema.columns.len(), 3);
        assert_eq!(schema.columns[0].name, "company_name");
        assert!(schema.columns[0].required);
    }

    #[test]
    fn test_proposed_schema_roundtrip() {
        let schema = ProposedSchema {
            columns: vec![
                SchemaColumn {
                    name: "name".to_string(),
                    col_type: "text".to_string(),
                    description: "Entity name".to_string(),
                    required: true,
                },
                SchemaColumn {
                    name: "value".to_string(),
                    col_type: "number".to_string(),
                    description: "A numeric value".to_string(),
                    required: false,
                },
            ],
        };

        let json = serde_json::to_string(&schema).unwrap();
        let parsed: ProposedSchema = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.columns.len(), 2);
    }

    #[test]
    fn columns_are_named_in_the_language_of_the_query() {
        let prompt = system_prompt("Russian");
        assert!(prompt.contains("in Russian"));
        assert!(!prompt.contains("snake_case"));
        assert!(prompt.contains("first column"));
    }
}
