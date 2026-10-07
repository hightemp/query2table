//! Reads images with a model that sees them: transcribes scanned pages and describes pictures.

use crate::providers::llm::{ImageInput, LlmError, LlmManager, Message};

/// Asks a vision model about images; `model` may differ from the main model.
pub struct VisionReader<'a> {
    pub manager: &'a LlmManager,
    pub model: String,
}

const TRANSCRIBE: &str = "You read scanned document pages. Transcribe all text on the page exactly, in its original language, top to bottom. \
Keep headings, lists and paragraphs on separate lines. Write tables as rows with cells separated by \" | \". \
Describe charts or photos in one short bracketed line, e.g. [Chart: sales by month]. \
Return only the page content, with no comments. If the page has no text, return [No text].";

const DESCRIBE: &str = "You describe images for someone who cannot see them. Describe what the image shows: objects, people, place, style, \
colors, and its purpose if clear (photo, diagram, screenshot, chart, product, document). \
Copy any visible text exactly. For charts and tables give the values you can read. \
Be specific and factual, at most 200 words, no introduction.";

impl VisionReader<'_> {
    /// The text of a scanned page.
    pub async fn transcribe_page(&self, image: ImageInput) -> Result<String, LlmError> {
        let messages = vec![Message::system(TRANSCRIBE), Message::user_with_images("Transcribe this page.", vec![image])];
        let reply = self.manager.complete_for_stage_with_model("read_scan", messages, &self.model, false).await?;
        Ok(clean(&reply.content))
    }

    /// A text description of a picture, for models that cannot see it and for search.
    pub async fn describe_image(&self, image: ImageInput) -> Result<String, LlmError> {
        let messages = vec![Message::system(DESCRIBE), Message::user_with_images("Describe this image.", vec![image])];
        let reply = self.manager.complete_for_stage_with_model("describe_image", messages, &self.model, false).await?;
        Ok(clean(&reply.content))
    }
}

/// Drops Markdown fences some models wrap plain answers in, and the empty-page marker.
fn clean(text: &str) -> String {
    let text = text.trim();
    let text = text
        .strip_prefix("```")
        .map(|rest| rest.split_once('\n').map(|(_, body)| body).unwrap_or(rest))
        .and_then(|body| body.strip_suffix("```"))
        .unwrap_or(text)
        .trim();
    if text.eq_ignore_ascii_case("[no text]") { String::new() } else { text.to_string() }
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn answers_lose_fences_and_empty_markers() {
        assert_eq!(clean("```text\nInvoice 42\nTotal: 10\n```"), "Invoice 42\nTotal: 10");
        assert_eq!(clean("  [No text] "), "");
        assert_eq!(clean("Plain"), "Plain");
    }
}
