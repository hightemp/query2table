//! Builds small files of every supported format for tests.

use std::io::{Cursor, Write};

pub enum DocPart<'a> {
    Heading(&'a str),
    Paragraph(&'a str),
    Table(&'a [&'a [&'a str]]),
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn zip(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, content) in entries {
        writer.start_file(*name, zip::write::SimpleFileOptions::default()).unwrap();
        writer.write_all(content.as_bytes()).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

pub fn docx(parts: &[DocPart]) -> Vec<u8> {
    let mut body = String::new();
    for part in parts {
        match part {
            DocPart::Heading(text) => body.push_str(&format!(
                r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>{}</w:t></w:r></w:p>"#,
                escape(text)
            )),
            DocPart::Paragraph(text) => {
                body.push_str(&format!(r#"<w:p><w:r><w:t xml:space="preserve">{}</w:t></w:r></w:p>"#, escape(text)))
            }
            DocPart::Table(rows) => {
                body.push_str("<w:tbl>");
                for row in rows.iter() {
                    body.push_str("<w:tr>");
                    for cell in row.iter() {
                        body.push_str(&format!("<w:tc><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:tc>", escape(cell)));
                    }
                    body.push_str("</w:tr>");
                }
                body.push_str("</w:tbl>");
            }
        }
    }
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body}</w:body></w:document>"#
    );
    zip(&[("[Content_Types].xml", "<Types/>"), ("word/document.xml", &document)])
}

pub fn odt(parts: &[DocPart]) -> Vec<u8> {
    let mut body = String::new();
    for part in parts {
        match part {
            DocPart::Heading(text) => body.push_str(&format!("<text:h text:outline-level=\"1\">{}</text:h>", escape(text))),
            DocPart::Paragraph(text) => body.push_str(&format!("<text:p>{}</text:p>", escape(text))),
            DocPart::Table(rows) => {
                body.push_str("<table:table>");
                for row in rows.iter() {
                    body.push_str("<table:table-row>");
                    for cell in row.iter() {
                        body.push_str(&format!("<table:table-cell><text:p>{}</text:p></table:table-cell>", escape(cell)));
                    }
                    body.push_str("</table:table-row>");
                }
                body.push_str("</table:table>");
            }
        }
    }
    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><office:document-content xmlns:office="urn:oasis:names:tc:opendocument:xmlns:office:1.0" xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0" xmlns:table="urn:oasis:names:tc:opendocument:xmlns:table:1.0"><office:body><office:text>{body}</office:text></office:body></office:document-content>"#
    );
    zip(&[("mimetype", "application/vnd.oasis.opendocument.text"), ("content.xml", &content)])
}

pub fn xlsx(sheets: &[(&str, Vec<Vec<String>>)]) -> Vec<u8> {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    for (name, rows) in sheets {
        let sheet = workbook.add_worksheet();
        sheet.set_name(*name).unwrap();
        for (r, row) in rows.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                match value.parse::<f64>() {
                    Ok(number) => sheet.write_number(r as u32, c as u16, number).unwrap(),
                    Err(_) => sheet.write_string(r as u32, c as u16, value).unwrap(),
                };
            }
        }
    }
    workbook.save_to_buffer().unwrap()
}

/// A PDF with one page per entry; an empty entry makes a page without text (like a scan).
pub fn pdf(pages: &[&str]) -> Vec<u8> {
    use pdf_extract::content::{Content, Operation};
    use pdf_extract::{dictionary, Document, Object, Stream};

    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut kids = Vec::new();
    for text in pages {
        let operations = if text.is_empty() {
            Vec::new()
        } else {
            vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![72.into(), 700.into()]),
                Operation::new("Tj", vec![Object::string_literal(*text)]),
                Operation::new("ET", vec![]),
            ]
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, Content { operations }.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        kids.push(Object::from(page_id));
    }
    let count = kids.len() as i64;
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => count }));
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

pub fn png(width: u32, height: u32, alpha: bool) -> Vec<u8> {
    let image = if alpha {
        image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(width, height, image::Rgba([10, 120, 200, 128])))
    } else {
        image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(width, height, |x, y| image::Rgb([(x % 255) as u8, (y % 255) as u8, 90])))
    };
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}
