use tracing::debug;

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;
use crate::providers::search::ImageSearchResult;

/// An image result with a relevance score assigned by the ranker.
#[derive(Debug, Clone)]
pub struct RankedImageResult {
    pub result: ImageSearchResult,
    pub relevance_score: f64,
}

/// Uses LLM to rank/filter image search results for relevance to the original query.
pub struct ImageRanker;

impl ImageRanker {
    /// Rank image results by relevance to the query.
    /// Returns results sorted by relevance score (highest first), filtering out irrelevant ones.
    pub async fn rank(
        query: &str,
        results: Vec<ImageSearchResult>,
        llm: &LlmManager,
        min_relevance: f64,
    ) -> Result<Vec<RankedImageResult>, String> {
        if results.is_empty() {
            return Ok(vec![]);
        }

        // Build a list of titles for LLM evaluation
        let titles: Vec<String> = results.iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {}", i + 1, r.title))
            .collect();

        let prompt = format!(
            "You are evaluating image search results for the query: \"{}\"\n\n\
             For each image below, rate its relevance from 0.0 to 1.0 where:\n\
             - 1.0 = highly relevant to the query\n\
             - 0.5 = somewhat relevant\n\
             - 0.0 = not relevant at all\n\n\
             Images:\n{}\n\n\
             Respond with ONLY a JSON array of numbers (relevance scores), one per image.\n\
             Example: [0.9, 0.7, 0.2, 0.8]",
            query,
            titles.join("\n")
        );

        let messages = vec![
            Message::system("You are an image relevance evaluator. Respond only with a JSON array of numbers."),
            Message::user(prompt),
        ];

        let response = llm.complete(messages, true).await
            .map_err(|e| format!("LLM ranking failed: {e}"))?;

        // Parse scores from LLM response
        let scores = Self::parse_scores(&response.content, results.len());

        debug!(
            query = %query,
            total = results.len(),
            scores_parsed = scores.len(),
            "Image ranking complete"
        );

        let mut ranked: Vec<RankedImageResult> = results.into_iter()
            .zip(scores.into_iter())
            .map(|(result, score)| RankedImageResult {
                result,
                relevance_score: score,
            })
            .filter(|r| r.relevance_score >= min_relevance)
            .collect();

        ranked.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(ranked)
    }

    /// Parse a JSON array of f64 scores from LLM response.
    /// Falls back to 0.5 for any unparseable values.
    fn parse_scores(response: &str, expected_count: usize) -> Vec<f64> {
        // Try to find a JSON array in the response
        let trimmed = response.trim();
        let json_str = if let Some(start) = trimmed.find('[') {
            if let Some(end) = trimmed.rfind(']') {
                &trimmed[start..=end]
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        if let Ok(scores) = serde_json::from_str::<Vec<f64>>(json_str) {
            if scores.len() == expected_count {
                return scores.into_iter().map(|s| s.clamp(0.0, 1.0)).collect();
            }
        }

        // Fallback: assign 0.5 to all
        vec![0.5; expected_count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scores_valid() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7, 0.3]", 3);
        assert_eq!(scores, vec![0.9, 0.7, 0.3]);
    }

    #[test]
    fn test_parse_scores_with_text() {
        let scores = ImageRanker::parse_scores("Here are the scores: [0.8, 0.6, 0.4]", 3);
        assert_eq!(scores, vec![0.8, 0.6, 0.4]);
    }

    #[test]
    fn test_parse_scores_fallback() {
        let scores = ImageRanker::parse_scores("invalid response", 3);
        assert_eq!(scores, vec![0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_parse_scores_wrong_count() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7]", 3);
        assert_eq!(scores, vec![0.5, 0.5, 0.5]);
    }

    #[test]
    fn test_parse_scores_clamp() {
        let scores = ImageRanker::parse_scores("[1.5, -0.3, 0.7]", 3);
        assert_eq!(scores, vec![1.0, 0.0, 0.7]);
    }
}
