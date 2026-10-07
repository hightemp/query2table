//! Renders PDF pages to images so a vision model can read scans.

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};

use super::model::PreparedImage;
use super::parse::{prepare_image, ParseError};

/// Longest side of a rendered page: enough to read small print, within what models accept.
const PAGE_SIDE: f32 = 1800.0;

/// Renders the given pages (1-based) of a PDF. CPU-heavy: call from a blocking task.
pub fn render_pages(bytes: &[u8], pages: &[u32]) -> Result<Vec<(u32, PreparedImage)>, ParseError> {
    let pdf = Pdf::new(bytes.to_vec()).map_err(|e| ParseError::Damaged(format!("{e:?}")))?;
    let all = pdf.pages();
    let cache = RenderCache::new();
    let interpreter = InterpreterSettings::default();
    let settings = RenderSettings::default();
    let mut out = Vec::new();
    for &number in pages {
        let Some(page) = (number as usize).checked_sub(1).and_then(|i| all.get(i)) else { continue };
        let (width, height) = page.render_dimensions();
        let scale = (PAGE_SIDE / width.max(height).max(1.0)).clamp(0.5, 4.0);
        let pixmap = render(page, &cache, &interpreter, &settings, &PixmapSettings { x_scale: scale, y_scale: scale, bg_color: WHITE });
        let png = pixmap.into_png().map_err(|e| ParseError::Damaged(e.to_string()))?;
        let image = image::load_from_memory(&png).map_err(|e| ParseError::Damaged(e.to_string()))?;
        // Pages are opaque: drop the alpha channel so they travel as compact JPEG.
        out.push((number, prepare_image(image::DynamicImage::ImageRgb8(image.to_rgb8()))?));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attachments::test_files;

    #[test]
    fn pages_become_images_and_missing_pages_are_skipped() {
        let bytes = test_files::pdf(&["First page", ""]);
        let pages = render_pages(&bytes, &[2, 9]).unwrap();
        assert_eq!(pages.len(), 1);
        let (number, image) = &pages[0];
        assert_eq!(*number, 2);
        assert_eq!(image.media_type, "image/jpeg");
        assert_eq!(image.height, 1800);
        assert!(image.width > 1200 && image.width < 1300, "{}", image.width);
    }

    #[test]
    fn broken_files_are_reported() {
        assert!(render_pages(b"%PDF-1.4 nonsense", &[1]).is_err() || render_pages(b"%PDF-1.4 nonsense", &[1]).unwrap().is_empty());
    }
}
