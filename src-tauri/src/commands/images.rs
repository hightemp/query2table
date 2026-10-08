//! Image previews and downloads for image-search results.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const PREVIEW_LIMIT: usize = 10 * 1024 * 1024;
const SAVE_LIMIT: usize = 50 * 1024 * 1024;

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
pub(crate) async fn download_image(url: &str, limit: usize) -> Result<(String, Vec<u8>), String> {
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

/// Tries the original image first, then the fallback (usually the thumbnail).
async fn download_with_fallback(
    url: &str,
    fallback_url: Option<&str>,
    limit: usize,
) -> Result<(String, Vec<u8>), String> {
    match download_image(url, limit).await {
        Ok(image) => Ok(image),
        Err(error) => match fallback_url.filter(|fallback| *fallback != url) {
            Some(fallback) => download_image(fallback, limit)
                .await
                .map_err(|fallback_error| format!("{error}; thumbnail: {fallback_error}")),
            None => Err(error),
        },
    }
}

fn extension_for(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" | "image/jpg" | "image/pjpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/avif" => "avif",
        "image/svg+xml" => "svg",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        _ => "img",
    }
}

/// File name made from a title: path separators and control characters removed, length capped.
fn safe_file_stem(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { ' ' } else { c })
        .collect();
    let stem = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let stem: String = stem.chars().take(80).collect();
    let stem = stem.trim_matches(|c: char| c == '.' || c.is_whitespace()).to_string();
    if stem.is_empty() { "image".to_string() } else { stem }
}

/// A path in `dir` that does not exist yet: "name.jpg", "name (2).jpg", …
fn unused_path(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let mut path = dir.join(format!("{stem}.{extension}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}).{extension}"));
        n += 1;
    }
    path
}

/// Proxy-fetch an image URL through the backend to avoid hotlink protection.
/// Returns a data URL (data:image/...;base64,...).
#[tauri::command]
pub async fn proxy_image(url: String) -> Result<String, String> {
    use base64::Engine;
    let (content_type, bytes) = download_image(&url, PREVIEW_LIMIT).await?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{content_type};base64,{b64}"))
}

/// Saves one image to the path chosen by the user. A path without an extension gets one
/// matching the downloaded file. Returns the path written.
#[tauri::command]
pub async fn save_image(
    url: String,
    fallback_url: Option<String>,
    path: String,
) -> Result<String, String> {
    let (content_type, bytes) =
        download_with_fallback(&url, fallback_url.as_deref(), SAVE_LIMIT).await?;
    let mut path = PathBuf::from(path);
    if path.extension().is_none() {
        path.set_extension(extension_for(&content_type));
    }
    tokio::fs::write(&path, bytes)
        .await
        .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(path.display().to_string())
}

#[derive(Debug, Deserialize)]
pub struct ImageDownload {
    pub url: String,
    pub fallback_url: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize, Default)]
pub struct SaveImagesResult {
    pub saved: Vec<String>,
    /// Name and reason for each image that could not be saved.
    pub failed: Vec<(String, String)>,
}

/// Saves several images into a folder. Existing files are never overwritten.
#[tauri::command]
pub async fn save_images(
    items: Vec<ImageDownload>,
    directory: String,
) -> Result<SaveImagesResult, String> {
    let dir = PathBuf::from(directory);
    if !dir.is_dir() {
        return Err(format!("Folder not found: {}", dir.display()));
    }
    let mut result = SaveImagesResult::default();
    for item in items {
        match download_with_fallback(&item.url, item.fallback_url.as_deref(), SAVE_LIMIT).await {
            Ok((content_type, bytes)) => {
                let path = unused_path(&dir, &safe_file_stem(&item.name), extension_for(&content_type));
                match tokio::fs::write(&path, bytes).await {
                    Ok(()) => result.saved.push(path.display().to_string()),
                    Err(e) => result.failed.push((item.name, e.to_string())),
                }
            }
            Err(error) => result.failed.push((item.name, error)),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_safe_and_never_overwrite() {
        assert_eq!(safe_file_stem("a/b\\c: \"d\"?"), "a b c d");
        assert_eq!(safe_file_stem("  ..  "), "image");
        assert_eq!(safe_file_stem(&"x".repeat(200)).len(), 80);
        let dir = tempfile::tempdir().unwrap();
        let first = unused_path(dir.path(), "tower", "jpg");
        std::fs::write(&first, b"1").unwrap();
        let second = unused_path(dir.path(), "tower", "jpg");
        assert_eq!(second.file_name().unwrap(), "tower (2).jpg");
    }

    #[test]
    fn extensions_follow_the_content_type() {
        assert_eq!(extension_for("image/jpeg"), "jpg");
        assert_eq!(extension_for("image/webp"), "webp");
        assert_eq!(extension_for("image/x-unknown"), "img");
    }

    #[tokio::test]
    async fn pages_are_rejected_and_the_thumbnail_is_used_instead() {
        use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET")).and(path("/page"))
            .respond_with(ResponseTemplate::new(200).insert_header("content-type", "text/html").set_body_string("<html>"))
            .mount(&server).await;
        Mock::given(method("GET")).and(path("/thumb"))
            .respond_with(ResponseTemplate::new(200).insert_header("content-type", "image/png").set_body_bytes(vec![0x89, b'P']))
            .mount(&server).await;
        let page = format!("{}/page", server.uri());
        let thumb = format!("{}/thumb", server.uri());
        let error = download_image(&page, PREVIEW_LIMIT).await.unwrap_err();
        assert!(error.contains("Not an image"), "{error}");

        let dir = tempfile::tempdir().unwrap();
        let saved = save_image(page.clone(), Some(thumb.clone()), dir.path().join("pic").display().to_string())
            .await
            .unwrap();
        assert!(saved.ends_with("pic.png"), "{saved}");

        let result = save_images(
            vec![
                ImageDownload { url: page.clone(), fallback_url: Some(thumb), name: "Tower / night".into() },
                ImageDownload { url: page, fallback_url: None, name: "Broken".into() },
            ],
            dir.path().display().to_string(),
        )
        .await
        .unwrap();
        assert_eq!(result.saved.len(), 1);
        assert!(result.saved[0].ends_with("Tower night.png"));
        assert_eq!(result.failed[0].0, "Broken");
    }
}
