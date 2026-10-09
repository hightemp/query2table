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
    /// ISO 639-1 code of the language the query is written in; the table speaks it.
    #[serde(default)]
    pub query_language: String,
    /// What the attached files hold, for the roles planning the table; never from the model.
    #[serde(skip)]
    pub files_context: Option<String>,
}

/// English name of a language for prompts ("ru" → "Russian"); unknown codes stay as they are.
pub fn language_name(code: &str) -> String {
    match code {
        "en" => "English", "ru" => "Russian", "de" => "German", "fr" => "French", "es" => "Spanish", "it" => "Italian",
        "pt" => "Portuguese", "uk" => "Ukrainian", "be" => "Belarusian", "kk" => "Kazakh", "pl" => "Polish", "cs" => "Czech",
        "nl" => "Dutch", "sv" => "Swedish", "fi" => "Finnish", "tr" => "Turkish", "ar" => "Arabic", "he" => "Hebrew",
        "zh" => "Chinese", "ja" => "Japanese", "ko" => "Korean", "hi" => "Hindi", "vi" => "Vietnamese", "id" => "Indonesian",
        other => other,
    }
    .to_string()
}

/// A guess from the writing system when the model does not say: Cyrillic → Russian, and so on.
fn guess_language(text: &str) -> &'static str {
    let count = |range: std::ops::RangeInclusive<char>| text.chars().filter(|c| range.contains(c)).count();
    let letters = text.chars().filter(|c| c.is_alphabetic()).count().max(1);
    let scripts = [
        ('\u{0400}'..='\u{04FF}', "ru"),
        ('\u{0600}'..='\u{06FF}', "ar"),
        ('\u{0590}'..='\u{05FF}', "he"),
        ('\u{3040}'..='\u{30FF}', "ja"),
        ('\u{AC00}'..='\u{D7AF}', "ko"),
        ('\u{4E00}'..='\u{9FFF}', "zh"),
    ];
    scripts.into_iter().find(|(range, _)| count(range.clone()) * 3 >= letters).map(|(_, code)| code).unwrap_or("en")
}

impl QueryIntent {
    /// Makes sure the query's own language is known and searched first.
    pub fn settle_languages(&mut self, query: &str) {
        let code = self.query_language.trim().to_lowercase();
        self.query_language = if code.len() == 2 && code.chars().all(|c| c.is_ascii_lowercase()) { code } else { guess_language(query).to_string() };
        let mut languages = vec![self.query_language.clone()];
        for language in &self.languages {
            let language = language.trim().to_lowercase();
            if !language.is_empty() && !languages.contains(&language) {
                languages.push(language);
            }
        }
        self.languages = languages;
    }

    /// The language the table is written in, by name, for prompts.
    pub fn language(&self) -> String {
        language_name(if self.query_language.is_empty() { "en" } else { &self.query_language })
    }

    /// The files part of a planning prompt, or nothing without files.
    pub fn files_note(&self, max_chars: usize) -> String {
        match &self.files_context {
            Some(context) => format!("\n\n{}", crate::utils::text::truncate_chars(context, max_chars)),
            None => String::new(),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"You are a query interpreter for a research tool that converts natural-language questions into structured data collection plans.

Given a user query, extract:
- entity_type: The main type of entity being searched (e.g., "companies", "universities", "restaurants")
- attributes: List of specific attributes/fields the user wants to know about each entity (e.g., ["name", "website", "founding_year", "employee_count"])
- constraints: Any filtering criteria (e.g., ["located in Germany", "founded after 2010", "has more than 100 employees"])
- geo: Geographic focus if any (e.g., "Germany", "San Francisco Bay Area"), or null
- query_language: ISO 639-1 code of the language the query itself is written in (e.g. "ru" for a query in Russian)
- languages: Languages to search in. Always include the query's language; add the language of a country or region the query targets, and "en" when good sources are likely in English.

Write entity_type, attributes and constraints in the language of the query.

Respond with valid JSON only. No markdown, no explanation."#;

/// Interprets a natural-language query into a structured QueryIntent.
pub struct QueryInterpreter;

impl QueryInterpreter {
    /// Interpret a natural-language query into structured intent.
    pub async fn interpret(
        query: &str,
        llm: &LlmManager,
    ) -> Result<QueryIntent, LlmError> {
        Self::interpret_with_files(query, None, llm).await
    }

    /// Interpret a query that comes with attached files; `files` describes them and holds the
    /// passages matching the query.
    pub async fn interpret_with_files(
        query: &str,
        files: Option<&str>,
        llm: &LlmManager,
    ) -> Result<QueryIntent, LlmError> {
        debug!(query = %query, "Interpreting query");

        let mut request = format!("Parse this research query into structured JSON:\n\n\"{}\"", query);
        if let Some(files) = files {
            request.push_str("\n\nThe query refers to these attached files; take the entities and attributes from them when it asks about them:\n\n");
            request.push_str(files);
        }
        let messages = vec![Message::system(SYSTEM_PROMPT), Message::user(request)];

        let response = llm.complete_for_stage("interpreter", messages, true).await?;

        let mut intent: QueryIntent = serde_json::from_str(&response.content)
            .map_err(|e| {
                let message = format!("Failed to parse query intent: {e}");
                llm.report_invalid_response("interpreter", &message, &response, crate::providers::llm::IssueOutcome::Stopped);
                LlmError::ParseError(message)
            })?;

        intent.original_query = query.to_string();
        intent.settle_languages(query);
        intent.files_context = files.map(str::to_string);


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
            query_language: "en".to_string(),
            files_context: None,
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

    fn intent(languages: &[&str], query_language: &str) -> QueryIntent {
        QueryIntent {
            entity_type: "channels".into(),
            attributes: vec![],
            constraints: vec![],
            geo: None,
            languages: languages.iter().map(|l| l.to_string()).collect(),
            original_query: String::new(),
            query_language: query_language.into(),
            files_context: None,
        }
    }

    #[test]
    fn the_query_language_comes_first_among_search_languages() {
        let mut found = intent(&["en", "de"], "ru");
        found.settle_languages("Найди немецкие компании");
        assert_eq!((found.query_language.as_str(), found.languages.clone()), ("ru", vec!["ru".to_string(), "en".into(), "de".into()]));

        // Without an answer from the model, the script of the query decides.
        let mut guessed = intent(&[], "");
        guessed.settle_languages("Найди ютуб каналы про роботов");
        assert_eq!((guessed.query_language.as_str(), guessed.languages.clone()), ("ru", vec!["ru".to_string()]));
        let mut english = intent(&[], "");
        english.settle_languages("Find robot channels");
        assert_eq!(english.languages, vec!["en".to_string()]);
        assert_eq!(language_name("ru"), "Russian");
        assert_eq!(language_name("xx"), "xx");
    }
}
