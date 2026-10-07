//! Plain text, Markdown, JSON and HTML files.

use super::{decode_text, ParseError, MAX_TEXT_CHARS};
use crate::attachments::chunk::{fragments, FRAGMENT_CHARS};
use crate::attachments::model::{AttachmentKind, Locator, ParsedFile};

fn text_file(mime: &'static str, blocks: Vec<(Locator, String)>, headings: Vec<String>) -> ParsedFile {
    ParsedFile {
        kind: AttachmentKind::Text,
        mime,
        page_count: None,
        sheets: Vec::new(),
        fragments: fragments(blocks, FRAGMENT_CHARS),
        scanned_pages: Vec::new(),
        headings,
        image: None,
    }
}

fn limited(text: String) -> String {
    if text.chars().count() > MAX_TEXT_CHARS { text.chars().take(MAX_TEXT_CHARS).collect() } else { text }
}

pub fn parse_plain(bytes: &[u8], mime: &'static str) -> Result<ParsedFile, ParseError> {
    let text = limited(decode_text(bytes)).replace("\r\n", "\n");
    Ok(text_file(mime, vec![(Locator::default(), text)], Vec::new()))
}

/// Markdown keeps its headings as sections, so fragments can be cited by heading.
pub fn parse_markdown(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let text = limited(decode_text(bytes)).replace("\r\n", "\n");
    let mut blocks = Vec::new();
    let mut headings = Vec::new();
    let mut locator = Locator::default();
    let mut current = String::new();
    let mut in_code = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        let heading = (!in_code && line.starts_with('#'))
            .then(|| line.trim_start_matches('#'))
            .filter(|rest| rest.starts_with(' '))
            .map(|rest| rest.trim().to_string());
        if let Some(heading) = heading.filter(|h| !h.is_empty()) {
            blocks.push((locator.clone(), std::mem::take(&mut current)));
            headings.push(heading.clone());
            locator = Locator { section: Some(heading), ..Locator::default() };
        }
        current.push_str(line);
        current.push('\n');
    }
    blocks.push((locator, current));
    Ok(text_file("text/markdown", blocks, headings))
}

/// Saved web pages: the readable text, like pages fetched during a run.
pub fn parse_html(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let html = decode_text(bytes);
    let converter = htmd::HtmlToMarkdown::builder()
        .skip_tags(vec!["script", "style", "noscript", "template", "svg", "head"])
        .build();
    let markdown = match converter.convert(&html) {
        Ok(markdown) if !markdown.trim().is_empty() => markdown,
        _ => crate::roles::document_parser::DocumentParser::parse(&html, "").text,
    };
    let mut parsed = parse_markdown(markdown.as_bytes())?;
    parsed.mime = "text/html";
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_sections_follow_headings() {
        let parsed = parse_markdown(b"Preface text\n\n# Prices\nCheap plan: $5\n\n```\n# not a heading\n```\n## Support\nEmail us").unwrap();
        assert_eq!(parsed.headings, vec!["Prices", "Support"]);
        let sections: Vec<_> = parsed.fragments.iter().map(|f| f.locator.section.clone()).collect();
        assert_eq!(sections, vec![None, Some("Prices".into()), Some("Support".into())]);
        assert!(parsed.fragments[1].text.contains("# not a heading"));
    }

    #[test]
    fn html_becomes_readable_text() {
        let parsed = parse_html(b"<html><head><title>T</title><script>var x=1</script></head><body><h1>Plans</h1><p>Pro costs <b>$10</b>.</p></body></html>").unwrap();
        let text: String = parsed.fragments.iter().map(|f| f.text.as_str()).collect();
        assert!(text.contains("Pro costs") && text.contains("$10"));
        assert!(!text.contains("var x"));
    }
}
