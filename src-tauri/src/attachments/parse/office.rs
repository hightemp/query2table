//! Word (DOCX) and LibreOffice (ODT) documents: paragraphs, headings and tables.

use std::io::{Cursor, Read};

use quick_xml::events::Event;
use quick_xml::Reader;

use super::{ParseError, MAX_TEXT_CHARS};
use crate::attachments::chunk::{fragments, FRAGMENT_CHARS};
use crate::attachments::model::{AttachmentKind, Locator, ParsedFile};

/// Largest XML part read from an archive; guards against zip bombs.
const MAX_PART_BYTES: u64 = 200 * 1024 * 1024;

pub(crate) fn read_part(bytes: &[u8], name: &str) -> Result<Vec<u8>, ParseError> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| ParseError::Damaged(e.to_string()))?;
    if archive.by_name("EncryptionInfo").is_ok() || archive.by_name("EncryptedPackage").is_ok() {
        return Err(ParseError::Protected);
    }
    let part = archive.by_name(name).map_err(|_| ParseError::Damaged(format!("{name} is missing")))?;
    let mut out = Vec::new();
    part.take(MAX_PART_BYTES + 1).read_to_end(&mut out).map_err(|e| ParseError::Damaged(e.to_string()))?;
    if out.len() as u64 > MAX_PART_BYTES {
        return Err(ParseError::TooLarge);
    }
    Ok(out)
}

/// Tag names of one document format.
struct Dialect {
    paragraph: &'static [u8],
    heading: &'static [u8],
    text: &'static [u8],
    tab: &'static [u8],
    line_break: &'static [u8],
    row: &'static [u8],
    cell: &'static [u8],
}

const DOCX: Dialect = Dialect {
    paragraph: b"w:p",
    heading: b"",
    text: b"w:t",
    tab: b"w:tab",
    line_break: b"w:br",
    row: b"w:tr",
    cell: b"w:tc",
};

const ODT: Dialect = Dialect {
    paragraph: b"text:p",
    heading: b"text:h",
    text: b"",
    tab: b"text:tab",
    line_break: b"text:line-break",
    row: b"table:table-row",
    cell: b"table:table-cell",
};

pub fn parse_docx(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let xml = read_part(bytes, "word/document.xml")?;
    walk(&xml, &DOCX)
}

pub fn parse_odt(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let xml = read_part(bytes, "content.xml")?;
    walk(&xml, &ODT)
}

/// Collects paragraphs in order, starting a new section at each heading; table rows become
/// "cell | cell" lines.
fn walk(xml: &[u8], dialect: &Dialect) -> Result<ParsedFile, ParseError> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();

    let mut blocks: Vec<(Locator, String)> = Vec::new();
    let mut headings = Vec::new();
    let mut locator = Locator::default();
    let mut paragraph = String::new();
    let mut is_heading = false;
    let mut in_text = dialect.text.is_empty();
    let mut row: Option<Vec<String>> = None;
    let mut cell_depth = 0usize;
    let mut total = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let name = name.as_ref();
                if name == dialect.paragraph || (!dialect.heading.is_empty() && name == dialect.heading) {
                    paragraph.clear();
                    is_heading = name == dialect.heading;
                } else if name == dialect.text {
                    in_text = true;
                } else if name == dialect.row {
                    row = Some(Vec::new());
                } else if name == dialect.cell {
                    cell_depth += 1;
                    if let Some(row) = row.as_mut() {
                        row.push(String::new());
                    }
                } else if name == b"w:pStyle" {
                    is_heading |= docx_heading_style(&e);
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                let name = name.as_ref();
                if name == dialect.tab {
                    paragraph.push('\t');
                } else if name == dialect.line_break {
                    paragraph.push('\n');
                } else if name == b"w:pStyle" {
                    is_heading |= docx_heading_style(&e);
                } else if name == b"text:s" {
                    paragraph.push(' ');
                }
            }
            Ok(Event::Text(e)) if in_text => {
                let text = e.decode().map_err(|err| ParseError::Damaged(err.to_string()))?;
                paragraph.push_str(&quick_xml::escape::unescape(&text).unwrap_or(text.clone()));
            }
            Ok(Event::GeneralRef(e)) if in_text => {
                if let Ok(Some(ch)) = e.resolve_char_ref() {
                    paragraph.push(ch);
                } else {
                    let name = e.decode().unwrap_or_default();
                    paragraph.push_str(match name.as_ref() {
                        "amp" => "&",
                        "lt" => "<",
                        "gt" => ">",
                        "quot" => "\"",
                        "apos" => "'",
                        _ => "",
                    });
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let name = name.as_ref();
                if name == dialect.text {
                    in_text = false;
                } else if name == dialect.paragraph || (!dialect.heading.is_empty() && name == dialect.heading) {
                    let text = paragraph.trim().to_string();
                    paragraph.clear();
                    if text.is_empty() {
                        continue;
                    }
                    if cell_depth > 0 {
                        if let Some(cell) = row.as_mut().and_then(|r| r.last_mut()) {
                            if !cell.is_empty() {
                                cell.push(' ');
                            }
                            cell.push_str(&text);
                        }
                    } else if is_heading {
                        headings.push(text.clone());
                        locator = Locator { section: Some(text.clone()), ..Locator::default() };
                        blocks.push((locator.clone(), text));
                    } else {
                        total += text.len();
                        blocks.push((locator.clone(), text));
                    }
                    is_heading = false;
                } else if name == dialect.cell {
                    cell_depth = cell_depth.saturating_sub(1);
                } else if name == dialect.row {
                    if let Some(cells) = row.take() {
                        let line = cells.iter().map(|c| c.trim()).collect::<Vec<_>>().join(" | ");
                        if !line.replace('|', "").trim().is_empty() {
                            total += line.len();
                            blocks.push((locator.clone(), line));
                        }
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ParseError::Damaged(e.to_string())),
            _ => {}
        }
        if total > MAX_TEXT_CHARS {
            break;
        }
        buf.clear();
    }

    // Table rows of one table read better as one block: join consecutive rows line by line.
    let mut merged: Vec<(Locator, String)> = Vec::new();
    for (loc, text) in blocks {
        match merged.last_mut() {
            Some((last_loc, last)) if *last_loc == loc && last.contains(" | ") && text.contains(" | ") => {
                last.push('\n');
                last.push_str(&text);
            }
            _ => merged.push((loc, text)),
        }
    }

    Ok(ParsedFile {
        kind: AttachmentKind::Document,
        mime: if dialect.paragraph == DOCX.paragraph {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        } else {
            "application/vnd.oasis.opendocument.text"
        },
        page_count: None,
        sheets: Vec::new(),
        fragments: fragments(merged, FRAGMENT_CHARS),
        scanned_pages: Vec::new(),
        headings,
        image: None,
    })
}

/// Word marks headings with styles such as "Heading1", "Title" or localized "Заголовок1".
fn docx_heading_style(e: &quick_xml::events::BytesStart) -> bool {
    e.attributes().flatten().any(|attr| {
        attr.key.as_ref() == b"w:val" && {
            let value = String::from_utf8_lossy(&attr.value).to_lowercase();
            value.starts_with("heading") || value == "title" || value.starts_with("заголовок")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;

    #[test]
    fn docx_paragraphs_headings_and_tables() {
        let bytes = test_files::docx(&[
            test_files::DocPart::Paragraph("Company overview & goals"),
            test_files::DocPart::Heading("Pricing"),
            test_files::DocPart::Paragraph("Plans are billed monthly."),
            test_files::DocPart::Table(&[&["Plan", "Price"], &["Pro", "$10"], &["Team", "$25"]]),
        ]);
        let parsed = parse_docx(&bytes).unwrap();
        assert_eq!(parsed.headings, vec!["Pricing"]);
        assert_eq!(parsed.fragments[0].text, "Company overview & goals");
        let pricing = &parsed.fragments[1];
        assert_eq!(pricing.locator.section.as_deref(), Some("Pricing"));
        assert!(pricing.text.contains("Plans are billed monthly."));
        assert!(pricing.text.contains("Plan | Price\nPro | $10\nTeam | $25"), "{}", pricing.text);
    }

    #[test]
    fn odt_paragraphs_headings_and_tables() {
        let bytes = test_files::odt(&[
            test_files::DocPart::Heading("Введение"),
            test_files::DocPart::Paragraph("Отчёт о продажах"),
            test_files::DocPart::Table(&[&["Город", "Выручка"], &["Москва", "120"]]),
        ]);
        let parsed = parse_odt(&bytes).unwrap();
        assert_eq!(parsed.headings, vec!["Введение"]);
        let text = &parsed.fragments[0].text;
        assert!(text.contains("Отчёт о продажах") && text.contains("Город | Выручка\nМосква | 120"), "{text}");
    }

    #[test]
    fn broken_archives_are_reported() {
        assert!(matches!(parse_docx(b"PK\x03\x04broken"), Err(ParseError::Damaged(_))));
        let not_word = test_files::zip(&[("other.xml", "<x/>")]);
        assert!(matches!(parse_docx(&not_word), Err(ParseError::Damaged(_))));
    }
}
