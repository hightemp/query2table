use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::providers::llm::{LlmManager, LlmError, Message};

/// Structured intent parsed from a natural-language query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIntent {
    pub entity_type: String,
    pub attributes: Vec<String>,
    pub constraints: Vec<String>,
    pub geo: Option<String>,
    pub languages: Vec<String>,
    #[serde(default)]
    pub original_query: String,
}

const SYSTEM_PROMPT: &str = r#"You are a query interpreter for a research tool that converts natural-language questions into structured data collection plans.

Given a user query, extract:
- entity_type: The main type of entity being searched (e.g., "companies", "universities", "restaurants")
- attributes: List of specific attributes/fields the user wants to know about each entity (e.g., ["name", "website", "founding_year", "employee_count"])
- constraints: Any filtering criteria (e.g., ["located in Germany", "founded after 2010", "has more than 100 employees"])
- geo: Geographic focus if any (e.g., "Germany", "San Francisco Bay Area"), or null
- languages: Languages to search in (default: ["en"]). If the query mentions a specific country/region, include that language too.

Respond with valid JSON only. No markdown, no explanation."#;

/// Interprets a natural-language query into a structured QueryIntent.
pub struct QueryInterpreter;

impl QueryInterpreter {
    /// Interpret a natural-language query into structured intent.
    pub async fn interpret(
        query: &str,
        llm: &LlmManager,
    ) -> Result<QueryIntent, LlmError> {
        debug!(query = %query, "Interpreting query");

        let messages = vec![
            Message::system(SYSTEM_PROMPT),
            Message::user(format!(
                "Parse this research query into structured JSON:\n\n\"{}\"",
                query
            )),
        ];

        let response = llm.complete(messages, true).await?;

        let mut intent: QueryIntent = serde_json::from_str(&response.content)
            .map_err(|e| LlmError::ParseError(format!(
                "Failed to parse query intent: {}. Response: {}",
                e, response.content
            )))?;

        intent.original_query = query.to_string();

        if intent.languages.is_empty() {
            intent.languages.push("en".to_string());
        }

        debug!(entity_type = %intent.entity_type, attributes = ?intent.attributes, "Query interpreted");

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_intent_serialize() {
        let intent = QueryIntent {
            entity_type: "companies".to_string(),
            attributes: vec!["name".to_string(), "website".to_string()],
            constraints: vec!["in Germany".to_string()],
            geo: Some("Germany".to_string()),
            languages: vec!["en".to_string(), "de".to_string()],
            original_query: "Find tech companies in Germany".to_string(),
        };

        let json = serde_json::to_string(&intent).unwrap();
        let parsed: QueryIntent = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_type, "companies");
        assert_eq!(parsed.attributes.len(), 2);
    }

    #[test]
    fn test_query_intent_deserialize_from_llm() {
        let llm_output = r#"{
            "entity_type": "universities",
            "attributes": ["name", "location", "ranking", "student_count"],
            "constraints": ["top 50 in Europe"],
            "geo": "Europe",
            "languages": ["en"]
        }"#;

        let mut intent: QueryIntent = serde_json::from_str(llm_output).unwrap();
        intent.original_query = "test".to_string();
        assert_eq!(intent.entity_type, "universities");
        assert_eq!(intent.attributes.len(), 4);
        assert_eq!(intent.geo, Some("Europe".to_string()));
    }
}
