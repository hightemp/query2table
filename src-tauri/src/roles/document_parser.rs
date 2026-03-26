use scraper::{Html, Selector};
use tracing::debug;

/// Cleaned document content extracted from HTML.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub text: String,
    pub url: String,
}

/// Parses HTML into clean text suitable for LLM extraction.
pub struct DocumentParser;

impl DocumentParser {
    /// Parse HTML content and extract clean text.
    pub fn parse(html: &str, url: &str) -> ParsedDocument {
        let document = Html::parse_document(html);

        let title = Self::extract_title(&document);
        let text = Self::extract_text(&document);

        debug!(url = %url, title = %title, text_len = text.len(), "Parsed document");

        ParsedDocument { title, text, url: url.to_string() }
    }

    fn extract_title(document: &Html) -> String {
        if let Ok(sel) = Selector::parse("title") {
            if let Some(el) = document.select(&sel).next() {
                let t = el.text().collect::<String>().trim().to_string();
                if !t.is_empty() {
                    return t;
                }
            }
        }
        // Fallback to h1
        if let Ok(sel) = Selector::parse("h1") {
            if let Some(el) = document.select(&sel).next() {
                return el.text().collect::<String>().trim().to_string();
            }
        }
        String::new()
    }

    fn extract_text(document: &Html) -> String {
        // Remove noise elements first, then extract from main content areas
        let main_text = Self::extract_main_content(document);
        if !main_text.is_empty() {
            return Self::clean_whitespace(&main_text);
        }

        // Fallback: extract from body with noise removal
        let body_text = Self::extract_body_text(document);
        Self::clean_whitespace(&body_text)
    }

    fn extract_main_content(document: &Html) -> String {
        // Try article, main, [role="main"] selectors in order
        let selectors = ["article", "main", "[role=\"main\"]", ".content", "#content"];

        for sel_str in &selectors {
            if let Ok(sel) = Selector::parse(sel_str) {
                let texts: Vec<String> = document
                    .select(&sel)
                    .map(|el| Self::element_visible_text(&el))
                    .collect();

                if !texts.is_empty() {
                    let combined = texts.join("\n\n");
                    if combined.len() > 100 {
                        return combined;
                    }
                }
            }
        }

        String::new()
    }

    fn extract_body_text(document: &Html) -> String {
        if let Ok(sel) = Selector::parse("body") {
            if let Some(body) = document.select(&sel).next() {
                return Self::element_visible_text(&body);
            }
        }
        // Last resort: all text
        document.root_element().text().collect::<Vec<_>>().join(" ")
    }

    fn element_visible_text(element: &scraper::ElementRef) -> String {
        let noise_tags: std::collections::HashSet<&str> = [
            "script", "style", "nav", "header", "footer", "aside",
            "noscript", "svg", "iframe", "form",
        ].into_iter().collect();

        let mut parts = Vec::new();

        // Use select to grab all text not inside noise tags
        // We iterate over children recursively, skipping subtrees rooted at noise elements
        Self::collect_text_recursive(element, &noise_tags, &mut parts);

        parts.join(" ")
    }

    fn collect_text_recursive(
        node: &scraper::ElementRef,
        noise_tags: &std::collections::HashSet<&str>,
        parts: &mut Vec<String>,
    ) {
        for child in node.children() {
            if let Some(el) = child.value().as_element() {
                if noise_tags.contains(el.name()) {
                    continue; // Skip entire subtree
                }
                // Recurse into non-noise child elements
                if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                    Self::collect_text_recursive(&child_ref, noise_tags, parts);
                }
            } else if let Some(text) = child.value().as_text() {
                let t = text.trim();
                if !t.is_empty() {
                    parts.push(t.to_string());
                }
            }
        }
    }

    fn clean_whitespace(text: &str) -> String {
        // Collapse multiple whitespace/newlines
        let mut result = String::with_capacity(text.len());
        let mut prev_space = false;

        for ch in text.chars() {
            if ch.is_whitespace() {
                if !prev_space {
                    result.push(' ');
                    prev_space = true;
                }
            } else {
                result.push(ch);
                prev_space = false;
            }
        }

        result.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_html() {
        let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Hello World</h1>
            <p>This is a test paragraph with some content.</p>
        </body>
        </html>"#;

        let doc = DocumentParser::parse(html, "https://example.com");
        assert_eq!(doc.title, "Test Page");
        assert!(doc.text.contains("Hello World"));
        assert!(doc.text.contains("test paragraph"));
    }

    #[test]
    fn test_removes_script_and_style() {
        let html = r#"
        <html>
        <body>
            <script>var x = 1;</script>
            <style>.foo { color: red; }</style>
            <p>Visible content</p>
        </body>
        </html>"#;

        let doc = DocumentParser::parse(html, "https://example.com");
        assert!(doc.text.contains("Visible content"));
        assert!(!doc.text.contains("var x"));
        assert!(!doc.text.contains("color: red"));
    }

    #[test]
    fn test_removes_nav_footer() {
        let html = r#"
        <html>
        <body>
            <nav><a href="/">Home</a><a href="/about">About</a></nav>
            <article>
                <h1>Main Article</h1>
                <p>Article body text that is long enough to be selected as main content. 
                It needs to be over 100 characters so the main content selector picks it up properly.</p>
            </article>
            <footer>Copyright 2024</footer>
        </body>
        </html>"#;

        let doc = DocumentParser::parse(html, "https://example.com");
        assert!(doc.text.contains("Main Article"));
        assert!(doc.text.contains("Article body text"));
        // nav/footer removed from article extraction
        assert!(!doc.text.contains("Copyright 2024"));
    }

    #[test]
    fn test_extracts_title_from_h1_fallback() {
        let html = r#"
        <html>
        <body>
            <h1>Fallback Title</h1>
            <p>Some text</p>
        </body>
        </html>"#;

        let doc = DocumentParser::parse(html, "https://example.com");
        assert_eq!(doc.title, "Fallback Title");
    }

    #[test]
    fn test_clean_whitespace() {
        let text = "  Hello   World  \n\n\n  Test  ";
        let cleaned = DocumentParser::clean_whitespace(text);
        assert_eq!(cleaned, "Hello World Test");
    }

    #[test]
    fn test_empty_html() {
        let doc = DocumentParser::parse("", "https://example.com");
        assert!(doc.title.is_empty());
    }
}
