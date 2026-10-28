//! Downloads from GitHub releases through the app's proxy, and checksum checks.

use std::path::Path;
use std::time::{Duration, Instant};

use reqwest::{header, redirect, Client, Response, StatusCode};
use sha2::Digest;
use tokio::io::AsyncWriteExt;

use super::ComponentError;

const USER_AGENT: &str = concat!("Query2Table/", env!("CARGO_PKG_VERSION"));
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);
/// No bytes for this long means the connection is dead.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

pub struct Http {
    client: Client,
    /// Reads the `releases/latest` redirect without following it.
    no_redirect: Client,
    /// `https://github.com`, or a test server.
    base: String,
}

impl Http {
    pub fn new(base: &str) -> Result<Self, ComponentError> {
        Self::with_read_timeout(base, READ_TIMEOUT)
    }

    pub fn with_read_timeout(base: &str, read_timeout: Duration) -> Result<Self, ComponentError> {
        let build = |policy| {
            crate::providers::http::apply_proxy(Client::builder())
                .user_agent(USER_AGENT)
                .connect_timeout(Duration::from_secs(15))
                .read_timeout(read_timeout)
                .redirect(policy)
                .build()
                .map_err(network)
        };
        Ok(Self { client: build(redirect::Policy::limited(10))?, no_redirect: build(redirect::Policy::none())?, base: base.trim_end_matches('/').to_string() })
    }

    /// The latest release tag, read from the `releases/latest` redirect rather than the REST API,
    /// which allows only 60 requests an hour without a token.
    pub async fn latest_tag(&self, repo: &str) -> Result<String, ComponentError> {
        let url = format!("{}/{repo}/releases/latest", self.base);
        let resp = check_status(self.no_redirect.get(&url).send().await.map_err(network)?, &url, true)?;
        resp.headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|location| location.rsplit_once("/tag/"))
            .map(|(_, tag)| tag.to_string())
            .filter(|tag| !tag.is_empty())
            .ok_or_else(|| ComponentError::Network(format!("no release tag in the redirect from {url}")))
    }

    pub fn asset_url(&self, repo: &str, tag: &str, asset: &str) -> String {
        format!("{}/{repo}/releases/download/{tag}/{asset}", self.base)
    }

    pub async fn get_text(&self, url: &str) -> Result<String, ComponentError> {
        let resp = check_status(self.client.get(url).send().await.map_err(network)?, url, false)?;
        resp.text().await.map_err(network)
    }

    /// Streams `url` into `dest`, reporting `(downloaded, total)` at most every 100 ms.
    pub async fn download(&self, url: &str, dest: &Path, mut on_progress: impl FnMut(u64, Option<u64>)) -> Result<(), ComponentError> {
        let mut resp = check_status(self.client.get(url).send().await.map_err(network)?, url, false)?;
        let total = resp.content_length();
        let mut file = tokio::fs::File::create(dest).await?;
        let mut downloaded = 0u64;
        let mut last_report = Instant::now();
        on_progress(0, total);
        while let Some(chunk) = resp.chunk().await.map_err(network)? {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if last_report.elapsed() >= PROGRESS_INTERVAL {
                on_progress(downloaded, total);
                last_report = Instant::now();
            }
        }
        file.flush().await?;
        file.sync_all().await?;
        on_progress(downloaded, total);
        Ok(())
    }
}

fn network(error: reqwest::Error) -> ComponentError {
    ComponentError::Network(error.to_string())
}

fn check_status(resp: Response, url: &str, allow_redirect: bool) -> Result<Response, ComponentError> {
    let status = resp.status();
    if status.is_success() || (allow_redirect && status.is_redirection()) {
        return Ok(resp);
    }
    let rate_limited = status == StatusCode::TOO_MANY_REQUESTS
        || (status == StatusCode::FORBIDDEN && resp.headers().get("x-ratelimit-remaining").is_some_and(|v| v == "0"));
    if rate_limited {
        return Err(ComponentError::RateLimited);
    }
    Err(ComponentError::Network(format!("HTTP {status} for {url}")))
}

/// The hash of `name` in a listing of `<hash>  <name>`, `<hash> *<name>`, or PowerShell
/// `Get-FileHash` output (Deno's Windows builds).
pub fn find_checksum(listing: &str, name: &str) -> Option<String> {
    listing
        .lines()
        .find_map(|line| {
            let (hash, file) = line.trim().split_once(char::is_whitespace)?;
            let file = file.trim().trim_start_matches('*');
            (file == name && is_hex(hash)).then(|| hash.to_ascii_lowercase())
        })
        .or_else(|| find_powershell_checksum(listing, name))
}

/// `Hash : <HEX>` followed by `Path : C:\...\<name>`.
fn find_powershell_checksum(listing: &str, name: &str) -> Option<String> {
    let mut hash = None;
    for line in listing.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        let value = value.trim();
        match key.trim() {
            "Hash" if is_hex(value) => hash = Some(value.to_ascii_lowercase()),
            "Path" => {
                if value.rsplit(['\\', '/']).next() == Some(name) {
                    return hash;
                }
                hash = None;
            }
            _ => {}
        }
    }
    None
}

fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// The SHA-256 of a file, as lowercase hex.
pub async fn sha256(path: &Path) -> Result<String, ComponentError> {
    let path = path.to_owned();
    tokio::task::spawn_blocking(move || -> Result<String, ComponentError> {
        use std::io::Read;
        let mut file = std::fs::File::open(path)?;
        let mut hasher = sha2::Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|e| ComponentError::Other(e.to_string()))?
}

pub async fn verify(path: &Path, expected: &str, name: &str) -> Result<(), ComponentError> {
    let actual = sha256(path).await?;
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        tracing::error!(name, expected, actual, "checksum mismatch");
        Err(ComponentError::Checksum(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_listing_formats() {
        let sums = "aa11  yt-dlp\nbb22  yt-dlp.exe\ncc33 *yt-dlp_macos\n";
        assert_eq!(find_checksum(sums, "yt-dlp.exe").as_deref(), Some("bb22"));
        assert_eq!(find_checksum(sums, "yt-dlp_macos").as_deref(), Some("cc33"));
        assert_eq!(find_checksum(sums, "yt-dlp_linux"), None);
    }

    /// Deno's Windows `.sha256sum` files are PowerShell `Get-FileHash` output (CRLF,
    /// uppercase hash, full build path), as in the v2.9.8 release.
    #[test]
    fn checksum_in_powershell_format() {
        let listing = "\r\nAlgorithm : SHA256\r\nHash      : CBA84B26E0DF5AE7F74FC66B86A95B866872A86292EBD84FA1E959927EDD93BF\r\nPath      : C:\\a\\deno\\deno\\target\\release\\deno-x86_64-pc-windows-msvc.zip\r\n\r\n";
        assert_eq!(
            find_checksum(listing, "deno-x86_64-pc-windows-msvc.zip").as_deref(),
            Some("cba84b26e0df5ae7f74fc66b86a95b866872a86292ebd84fa1e959927edd93bf")
        );
        assert_eq!(find_checksum(listing, "deno-aarch64-pc-windows-msvc.zip"), None);
    }

    /// A server that sends headers and then goes silent must not hang an install, which
    /// holds the install lock.
    #[tokio::test]
    async fn a_stalled_download_times_out() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = tokio::io::AsyncReadExt::read(&mut socket, &mut buf).await;
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\nabc").await.unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
        });
        let http = Http::with_read_timeout("http://unused", Duration::from_millis(300)).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let started = Instant::now();
        let result = http.download(&format!("http://{addr}/x"), &dir.path().join("x"), |_, _| {}).await;
        assert!(result.is_err(), "{result:?}");
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn file_digest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc.txt");
        std::fs::write(&path, b"abc").unwrap();
        assert_eq!(sha256(&path).await.unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
        assert!(matches!(verify(&path, "00", "abc").await, Err(ComponentError::Checksum(_))));
    }
}
