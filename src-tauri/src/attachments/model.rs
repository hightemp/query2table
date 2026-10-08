use serde::{Deserialize, Serialize};

/// What kind of content a file holds; decides how it is parsed and shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentKind {
    Document,
    Spreadsheet,
    Text,
    Image,
}

impl AttachmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Spreadsheet => "spreadsheet",
            Self::Text => "text",
            Self::Image => "image",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "document" => Self::Document,
            "spreadsheet" => Self::Spreadsheet,
            "image" => Self::Image,
            _ => Self::Text,
        }
    }
}

/// Where a fragment sits in its file: a page, a sheet and rows, or a section.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Locator {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheet: Option<String>,
    /// First and last spreadsheet row (1-based, as numbered in the spreadsheet).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<(u32, u32)>,
    /// Nearest heading above the fragment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

impl Locator {
    pub fn page(page: u32) -> Self {
        Self { page: Some(page), ..Self::default() }
    }

    /// `attachment://<id>?page=3`, `attachment://<id>?sheet=Prices&rows=40-60`, `attachment://<id>?section=Intro`.
    pub fn to_url(&self, attachment_id: &str) -> String {
        let mut url = url::Url::parse(&format!("attachment://{attachment_id}")).expect("valid attachment URL");
        {
            let mut query = url.query_pairs_mut();
            if let Some(page) = self.page {
                query.append_pair("page", &page.to_string());
            }
            if let Some(sheet) = &self.sheet {
                query.append_pair("sheet", sheet);
            }
            if let Some((from, to)) = self.rows {
                query.append_pair("rows", &format!("{from}-{to}"));
            }
            if let Some(section) = &self.section {
                query.append_pair("section", section);
            }
        }
        let text = url.to_string();
        text.strip_suffix('?').map(str::to_string).unwrap_or(text)
    }

    /// "report.pdf, page 3", "prices.xlsx, sheet Prices, rows 40–60", "notes.md, section Intro".
    pub fn label(&self, file_name: &str) -> String {
        let mut parts = vec![file_name.to_string()];
        if let Some(page) = self.page {
            parts.push(format!("page {page}"));
        }
        if let Some(sheet) = &self.sheet {
            parts.push(format!("sheet {sheet}"));
        }
        if let Some((from, to)) = self.rows {
            parts.push(if from == to { format!("row {from}") } else { format!("rows {from}–{to}") });
        }
        if let Some(section) = &self.section {
            parts.push(format!("section {section}"));
        }
        parts.join(", ")
    }

    /// The attachment id and place of an `attachment://` URL.
    pub fn from_url(value: &str) -> Option<(String, Self)> {
        let url = url::Url::parse(value).ok()?;
        if url.scheme() != "attachment" {
            return None;
        }
        let id = url.host_str()?.to_string();
        let mut locator = Self::default();
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "page" => locator.page = value.parse().ok(),
                "sheet" => locator.sheet = Some(value.into_owned()),
                "rows" => {
                    locator.rows = value
                        .split_once('-')
                        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
                }
                "section" => locator.section = Some(value.into_owned()),
                _ => {}
            }
        }
        Some((id, locator))
    }
}

/// A piece of a file's text small enough to give a model, with its place in the file.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub locator: Locator,
    pub text: String,
}

/// A downscaled image ready to send to a model, plus a thumbnail for the UI.
#[derive(Debug, Clone)]
pub struct PreparedImage {
    pub media_type: &'static str,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub thumbnail_jpeg: Vec<u8>,
}

/// Everything learned from a file when it is attached.
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub kind: AttachmentKind,
    pub mime: &'static str,
    pub page_count: Option<u32>,
    pub sheets: Vec<SheetSummary>,
    pub fragments: Vec<Fragment>,
    /// Pages with no text layer (scans), to be read by a vision model.
    pub scanned_pages: Vec<u32>,
    /// Headings found in the document, in order.
    pub headings: Vec<String>,
    pub image: Option<PreparedImage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetSummary {
    pub name: String,
    pub rows: u32,
    pub columns: Vec<String>,
}

/// Readiness of an attached file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentStatus {
    Ready,
    /// Some or all pages are scans that need a model that sees images.
    NeedsVision,
}

/// An attached file as the interface sees it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttachmentInfo {
    pub id: String,
    pub file_name: String,
    pub kind: AttachmentKind,
    pub mime: String,
    pub size: i64,
    pub status: AttachmentStatus,
    pub page_count: Option<i64>,
    pub sheet_count: Option<i64>,
    pub char_count: i64,
    pub scanned_pages: i64,
    /// Small preview of images, as a data URL.
    pub thumbnail: Option<String>,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_in_files_round_trip_as_urls() {
        let page = Locator::page(12);
        assert_eq!(page.to_url("abc"), "attachment://abc?page=12");
        let rows = Locator { sheet: Some("Цены и сроки".into()), rows: Some((40, 60)), ..Locator::default() };
        let url = rows.to_url("abc");
        assert_eq!(Locator::from_url(&url), Some(("abc".to_string(), rows.clone())));
        assert_eq!(Locator::default().to_url("abc"), "attachment://abc");
        assert_eq!(Locator::from_url("https://example.com/?page=1"), None);
        assert_eq!(page.label("report.pdf"), "report.pdf, page 12");
        assert_eq!(rows.label("p.xlsx"), "p.xlsx, sheet Цены и сроки, rows 40–60");
    }
}
