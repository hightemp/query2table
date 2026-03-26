use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::providers::llm::{LlmManager, LlmError, Message};
use super::search_planner::PlannedSearch;

/// Expanded queries in multiple languages and phrasings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedQueries {
    pub queries: Vec<PlannedSearch>,
}

const SYSTEM_PROMPT: &str = r#"You are a multilingual search query expander. Given a set of search queries and target languages, generate additional query variations.

For each input query:
1. Translate to each target language (if not already in that language)
2. Create synonym/rephrasing variants (e.g., "companies" → "firms", "businesses", "startups")
3. Add useful search modifiers (e.g., "list of", "top", "directory", "database")

Each expanded query must have:
- query_text: the search string
- language: ISO 639-1 code
- geo_target: country code or null
- priority: 1-3 (translations get same priority as source, variants get +1)

Keep the total to 2-4 expansions per input query. Avoid duplicates.

Respond with valid JSON: {"queries": [...]}. No markdown, no explanation."#;

/// Expands search queries with multilingual variants and synonyms.
pub struct QueryExpander;

impl QueryExpander {
    pub async fn expand(
        queries: &[PlannedSearch],
        languages: &[String],
        llm: &LlmManager,
    ) -> Result<ExpandedQueries, LlmError> {
        debug!(input_count = queries.len(), languages = ?languages, "Expanding queries");

        let queries_json = serde_json::to_string(queries)
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let messages = vec![
            Message::system(SYSTEM_PROMPT),
            Message::user(format!(
                "Expand these search queries into additional languages and phrasings.\n\nTarget languages: {}\n\nInput queries:\n{}",
                languages.join(", "),
                queries_json,
            )),
        ];

        let response = llm.complete(messages, true).await?;

        let expanded: ExpandedQueries = serde_json::from_str(&response.content)
            .map_err(|e| LlmError::ParseError(format!(
                "Failed to parse expanded queries: {}. Response: {}",
                e, response.content
            )))?;

        debug!(expanded_count = expanded.queries.len(), "Queries expanded");

        Ok(expanded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expanded_queries_deserialize() {
        let json = r#"{
            "queries": [
                {"query_text": "top tech companies Germany", "language": "en", "geo_target": "DE", "priority": 1},
                {"query_text": "die besten Tech-Unternehmen Deutschland", "language": "de", "geo_target": "DE", "priority": 1},
                {"query_text": "list of German technology firms", "language": "en", "geo_target": "DE", "priority": 2}
            ]
        }"#;

        let expanded: ExpandedQueries = serde_json::from_str(json).unwrap();
        assert_eq!(expanded.queries.len(), 3);
    }
}
