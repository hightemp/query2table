use tracing::{debug, warn};

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;
use crate::roles::document_parser::ParsedDocument;

/// A page to be scored for relevance, with its source metadata.
#[derive(Debug, Clone)]
pub struct PageCandidate {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub document: ParsedDocument,
}

/// A link result with a relevance score and LLM-generated description.
#[derive(Debug, Clone)]
pub struct RankedLink {
    pub url: String,
    pub title: String,
    pub description: String,
    pub relevance_score: f64,
}

/// Uses LLM to score page content relevance to the original query and
/// generate a concise description, keeping only the most relevant pages.
pub struct LinkRanker;

impl LinkRanker {
    /// Rank pages by content relevance to the query.
    /// Scores each page from its fetched content (one LLM call per page),
    /// filters out pages below `min_relevance`, and sorts by score descending.
    pub async fn rank(
        query: &str,
        candidates: Vec<PageCandidate>,
        llm: &LlmManager,
        min_relevance: f64,
        max_text_chars: Option<usize>,
    ) -> Result<Vec<RankedLink>, String> {
        if candidates.is_empty() {
            return Ok(vec![]);
        }

        let mut ranked: Vec<RankedLink> = Vec::new();

        for candidate in &candidates {
            match Self::score_one(query, candidate, llm, max_text_chars).await {
                Ok((score, description)) => {
                    let description = if description.trim().is_empty() {
                        candidate.snippet.clone()
                    } else {
                        description
                    };
                    let title = if candidate.title.trim().is_empty() {
                        candidate.document.title.clone()
                    } else {
                        candidate.title.clone()
                    };
                    ranked.push(RankedLink {
                        url: candidate.url.clone(),
                        title,
                        description,
                        relevance_score: score,
                    });
                }
                Err(e) => {
                    warn!(url = %candidate.url, error = %e, "Link relevance scoring failed, skipping page");
                }
            }
        }

        // Filter and sort
        ranked.retain(|r| r.relevance_score >= min_relevance);
        ranked.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!(query = %query, passed = ranked.len(), "Link ranking complete");

        Ok(ranked)
    }

    /// Score a single page and produce a relevance score + concise description.
    async fn score_one(
        query: &str,
        candidate: &PageCandidate,
        llm: &LlmManager,
        max_text_chars: Option<usize>,
    ) -> Result<(f64, String), String> {
        let text = match max_text_chars {
            Some(limit) if candidate.document.text.len() > limit => {
                &candidate.document.text[..limit]
            }
            _ => &candidate.document.text,
        };

        let prompt = format!(
            "User query: \"{query}\"\n\n\
             Evaluate STRICTLY how relevant the following web page is to the user query, \
             based on its ACTUAL CONTENT.\n\n\
             Scoring rules:\n\
             - 0.9-1.0 = directly and fully answers/matches the query\n\
             - 0.7-0.8 = highly relevant, covers most of what the query asks\n\
             - 0.4-0.6 = generally on-topic but missing key aspects\n\
             - 0.1-0.3 = barely related\n\
             - 0.0 = completely irrelevant\n\n\
             Also write a concise 1-2 sentence description (max ~280 chars) summarizing \
             what this page offers WITH RESPECT TO the query. Write the description in the \
             same language as the query.\n\n\
             Page URL: {url}\n\
             Page title: {title}\n\
             Page content:\n{text}\n\n\
             Respond with ONLY valid JSON: {{\"relevance\": <float 0.0-1.0>, \"description\": \"<text>\"}}. \
             No markdown, no explanation.",
            query = query,
            url = candidate.url,
            title = candidate.title,
            text = text,
        );

        let messages = vec![
            Message::system(
                "You are a strict web page relevance judge. Output ONLY a JSON object with \
                 a float \"relevance\" field (0.0-1.0) and a string \"description\" field. \
                 No extra text.",
            ),
            Message::user(prompt),
        ];

        let response = llm
            .complete(messages, true)
            .await
            .map_err(|e| format!("LLM scoring failed: {e}"))?;

        let (score, description) = Self::parse_score(&response.content);

        debug!(
            url = %candidate.url,
            score,
            raw_response = %response.content.chars().take(200).collect::<String>(),
            "Link relevance score"
        );

        Ok((score, description))
    }

    /// Parse a JSON object `{ "relevance": f64, "description": String }` from the LLM response.
    /// On failure, returns score 0.0 (reject) and an empty description.
    fn parse_score(response: &str) -> (f64, String) {
        let trimmed = response.trim();
        let json_str = if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                &trimmed[start..=end]
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        #[derive(serde::Deserialize)]
        struct ScoreResponse {
            #[serde(default)]
            relevance: f64,
            #[serde(default)]
            description: String,
        }

        match serde_json::from_str::<ScoreResponse>(json_str) {
            Ok(parsed) => (parsed.relevance.clamp(0.0, 1.0), parsed.description),
            Err(_) => {
                warn!(
                    response = %trimmed.chars().take(100).collect::<String>(),
                    "Failed to parse link relevance score, rejecting page"
                );
                (0.0, String::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_score_valid() {
        let (score, desc) = LinkRanker::parse_score("{\"relevance\": 0.9, \"description\": \"A great page\"}");
        assert_eq!(score, 0.9);
        assert_eq!(desc, "A great page");
    }

    #[test]
    fn test_parse_score_with_text() {
        let (score, desc) = LinkRanker::parse_score("Here: {\"relevance\": 0.5, \"description\": \"ok\"}");
        assert_eq!(score, 0.5);
        assert_eq!(desc, "ok");
    }

    #[test]
    fn test_parse_score_clamp() {
        let (score, _) = LinkRanker::parse_score("{\"relevance\": 1.7, \"description\": \"x\"}");
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_parse_score_invalid_rejects() {
        let (score, desc) = LinkRanker::parse_score("not json");
        assert_eq!(score, 0.0);
        assert_eq!(desc, "");
    }
}
