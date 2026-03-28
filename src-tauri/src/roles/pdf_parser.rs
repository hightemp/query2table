use tracing::{debug, warn};

use super::document_parser::ParsedDocument;

/// Extracts text content from a PDF byte buffer.
pub struct PdfParser;

impl PdfParser {
    /// Parse PDF bytes and extract clean text.
    /// Returns a `ParsedDocument` with extracted title and text.
    /// If `max_chars` is `Some(n)`, the extracted text is truncated to `n` characters.
    pub fn parse(bytes: &[u8], url: &str, max_chars: Option<usize>) -> ParsedDocument {
        match Self::extract_text(bytes, max_chars) {
            Ok(text) => {
                let title = Self::extract_title(&text, url);
                let cleaned = Self::clean_text(&text);
                debug!(url = %url, title = %title, text_len = cleaned.len(), "Parsed PDF document");
                ParsedDocument {
                    title,
                    text: cleaned,
                    url: url.to_string(),
                }
            }
            Err(e) => {
                warn!(url = %url, error = %e, "Failed to extract text from PDF");
                ParsedDocument {
                    title: String::new(),
                    text: String::new(),
                    url: url.to_string(),
                }
            }
        }
    }

    fn extract_text(bytes: &[u8], max_chars: Option<usize>) -> Result<String, String> {
        let text = pdf_extract::extract_text_from_mem(bytes)
            .map_err(|e| format!("PDF extraction error: {}", e))?;

        if let Some(max) = max_chars {
            if text.len() > max {
                return Ok(text[..max].to_string());
            }
        }
        Ok(text)
    }

    /// Try to derive a title from the first line of text or the URL filename.
    fn extract_title(text: &str, url: &str) -> String {
        // Use first non-empty line as title candidate
        if let Some(first_line) = text.lines().find(|l| !l.trim().is_empty()) {
            let trimmed = first_line.trim();
            if trimmed.len() <= 200 {
                return trimmed.to_string();
            }
        }

        // Fallback: extract filename from URL
        if let Some(filename) = url.rsplit('/').next() {
            let name = filename.split('?').next().unwrap_or(filename);
            if !name.is_empty() {
                return name.to_string();
            }
        }

        String::new()
    }

    fn clean_text(text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut prev_newline_count = 0;

        for ch in text.chars() {
            if ch == '\n' || ch == '\r' {
                prev_newline_count += 1;
                if prev_newline_count <= 2 {
                    result.push('\n');
                }
            } else if ch.is_whitespace() {
                if !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
                prev_newline_count = 0;
            } else {
                result.push(ch);
                prev_newline_count = 0;
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_text() {
        let title = PdfParser::extract_title("Introduction to Rust\nChapter 1", "https://example.com/doc.pdf");
        assert_eq!(title, "Introduction to Rust");
    }

    #[test]
    fn test_extract_title_from_url_fallback() {
        let title = PdfParser::extract_title("", "https://example.com/report-2024.pdf");
        assert_eq!(title, "report-2024.pdf");
    }

    #[test]
    fn test_extract_title_empty() {
        let title = PdfParser::extract_title("", "https://example.com/");
        assert_eq!(title, "");
    }

    #[test]
    fn test_clean_text() {
        let text = "Hello   World\n\n\n\nParagraph two";
        let cleaned = PdfParser::clean_text(text);
        assert_eq!(cleaned, "Hello World\n\nParagraph two");
    }

    #[test]
    fn test_clean_text_preserves_structure() {
        let text = "Title\n\nFirst paragraph.\nSecond line.\n\nAnother section.";
        let cleaned = PdfParser::clean_text(text);
        assert!(cleaned.contains("Title\n\nFirst paragraph."));
        assert!(cleaned.contains("Another section."));
    }

    #[test]
    fn test_parse_invalid_pdf_returns_empty() {
        let invalid_bytes = b"This is not a PDF file at all";
        let doc = PdfParser::parse(invalid_bytes, "https://example.com/fake.pdf", None);
        assert!(doc.text.is_empty());
    }

    #[test]
    fn test_parse_real_minimal_pdf() {
        // Build a minimal valid PDF with lopdf (re-exported by pdf_extract) to test extraction round-trip
        use pdf_extract::dictionary;
        use pdf_extract::{Document, Object, Stream};
        use pdf_extract::content::{Content, Operation};

        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();

        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        let resources_id = doc.add_object(dictionary! {
            "Font" => dictionary! {
                "F1" => font_id,
            },
        });

        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![100.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal("Hello PDF World")]),
                Operation::new("ET", vec![]),
            ],
        };

        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));

        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });

        let pages = dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page_id.into()],
            "Count" => 1,
        };

        doc.objects.insert(pages_id, Object::Dictionary(pages));

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });

        doc.trailer.set("Root", catalog_id);

        // Serialize to bytes
        let mut buf = Vec::new();
        doc.save_to(&mut buf).unwrap();

        // Now test PdfParser
        let parsed = PdfParser::parse(&buf, "https://example.com/test.pdf", None);
        assert!(
            parsed.text.contains("Hello PDF World"),
            "Expected 'Hello PDF World' in parsed text, got: {:?}",
            parsed.text
        );
    }
}
