//! Turns an attached file into text fragments (or a prepared image) with their places in the file.

mod image;
pub use image::prepare as prepare_image;
mod office;
mod pdf;
mod sheet;
mod text;

use super::model::{AttachmentKind, ParsedFile};

/// Why a file could not be attached; shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("unsupported")]
    Unsupported,
    #[error("empty")]
    Empty,
    #[error("damaged: {0}")]
    Damaged(String),
    #[error("protected")]
    Protected,
    #[error("too large")]
    TooLarge,
}

impl ParseError {
    /// Stable code for the interface's translations.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unsupported => "unsupported",
            Self::Empty => "empty",
            Self::Damaged(_) => "damaged",
            Self::Protected => "protected",
            Self::TooLarge => "tooLarge",
        }
    }
}

/// File extensions that can be attached, as offered in the file dialog.
pub const EXTENSIONS: &[&str] = &[
    "pdf", "docx", "odt", "xlsx", "xlsm", "xls", "ods", "csv", "tsv", "txt", "md", "markdown", "json", "html", "htm",
    "png", "jpg", "jpeg", "webp", "gif",
];

/// Most characters of text kept from one file (about 2.5 million: a long book).
pub const MAX_TEXT_CHARS: usize = 2_500_000;

fn extension(file_name: &str) -> String {
    file_name.rsplit_once('.').map(|(_, ext)| ext.to_ascii_lowercase()).unwrap_or_default()
}

/// Parses a file by its extension. CPU-heavy: call from a blocking task.
pub fn parse(file_name: &str, bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    if bytes.is_empty() {
        return Err(ParseError::Empty);
    }
    let parsed = match extension(file_name).as_str() {
        "pdf" => pdf::parse(bytes)?,
        "docx" => office::parse_docx(bytes)?,
        "odt" => office::parse_odt(bytes)?,
        "xlsx" | "xlsm" | "xls" | "ods" => sheet::parse_workbook(bytes)?,
        "csv" => sheet::parse_csv(bytes, None)?,
        "tsv" => sheet::parse_csv(bytes, Some(b'\t'))?,
        "txt" => text::parse_plain(bytes, "text/plain")?,
        "md" | "markdown" => text::parse_markdown(bytes)?,
        "json" => text::parse_plain(bytes, "application/json")?,
        "html" | "htm" => text::parse_html(bytes)?,
        "png" | "jpg" | "jpeg" | "webp" | "gif" => image::parse(bytes)?,
        _ => return Err(ParseError::Unsupported),
    };
    let has_content = parsed.kind == AttachmentKind::Image || !parsed.fragments.is_empty() || !parsed.scanned_pages.is_empty();
    if has_content { Ok(parsed) } else { Err(ParseError::Empty) }
}

/// Decodes text that is UTF-8 (with or without BOM) or, failing that, a legacy Windows code page.
pub(crate) fn decode_text(bytes: &[u8]) -> String {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_string();
    }
    if bytes.starts_with(b"\xFF\xFE") || bytes.starts_with(b"\xFE\xFF") {
        let (text, _, _) = encoding_rs::UTF_16LE.decode(bytes);
        return text.into_owned();
    }
    // Cyrillic text without a declared encoding is most often Windows-1251.
    let (text, _, _) = encoding_rs::WINDOWS_1251.decode(bytes);
    text.into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_and_empty_files() {
        assert_eq!(parse("tool.exe", b"MZ").unwrap_err(), ParseError::Unsupported);
        assert_eq!(parse("notes.txt", b"").unwrap_err(), ParseError::Empty);
        assert_eq!(parse("notes.txt", b"   \n ").unwrap_err(), ParseError::Empty);
    }

    #[test]
    fn decodes_utf8_and_windows_1251() {
        assert_eq!(decode_text("\u{feff}Привет".as_bytes()), "Привет");
        let (cp1251, _, _) = encoding_rs::WINDOWS_1251.encode("Привет, мир");
        assert_eq!(decode_text(&cp1251), "Привет, мир");
    }
}
