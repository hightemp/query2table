use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::providers::llm::{LlmManager, LlmError, Message};
use super::query_interpreter::QueryIntent;
use super::schema_planner::ProposedSchema;

/// A single planned search query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedSearch {
    pub query_text: String,
    pub language: String,
    pub geo_target: Option<String>,
    pub priority: u8,
}

/// A complete search plan with multiple queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPlan {
    pub queries: Vec<PlannedSearch>,
}

const SYSTEM_PROMPT: &str = r#"You are a search strategist for a research data collection tool. Given information about the research goal and the table schema, generate effective search queries.

Generate 5-15 search queries that will find the target entities. Each query should have:
- query_text: the actual search string to use
- language: ISO 639-1 language code (e.g., "en", "de")
- geo_target: country/region code if relevant (e.g., "US", "DE"), or null
- priority: 1 (high) to 3 (low)

Strategy:
1. Start with direct queries for the entity type with constraints
2. Add queries targeting specific attributes or lists
3. Include queries for industry directories, comparison sites, "top N" lists
4. Vary phrasing to maximize coverage
5. If multiple languages are needed, add translated queries

Respond with valid JSON: {"queries": [...]}. No markdown, no explanation."#;

/// Plans search queries based on intent and schema.
pub struct SearchPlanner;

impl SearchPlanner {
    pub async fn plan(
        intent: &QueryIntent,
        schema: &ProposedSchema,
        llm: &LlmManager,
    ) -> Result<SearchPlan, LlmError> {
        debug!(entity_type = %intent.entity_type, "Planning search queries");

        let column_names: Vec<&str> = schema.columns.iter().map(|c| c.name.as_str()).collect();

        let messages = vec![
            Message::system(SYSTEM_PROMPT),
            Message::user(format!(
                "Research goal: Find {} with these attributes: {}\n\nConstraints: {}\nGeo: {}\nLanguages: {}\n\nTable columns: {}",
                intent.entity_type,
                intent.attributes.join(", "),
                intent.constraints.join(", "),
                intent.geo.as_deref().unwrap_or("none"),
                intent.languages.join(", "),
                column_names.join(", "),
            )),
        ];

        let response = llm.complete(messages, true).await?;

        let plan: SearchPlan = serde_json::from_str(&response.content)
            .map_err(|e| LlmError::ParseError(format!(
                "Failed to parse search plan: {}. Response: {}",
                e, response.content
            )))?;

        if plan.queries.is_empty() {
            return Err(LlmError::ParseError("Search plan has no queries".to_string()));
        }

        debug!(query_count = plan.queries.len(), "Search plan created");

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_plan_deserialize() {
        let json = r#"{
            "queries": [
                {"query_text": "top tech companies Germany 2024", "language": "en", "geo_target": "DE", "priority": 1},
                {"query_text": "deutsche tech unternehmen liste", "language": "de", "geo_target": "DE", "priority": 2}
            ]
        }"#;

        let plan: SearchPlan = serde_json::from_str(json).unwrap();
        assert_eq!(plan.queries.len(), 2);
        assert_eq!(plan.queries[0].priority, 1);
    }

    #[test]
    fn test_planned_search_roundtrip() {
        let search = PlannedSearch {
            query_text: "AI startups San Francisco".to_string(),
            language: "en".to_string(),
            geo_target: Some("US".to_string()),
            priority: 1,
        };

        let json = serde_json::to_string(&search).unwrap();
        let parsed: PlannedSearch = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.query_text, "AI startups San Francisco");
    }
}
