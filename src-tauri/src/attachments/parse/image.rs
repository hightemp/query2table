//! Images: turned upright, downscaled for models, with a thumbnail for the interface.

use std::io::Cursor;

use image::{DynamicImage, ImageDecoder, ImageReader};

use super::ParseError;
use crate::attachments::model::{AttachmentKind, ParsedFile, PreparedImage};

/// Longest side sent to a model; larger photos only cost more tokens.
pub const MAX_SIDE: u32 = 2048;
const THUMBNAIL_SIDE: u32 = 160;

pub fn parse(bytes: &[u8]) -> Result<ParsedFile, ParseError> {
    let image = decode(bytes)?;
    let prepared = prepare(image)?;
    Ok(ParsedFile {
        kind: AttachmentKind::Image,
        mime: prepared.media_type,
        page_count: None,
        sheets: Vec::new(),
        fragments: Vec::new(),
        scanned_pages: Vec::new(),
        headings: Vec::new(),
        image: Some(prepared),
    })
}

fn decode(bytes: &[u8]) -> Result<DynamicImage, ParseError> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| ParseError::Damaged(e.to_string()))?;
    let mut decoder = reader.into_decoder().map_err(|_| ParseError::Damaged("not a supported image".into()))?;
    let orientation = decoder.orientation().ok();
    let mut image = DynamicImage::from_decoder(decoder).map_err(|e| ParseError::Damaged(e.to_string()))?;
    if let Some(orientation) = orientation {
        image.apply_orientation(orientation);
    }
    Ok(image)
}

/// Downscales to [`MAX_SIDE`] and encodes as PNG (transparent images) or JPEG.
pub fn prepare(image: DynamicImage) -> Result<PreparedImage, ParseError> {
    let image = if image.width().max(image.height()) > MAX_SIDE {
        image.resize(MAX_SIDE, MAX_SIDE, image::imageops::FilterType::Triangle)
    } else {
        image
    };
    let (media_type, bytes) = if image.color().has_alpha() {
        ("image/png", encode(&image, image::ImageFormat::Png)?)
    } else {
        ("image/jpeg", encode(&DynamicImage::ImageRgb8(image.to_rgb8()), image::ImageFormat::Jpeg)?)
    };
    let thumbnail = image.thumbnail(THUMBNAIL_SIDE, THUMBNAIL_SIDE);
    let thumbnail_jpeg = encode(&DynamicImage::ImageRgb8(thumbnail.to_rgb8()), image::ImageFormat::Jpeg)?;
    Ok(PreparedImage { media_type, bytes, width: image.width(), height: image.height(), thumbnail_jpeg })
}

fn encode(image: &DynamicImage, format: image::ImageFormat) -> Result<Vec<u8>, ParseError> {
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, format).map_err(|e| ParseError::Damaged(e.to_string()))?;
    Ok(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;

    #[test]
    fn large_photos_are_downscaled_and_get_a_thumbnail() {
        let parsed = parse(&test_files::png(3000, 1000, false)).unwrap();
        let image = parsed.image.unwrap();
        assert_eq!((image.width, image.height), (2048, 683));
        assert_eq!(image.media_type, "image/jpeg");
        assert!(image.bytes.starts_with(&[0xFF, 0xD8]));
        assert!(image.thumbnail_jpeg.starts_with(&[0xFF, 0xD8]));
    }

    #[test]
    fn transparent_images_stay_png_and_garbage_is_rejected() {
        let image = parse(&test_files::png(40, 30, true)).unwrap().image.unwrap();
        assert_eq!((image.media_type, image.width, image.height), ("image/png", 40, 30));
        assert!(matches!(parse(b"not an image"), Err(ParseError::Damaged(_))));
    }
}
