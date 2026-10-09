use tracing::{debug, warn};

use crate::providers::llm::manager::LlmManager;
use crate::providers::llm::types::{ImageInput, Message};
use crate::providers::search::ImageSearchResult;

/// An image result with a relevance score assigned by the ranker.
#[derive(Debug, Clone)]
pub struct RankedImageResult {
    pub result: ImageSearchResult,
    pub relevance_score: f64,
}

/// Max images per LLM batch to avoid count mismatches.
const BATCH_SIZE: usize = 15;
/// Candidates per comparison request (each one is a picture in the request).
const COMPARE_BATCH: usize = 4;
/// Longest side of a candidate picture sent for comparison.
const COMPARE_SIDE: u32 = 384;

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
            if llm.spending_limit_reached() {
                break;
            }
            let items: Vec<String> = chunk
                .iter()
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

            let response = match llm.complete_for_stage("image_ranker", messages, true).await {
                Ok(response) => response,
                Err(error) if error.code() == "budget_limit" => break,
                Err(error) => return Err(format!("LLM ranking failed: {error}")),
            };

            let scores = Self::parse_scores(&response.content, chunk.len()).unwrap_or_else(|e| {
                llm.report_invalid_response("image_ranker", &format!("Invalid relevance scores: {e}. This image batch was skipped."), &response, crate::providers::llm::IssueOutcome::Skipped);
                vec![0.0; chunk.len()]
            });

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

    /// Re-scores the best candidates by how well they match the request and look like the
    /// `references` (attached pictures), with a model that sees images. The candidates come
    /// already ranked by text; at most `max_compared` of them are compared, a few per request.
    /// Candidates whose picture cannot be loaded keep their text score.
    pub async fn rank_with_reference(
        query: &str,
        ranked: Vec<RankedImageResult>,
        references: &[ImageInput],
        llm: &LlmManager,
        model: &str,
        min_relevance: f64,
        max_compared: usize,
    ) -> Vec<RankedImageResult> {
        let mut compared: Vec<RankedImageResult> = Vec::new();
        // Candidates beyond the limit are not compared and do not make the cut.
        let candidates: Vec<RankedImageResult> = ranked.into_iter().take(max_compared).collect();
        // Load the pictures first; the order of the batches follows the text ranking.
        let mut loaded: Vec<(RankedImageResult, Option<ImageInput>)> = Vec::new();
        for candidate in candidates {
            let picture = Self::load_picture(&candidate.result).await;
            loaded.push((candidate, picture));
        }
        let (with_picture, without): (Vec<_>, Vec<_>) = loaded.into_iter().partition(|(_, p)| p.is_some());
        compared.extend(without.into_iter().map(|(c, _)| c));
        for batch in with_picture.chunks(COMPARE_BATCH) {
            if llm.spending_limit_reached() {
                compared.extend(batch.iter().map(|(c, _)| c.clone()));
                continue;
            }
            let mut images: Vec<ImageInput> = references.to_vec();
            images.extend(batch.iter().filter_map(|(_, p)| p.clone()));
            let prompt = format!(
                "User request: \"{query}\"\n\n\
                 The first {refs} picture(s) are the reference picture(s) the user attached. The next {count} pictures are \
                 candidates found on the web, in order. Score each candidate 0.0-1.0 by how well it matches the request \
                 AND resembles the reference: same kind of object, model, style, colors and composition score high; \
                 a different object or style scores low.\n\n\
                 Respond with ONLY a JSON array of exactly {count} numbers, one per candidate.",
                refs = references.len(),
                count = batch.len(),
            );
            let messages = vec![
                Message::system("You compare pictures with a reference picture. Output ONLY a JSON array of float scores."),
                Message::user_with_images(prompt, images),
            ];
            let scores = match llm.complete_for_stage_with_model("image_compare", messages, model, true).await {
                Ok(response) => Self::parse_scores(&response.content, batch.len()).unwrap_or_else(|e| {
                    llm.report_invalid_response(
                        "image_compare",
                        &format!("Invalid similarity scores: {e}. These images keep their text scores."),
                        &response,
                        crate::providers::llm::IssueOutcome::Skipped,
                    );
                    batch.iter().map(|(c, _)| c.relevance_score).collect()
                }),
                Err(error) => {
                    warn!(error = %error, "Comparing with the reference failed; keeping text scores");
                    batch.iter().map(|(c, _)| c.relevance_score).collect()
                }
            };
            for ((candidate, _), score) in batch.iter().zip(scores) {
                compared.push(RankedImageResult { result: candidate.result.clone(), relevance_score: score });
            }
        }
        compared.retain(|r| r.relevance_score >= min_relevance);
        compared.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap_or(std::cmp::Ordering::Equal));
        compared
    }

    /// A small copy of a found picture (the thumbnail, or the full image) to show a model.
    async fn load_picture(result: &ImageSearchResult) -> Option<ImageInput> {
        for url in [&result.thumbnail_url, &result.image_url] {
            if url.is_empty() {
                continue;
            }
            let Ok((_, bytes)) = crate::commands::images::download_image(url, 8 * 1024 * 1024).await else { continue };
            let small = tokio::task::spawn_blocking(move || {
                let image = image::load_from_memory(&bytes).ok()?;
                let image = image.thumbnail(COMPARE_SIDE, COMPARE_SIDE);
                let mut out = std::io::Cursor::new(Vec::new());
                image::DynamicImage::ImageRgb8(image.to_rgb8()).write_to(&mut out, image::ImageFormat::Jpeg).ok()?;
                Some(out.into_inner())
            })
            .await
            .ok()
            .flatten();
            if let Some(bytes) = small {
                return Some(ImageInput::from_bytes("image/jpeg", &bytes));
            }
        }
        None
    }

    /// Parse a JSON array of f64 scores from LLM response.
    /// Reports invalid JSON so the caller can explain why a batch was rejected.
    fn parse_scores(response: &str, expected_count: usize) -> Result<Vec<f64>, serde_json::Error> {
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

        let scores = serde_json::from_str::<Vec<f64>>(json_str)?;
        if scores.len() == expected_count {
            return Ok(scores.into_iter().map(|s| s.clamp(0.0, 1.0)).collect());
        }
        // If count doesn't match exactly but close, try to use what we have
        if scores.len() >= expected_count {
            warn!(
                expected = expected_count,
                got = scores.len(),
                "LLM returned more scores than expected, truncating"
            );
            return Ok(scores.into_iter().take(expected_count).map(|s| s.clamp(0.0, 1.0)).collect());
        }
        // Fewer scores — pad remainder with 0.0 (reject)
        warn!(
            expected = expected_count,
            got = scores.len(),
            "LLM returned fewer scores than expected, padding with 0.0"
        );
        let mut padded: Vec<f64> = scores.into_iter().map(|s| s.clamp(0.0, 1.0)).collect();
        padded.resize(expected_count, 0.0);
        Ok(padded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scores_valid() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7, 0.3]", 3).unwrap();
        assert_eq!(scores, vec![0.9, 0.7, 0.3]);
    }

    #[test]
    fn test_parse_scores_with_text() {
        let scores = ImageRanker::parse_scores("Here are the scores: [0.8, 0.6, 0.4]", 3).unwrap();
        assert_eq!(scores, vec![0.8, 0.6, 0.4]);
    }

    #[test]
    fn test_parse_scores_fallback_rejects_all() {
        assert!(ImageRanker::parse_scores("invalid response", 3).is_err());
    }

    #[test]
    fn test_parse_scores_fewer_pads_with_zero() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7]", 3).unwrap();
        assert_eq!(scores, vec![0.9, 0.7, 0.0]);
    }

    #[test]
    fn test_parse_scores_more_truncates() {
        let scores = ImageRanker::parse_scores("[0.9, 0.7, 0.3, 0.5]", 3).unwrap();
        assert_eq!(scores, vec![0.9, 0.7, 0.3]);
    }

    #[test]
    fn test_parse_scores_clamp() {
        let scores = ImageRanker::parse_scores("[1.5, -0.3, 0.7]", 3).unwrap();
        assert_eq!(scores, vec![1.0, 0.0, 0.7]);
    }
}
