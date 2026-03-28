use tracing::{debug, warn};

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::Message;
use crate::providers::search::ImageSearchResult;

/// An image result with a relevance score assigned by the ranker.
#[derive(Debug, Clone)]
pub struct RankedImageResult {
    pub result: ImageSearchResult,
    pub relevance_score: f64,
}

/// Max images per LLM batch to avoid count mismatches.
const BATCH_SIZE: usize = 15;

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

        // Process in batches to improve LLM accuracy
        let mut all_ranked: Vec<RankedImageResult> = Vec::new();

        for chunk in results.chunks(BATCH_SIZE) {
            let items: Vec<String> = chunk.iter()
                .enumerate()
                .map(|(i, r)| {
                    let mut desc = format!("{}. title: \"{}\"", i + 1, r.title);
                    if !r.source_url.is_empty() {
                        desc.push_str(&format!(" | source: {}", r.source_url));
                    }
                    desc
                })
                .collect();

            let prompt = format!(
                "User query: \"{query}\"\n\n\
                 You must evaluate STRICTLY whether each image matches the EXACT request.\n\
                 The query may specify: object type, color, style, brand, model, quantity, context.\n\
                 ALL criteria in the query must be satisfied for a high score.\n\n\
                 Scoring rules:\n\
                 - 0.9-1.0 = matches ALL query criteria exactly (correct object, color, style, etc.)\n\
                 - 0.7-0.8 = matches most criteria but one minor detail differs\n\
                 - 0.4-0.6 = matches the general topic but missing key criteria (wrong color, wrong model, etc.)\n\
                 - 0.1-0.3 = barely related, mostly wrong\n\
                 - 0.0 = completely irrelevant\n\n\
                 BE STRICT. If the query asks for a green car, a yellow car gets 0.3 max.\n\
                 If the query asks for a specific model, a different model gets 0.3 max.\n\n\
                 Images ({count} total):\n{items}\n\n\
                 Respond with ONLY a JSON array of exactly {count} numbers.\n\
                 Example for 3 images: [0.9, 0.3, 0.7]",
                count = chunk.len(),
                items = items.join("\n")
            );

            let messages = vec![
                Message::system(
                    "You are a strict image relevance judge. Output ONLY a JSON array of float scores. \
                     No text, no explanation. The array length MUST equal the number of images."
                ),
                Message::user(prompt),
            ];

            let response = llm.complete(messages, true).await
                .map_err(|e| format!("LLM ranking failed: {e}"))?;

            let scores = Self::parse_scores(&response.content, chunk.len());

            debug!(
                query = %query,
                batch_size = chunk.len(),
                scores = ?scores,
                raw_response = %response.content.chars().take(200).collect::<String>(),
                "Image ranking batch"
            );

            for (result, score) in chunk.iter().zip(scores.into_iter()) {
                all_ranked.push(RankedImageResult {
                    result: result.clone(),
                    relevance_score: score,
                });
            }
        }

        // Filter and sort
        all_ranked.retain(|r| r.relevance_score >= min_relevance);
        all_ranked.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));

        debug!(
            query = %query,
            passed = all_ranked.len(),
            "Image ranking complete"
        );

        Ok(all_ranked)
    }

    /// Parse a JSON array of f64 scores from LLM response.
    /// On failure, assigns 0.0 (reject) instead of passing everything through.
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
            // If count doesn't match exactly but close, try to use what we have
            if scores.len() >= expected_count {
                warn!(
                    expected = expected_count,
                    got = scores.len(),
                    "LLM returned more scores than expected, truncating"
                );
                return scores.into_iter().take(expected_count).map(|s| s.clamp(0.0, 1.0)).collect();
            }
            // Fewer scores — pad remainder with 0.0 (reject)
            warn!(
                expected = expected_count,
                got = scores.len(),
                "LLM returned fewer scores than expected, padding with 0.0"
            );
            let mut padded: Vec<f64> = scores.into_iter().map(|s| s.clamp(0.0, 1.0)).collect();
            padded.resize(expected_count, 0.0);
            return padded;
        }

        // Total parse failure — reject all rather than accept all
        warn!(
            response = %trimmed.chars().take(100).collect::<String>(),
            "Failed to parse LLM ranking scores, rejecting batch"
        );
        vec![0.0; expected_count]
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
    fn test_parse_scores_fallback_rejects_all() {
        let scores = ImageRanker::parse_scores("invalid response", 3);
        assert_eq!(scores, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_parse_scores_fewer_pads_with_zero() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7]", 3);
        assert_eq!(scores, vec![0.9, 0.7, 0.0]);
    }

    #[test]
    fn test_parse_scores_more_truncates() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7, 0.3, 0.5]", 3);
        assert_eq!(scores, vec![0.9, 0.7, 0.3]);
    }

    #[test]
    fn test_parse_scores_clamp() {
        let scores = ImageRanker::parse_scores("[1.5, -0.3, 0.7]", 3);
        assert_eq!(scores, vec![1.0, 0.0, 0.7]);
    }
}
