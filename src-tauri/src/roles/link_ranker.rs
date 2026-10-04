use tracing::debug;

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;
use crate::roles::document_parser::ParsedDocument;

#[derive(Debug, Clone)]
pub struct PageCandidate {
    pub url: String,
    pub title: String,
    pub snippet: String,
    pub document: ParsedDocument,
}

#[derive(Debug, Clone)]
pub struct RankedLink {
    pub url: String,
    pub title: String,
    /// What the page contains, independent of the query.
    pub description: String,
    /// Why the page does or does not match the query.
    pub reason: String,
    pub relevance_score: f64,
}

/// LLM answer for one page.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct PageScore {
    #[serde(default)]
    pub relevance: f64,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub reason: String,
}

pub struct LinkRanker;

impl LinkRanker {
    /// Scores one fetched page against the query and describes it.
    /// An unusable model answer scores 0.0 and is reported as an LLM issue.
    pub async fn score(
        query: &str,
        candidate: &PageCandidate,
        llm: &LlmManager,
        max_text_chars: Option<usize>,
    ) -> Result<RankedLink, String> {
        let text = match max_text_chars {
            Some(limit) => crate::utils::text::truncate_chars(&candidate.document.text, limit),
            None => &candidate.document.text,
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
             Write two short texts in the same language as the query:\n\
             - \"description\": 1-2 sentences (max ~240 chars) saying what the page IS and \
             CONTAINS, as a neutral summary for a reader who has not opened it. Name the kind \
             of page (article, repository, documentation, forum thread, product page, list, \
             video) and its main topics. Do NOT mention the query, the user, relevance, or how \
             well it matches; never start with phrases like \"This page is exactly about\".\n\
             - \"reason\": one short phrase (max ~120 chars) explaining why the page does or \
             does not match the query.\n\n\
             Page URL: {url}\n\
             Page title: {title}\n\
             Page content:\n{text}\n\n\
             Respond with ONLY valid JSON: \
             {{\"relevance\": <float 0.0-1.0>, \"description\": \"<text>\", \"reason\": \"<text>\"}}. \
             No markdown, no explanation.",
            query = query,
            url = candidate.url,
            title = candidate.title,
            text = text,
        );

        let messages = vec![
            Message::system(
                "You are a strict web page relevance judge. Output ONLY a JSON object with \
                 a float \"relevance\" field (0.0-1.0) and string \"description\" and \
                 \"reason\" fields. No extra text.",
            ),
            Message::user(prompt),
        ];

        let response = llm
            .complete_for_stage("link_ranker", messages, true)
            .await
            .map_err(|e| format!("LLM scoring failed: {e}"))?;

        let score = Self::parse_score(&response.content).unwrap_or_else(|e| {
            llm.report_invalid_response(
                "link_ranker",
                &format!("Invalid relevance score: {e}. The page was scored as irrelevant."),
                &response,
                crate::providers::llm::IssueOutcome::Fallback,
            );
            PageScore { relevance: 0.0, description: String::new(), reason: String::new() }
        });

        debug!(
            url = %candidate.url,
            score = score.relevance,
            raw_response = %response.content.chars().take(200).collect::<String>(),
            "Link relevance score"
        );

        Ok(Self::ranked(candidate, score))
    }

    /// A link built from search data alone, used when no model is configured.
    pub fn unscored(candidate: &PageCandidate) -> RankedLink {
        Self::ranked(
            candidate,
            PageScore { relevance: 0.0, description: String::new(), reason: String::new() },
        )
    }

    fn ranked(candidate: &PageCandidate, score: PageScore) -> RankedLink {
        RankedLink {
            url: candidate.url.clone(),
            title: if candidate.title.trim().is_empty() {
                candidate.document.title.clone()
            } else {
                candidate.title.clone()
            },
            description: if score.description.trim().is_empty() {
                candidate.snippet.clone()
            } else {
                score.description.trim().to_string()
            },
            reason: score.reason.trim().to_string(),
            relevance_score: score.relevance,
        }
    }

    /// Parses `{ "relevance": f64, "description": String, "reason": String }` from the LLM response.
    pub fn parse_score(response: &str) -> Result<PageScore, serde_json::Error> {
        let trimmed = response.trim();
        let json_str = match (trimmed.find('{'), trimmed.rfind('}')) {
            (Some(start), Some(end)) if end > start => &trimmed[start..=end],
            _ => trimmed,
        };
        let mut parsed = serde_json::from_str::<PageScore>(json_str)?;
        parsed.relevance = parsed.relevance.clamp(0.0, 1.0);
        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_score_valid() {
        let score = LinkRanker::parse_score(
            "{\"relevance\": 0.9, \"description\": \"A great page\", \"reason\": \"Covers Tauri\"}",
        )
        .unwrap();
        assert_eq!(score.relevance, 0.9);
        assert_eq!(score.description, "A great page");
        assert_eq!(score.reason, "Covers Tauri");
    }

    #[test]
    fn test_parse_score_with_text_and_without_reason() {
        let score = LinkRanker::parse_score("Here: {\"relevance\": 0.5, \"description\": \"ok\"}").unwrap();
        assert_eq!(score.relevance, 0.5);
        assert_eq!(score.description, "ok");
        assert_eq!(score.reason, "");
    }

    #[test]
    fn test_parse_score_clamp() {
        let score = LinkRanker::parse_score("{\"relevance\": 1.7, \"description\": \"x\"}").unwrap();
        assert_eq!(score.relevance, 1.0);
    }

    #[test]
    fn test_parse_score_invalid_rejects() {
        assert!(LinkRanker::parse_score("not json").is_err());
    }

    #[test]
    fn test_truncate_on_char_boundary() {
        // '，' is 3 bytes each; a byte limit may land inside a char.
        let text = "aaa，，，".to_string();
        let limit = 4; // byte 4 is inside the first '，' (bytes 3..6)
        let mut end = limit;
        while end > 0 && !text.is_char_boundary(end) {
            end -= 1;
        }
        // Must not panic and must yield a valid UTF-8 slice.
        assert_eq!(&text[..end], "aaa");
    }
}
