//! Spreadsheets (XLSX, XLS, ODS) and CSV: each sheet in blocks of rows with the header repeated,
//! so a fragment reads on its own and cites exact row numbers.

use std::io::Cursor;

use calamine::{Data, Reader};

use super::{decode_text, ParseError, MAX_TEXT_CHARS};
use crate::attachments::chunk::FRAGMENT_CHARS;
use crate::attachments::model::{AttachmentKind, Fragment, Locator, ParsedFile, SheetSummary};

pub fn parse_workbook(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let mut workbook = calamine::open_workbook_auto_from_rs(Cursor::new(bytes.to_vec())).map_err(|e| {
        let message = e.to_string();
        if message.to_lowercase().contains("password") || message.to_lowercase().contains("encrypt") {
            ParseError::Protected
        } else {
            ParseError::Damaged(message)
        }
    })?;
    let mut sheets = Vec::new();
    for name in workbook.sheet_names() {
        let range = match workbook.worksheet_range(&name) {
            Ok(range) => range,
            Err(_) => continue,
        };
        let first_row = range.start().map(|(row, _)| row).unwrap_or(0);
        let rows: Vec<(u32, Vec<String>)> = range
            .rows()
            .enumerate()
            .map(|(i, row)| (first_row + i as u32 + 1, row.iter().map(cell_text).collect()))
            .collect();
        sheets.push((name, rows));
    }
    Ok(build(sheets, mime_of(bytes)))
}

fn mime_of(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"PK") {
        if bytes.windows(46).any(|w| w == b"application/vnd.oasis.opendocument.spreadsheet") {
            "application/vnd.oasis.opendocument.spreadsheet"
        } else {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        }
    } else {
        "application/vnd.ms-excel"
    }
}

fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", *f as i64),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.as_datetime().map(|d| d.to_string()).unwrap_or_else(|| d.to_string()),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(_) => String::new(),
    }
}

pub fn parse_csv(bytes: &[u8], delimiter: Option<u8>) -> Result<ParsedFile, ParseError> {
    let text = decode_text(bytes);
    let delimiter = delimiter.unwrap_or_else(|| sniff_delimiter(&text));
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for (i, record) in reader.records().enumerate() {
        let record = record.map_err(|e| ParseError::Damaged(e.to_string()))?;
        rows.push((i as u32 + 1, record.iter().map(|c| c.trim().to_string()).collect()));
    }
    let mut parsed = build(vec![(String::new(), rows)], "text/csv");
    parsed.sheets.retain(|s| s.rows > 0);
    Ok(parsed)
}

/// The most frequent of `,` `;` and tab in the first lines.
fn sniff_delimiter(text: &str) -> u8 {
    let sample: String = text.lines().take(20).collect::<Vec<_>>().join("\n");
    [b',', b';', b'\t']
        .into_iter()
        .max_by_key(|d| sample.matches(*d as char).count())
        .unwrap_or(b',')
}

/// Turns sheets into fragments: "Columns: …" then "Row N: …" lines, at most one fragment's worth each.
fn build(sheets: Vec<(String, Vec<(u32, Vec<String>)>)>, mime: &'static str) -> ParsedFile {
    let mut summaries = Vec::new();
    let mut fragments = Vec::new();
    let mut total = 0usize;
    for (name, rows) in sheets {
        let rows: Vec<(u32, Vec<String>)> = rows
            .into_iter()
            .map(|(n, cells)| (n, trim_trailing(cells)))
            .filter(|(_, cells)| !cells.is_empty())
            .collect();
        let Some((header_row, header)) = rows.first().cloned() else { continue };
        summaries.push(SheetSummary { name: name.clone(), rows: rows.len() as u32, columns: header.clone() });
        let sheet = (!name.is_empty()).then(|| name.clone());
        let head = format!("{}Columns: {}", sheet.as_ref().map(|s| format!("Sheet: {s}\n")).unwrap_or_default(), header.join(" | "));
        let mut body = String::new();
        let mut range: Option<(u32, u32)> = None;
        let flush = |body: &mut String, range: &mut Option<(u32, u32)>, out: &mut Vec<Fragment>| {
            if let Some(rows) = range.take() {
                out.push(Fragment {
                    locator: Locator { sheet: sheet.clone(), rows: Some(rows), ..Locator::default() },
                    text: format!("{head}\n{}", body.trim_end()),
                });
                body.clear();
            }
        };
        for (number, cells) in rows.iter().filter(|(n, _)| *n != header_row) {
            let line = format!("Row {number}: {}\n", cells.join(" | "));
            if !body.is_empty() && head.len() + body.len() + line.len() > FRAGMENT_CHARS {
                flush(&mut body, &mut range, &mut fragments);
            }
            total += line.len();
            body.push_str(&line);
            range = Some((range.map_or(*number, |r| r.0), *number));
            if total > MAX_TEXT_CHARS {
                break;
            }
        }
        if range.is_none() {
            // A sheet with only a header still tells what it holds.
            range = Some((header_row, header_row));
        }
        flush(&mut body, &mut range, &mut fragments);
        if total > MAX_TEXT_CHARS {
            break;
        }
    }
    ParsedFile {
        kind: AttachmentKind::Spreadsheet,
        mime,
        page_count: None,
        sheets: summaries,
        fragments,
        scanned_pages: Vec::new(),
        headings: Vec::new(),
        image: None,
    }
}

fn trim_trailing(mut cells: Vec<String>) -> Vec<String> {
    while cells.last().is_some_and(|c| c.is_empty()) {
        cells.pop();
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;

    #[test]
    fn workbook_sheets_become_row_blocks_with_header() {
        let mut rows: Vec<Vec<String>> = vec![vec!["Product".into(), "Price".into()]];
        rows.extend((1..=300).map(|i| vec![format!("Item {i} with a fairly long descriptive name"), format!("{i}.5")]));
        let bytes = test_files::xlsx(&[("Prices", rows), ("Empty", vec![])]);
        let parsed = parse_workbook(&bytes).unwrap();
        assert_eq!(parsed.sheets, vec![SheetSummary { name: "Prices".into(), rows: 301, columns: vec!["Product".into(), "Price".into()] }]);
        assert!(parsed.fragments.len() > 1);
        let first = &parsed.fragments[0];
        assert_eq!(first.locator.sheet.as_deref(), Some("Prices"));
        assert_eq!(first.locator.rows.unwrap().0, 2);
        assert!(first.text.starts_with("Sheet: Prices\nColumns: Product | Price\nRow 2: Item 1"));
        let second = &parsed.fragments[1];
        assert!(second.text.starts_with("Sheet: Prices\nColumns: Product | Price\nRow "));
        assert_eq!(second.locator.rows.unwrap().0, first.locator.rows.unwrap().1 + 1);
        assert_eq!(parsed.fragments.last().unwrap().locator.rows.unwrap().1, 301);
    }

    #[test]
    fn csv_with_semicolons_and_windows_1251() {
        let (bytes, _, _) = encoding_rs::WINDOWS_1251.encode("Город;Население\nМосква;13000000\nКазань;1300000\n");
        let parsed = parse_csv(&bytes, None).unwrap();
        assert_eq!(parsed.sheets[0].columns, vec!["Город", "Население"]);
        assert_eq!(parsed.fragments[0].text, "Columns: Город | Население\nRow 2: Москва | 13000000\nRow 3: Казань | 1300000");
        assert_eq!(parsed.fragments[0].locator, Locator { rows: Some((2, 3)), ..Locator::default() });
    }
}
