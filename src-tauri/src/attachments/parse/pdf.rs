//! PDF: text by page; pages without a text layer are noted as scans for a vision model.

use super::{ParseError, MAX_TEXT_CHARS};
use crate::attachments::chunk::{fragments, FRAGMENT_CHARS};
use crate::attachments::model::{AttachmentKind, Locator, ParsedFile};

/// A page with fewer letters than this has no usable text layer.
const MIN_PAGE_LETTERS: usize = 15;

pub fn parse(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    if !bytes.starts_with(b"%PDF") {
        return Err(ParseError::Damaged("not a PDF".into()));
    }
    let pages = extract_pages(bytes)?;
    let mut blocks = Vec::new();
    let mut scanned_pages = Vec::new();
    let mut total = 0;
    for (index, text) in pages.iter().enumerate() {
        let number = index as u32 + 1;
        let text = clean(text);
        if text.chars().filter(|c| c.is_alphanumeric()).count() < MIN_PAGE_LETTERS {
            scanned_pages.push(number);
            continue;
        }
        total += text.len();
        if total > MAX_TEXT_CHARS {
            break;
        }
        blocks.push((Locator::page(number), text));
    }
    Ok(ParsedFile {
        kind: AttachmentKind::Document,
        mime: "application/pdf",
        page_count: Some(pages.len() as u32),
        sheets: Vec::new(),
        fragments: fragments(blocks, FRAGMENT_CHARS),
        scanned_pages,
        headings: Vec::new(),
        image: None,
    })
}

fn extract_pages(bytes: &[u8]) -> Result<Vec<String>, ParseError> {
    // pdf-extract panics on some malformed files; a damaged file must not take the app down.
    let owned = bytes.to_vec();
    match std::panic::catch_unwind(move || pdf_extract::extract_text_from_mem_by_pages(&owned)) {
        Ok(Ok(pages)) => Ok(pages),
        Ok(Err(error)) => {
            let message = error.to_string();
            if message.to_lowercase().contains("encrypt") || message.to_lowercase().contains("password") {
                Err(ParseError::Protected)
            } else {
                Err(ParseError::Damaged(message))
            }
        }
        Err(_) => Err(ParseError::Damaged("the PDF reader stopped".into())),
    }
}

/// Joins words broken across lines and collapses runs of blank lines.
fn clean(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank = 0;
    for line in text.lines().map(str::trim_end) {
        if line.trim().is_empty() {
            blank += 1;
            continue;
        }
        if !out.is_empty() {
            out.push_str(if blank > 0 { "\n\n" } else { "\n" });
        }
        blank = 0;
        out.push_str(line);
    }
    out.replace("-\n", "")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;

    #[test]
    fn text_is_kept_by_page_and_empty_pages_are_scans() {
        let bytes = test_files::pdf(&["Annual report of Acme Corporation for 2025", "", "Revenue grew to 12 million dollars"]);
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed.page_count, Some(3));
        assert_eq!(parsed.scanned_pages, vec![2]);
        assert_eq!(parsed.fragments.len(), 2);
        assert_eq!(parsed.fragments[1].locator, Locator::page(3));
        assert!(parsed.fragments[1].text.contains("Revenue grew"));
    }

    #[test]
    fn damaged_files_are_reported() {
        assert!(matches!(parse(b"%PDF-1.5 garbage"), Err(ParseError::Damaged(_))));
        assert!(matches!(parse(b"hello"), Err(ParseError::Damaged(_))));
    }
}
