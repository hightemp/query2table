//! Downloading images from the web: previews, saved files, and pictures for research answers.

fn client() -> Result<reqwest::Client, String> {
    let builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36");
    crate::providers::http::apply_proxy(builder)
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))
}

/// Downloads an image and returns its content type and bytes.
/// Pages, other non-image responses and files above `limit` are rejected.
pub async fn download_image(url: &str, limit: usize) -> Result<(String, Vec<u8>), String> {
    let resp = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Fetch error: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_string();
    if !content_type.starts_with("image/") {
        return Err(format!("Not an image: {content_type}"));
    }
    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {e}"))?;
    if bytes.len() > limit {
        return Err(format!("Image too large (>{}MB)", limit / 1024 / 1024));
    }
    Ok((content_type, bytes.to_vec()))
}


/// An image made ready for a model or for keeping: downscaled, as JPEG (PNG when transparent).
#[derive(Debug, Clone)]
pub struct Picture {
    pub media_type: &'static str,
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// Size of the file downloaded for it, before downscaling.
    pub downloaded: usize,
}

/// Downscales image bytes so the longer side is at most `side`. None when they are not a
/// readable image.
pub fn shrink(bytes: &[u8], side: u32) -> Option<Picture> {
    let image = image::load_from_memory(bytes).ok()?;
    let image = if image.width().max(image.height()) > side { image.thumbnail(side, side) } else { image };
    let mut out = std::io::Cursor::new(Vec::new());
    let media_type = if image.color().has_alpha() {
        image.write_to(&mut out, image::ImageFormat::Png).ok()?;
        "image/png"
    } else {
        image::DynamicImage::ImageRgb8(image.to_rgb8()).write_to(&mut out, image::ImageFormat::Jpeg).ok()?;
        "image/jpeg"
    };
    Some(Picture { media_type, bytes: out.into_inner(), width: image.width(), height: image.height(), downloaded: bytes.len() })
}

/// The first of `urls` that downloads as a readable image, downscaled to `side`.
pub async fn picture(urls: &[&str], side: u32, limit: usize) -> Result<Picture, String> {
    let mut error = "No image address".to_string();
    for url in urls.iter().filter(|u| !u.is_empty()) {
        match download_image(url, limit).await {
            Ok((_, bytes)) => match tokio::task::spawn_blocking(move || shrink(&bytes, side)).await.ok().flatten() {
                Some(picture) => return Ok(picture),
                None => error = "The file is not an image that can be read".to_string(),
            },
            Err(e) => error = e,
        }
    }
    Err(error)
}
