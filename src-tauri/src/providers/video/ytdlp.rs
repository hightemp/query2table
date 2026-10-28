//! Runs yt-dlp. Its details come in one call (`--dump-single-json`); the full answer is kept in a
//! work folder so subtitles can be fetched later from it (`--load-info-json`) without asking the
//! site again, and a failed subtitle language never loses the details.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use super::subtitles::{self, Cue};
use super::{SubtitleTrack, VideoError, VideoInfo};
use crate::components::{process, JsRuntime, Tools};
use crate::providers::accounting::UsageObserver;
use crate::providers::search::VideoSearchResult;

/// Sites yt-dlp can search, by the name the agent uses, with yt-dlp's search key.
pub const SEARCHABLE_SITES: &[(&str, &str)] = &[("youtube", "ytsearch"), ("bilibili", "bilisearch"), ("niconico", "nicosearch"), ("soundcloud", "scsearch")];
/// The most results one site search returns.
const MAX_SITE_RESULTS: u32 = 20;

/// The longest one yt-dlp call may take.
const TIMEOUT: Duration = Duration::from_secs(90);

/// Where yt-dlp takes the cookies of a signed-in user from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cookies {
    /// `firefox`, `chrome`, … (`--cookies-from-browser`).
    Browser(String),
    /// A Netscape `cookies.txt` (`--cookies`).
    File(PathBuf),
}

/// Browsers yt-dlp reads cookies from.
pub const COOKIE_BROWSERS: &[&str] = &["firefox", "chrome", "edge", "brave", "opera", "vivaldi", "chromium", "safari"];

impl Cookies {
    /// The `video_cookies` setting (off, a browser, or `file` with `video_cookies_file`).
    pub fn from_settings(choice: Option<&str>, file: Option<&str>) -> Option<Self> {
        let choice = choice?.trim().to_lowercase();
        if choice == "file" {
            return file.map(str::trim).filter(|f| !f.is_empty()).map(|f| Self::File(f.into()));
        }
        COOKIE_BROWSERS.contains(&choice.as_str()).then_some(Self::Browser(choice))
    }
}

/// A video's details, and the file with yt-dlp's full answer.
#[derive(Debug, Clone)]
pub struct Video {
    pub info: VideoInfo,
    pub info_file: PathBuf,
}

pub struct YtDlp {
    tools: Tools,
    /// Temporary files of the run: answers and subtitles.
    work_dir: PathBuf,
    proxy: Option<String>,
    cookies: Option<Cookies>,
    timeout: Duration,
    observer: Option<Arc<dyn UsageObserver>>,
}

impl YtDlp {
    /// yt-dlp through the app's proxy.
    pub fn new(tools: Tools, work_dir: PathBuf) -> Self {
        Self { tools, work_dir, proxy: crate::providers::http::proxy::proxy_url(), cookies: None, timeout: TIMEOUT, observer: None }
    }

    pub fn with_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }

    pub fn with_cookies(mut self, cookies: Option<Cookies>) -> Self {
        self.cookies = cookies;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Counts what yt-dlp brings as downloaded data of the run.
    pub fn with_observer(mut self, observer: Arc<dyn UsageObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// The options of every call.
    fn common_args(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = self.tools.prefix.clone();
        args.extend(["--ignore-config", "--no-progress", "--socket-timeout", "20"].map(OsString::from));
        let engine = match &self.tools.js_runtime {
            JsRuntime::Deno(path) => Some(runtime("deno", path)),
            JsRuntime::QuickJs(path) => Some(runtime("quickjs", path)),
            JsRuntime::System => None,
        };
        if let Some(engine) = engine {
            args.extend(["--no-js-runtimes".into(), "--js-runtimes".into(), engine]);
        }
        // Without a proxy, an empty one: yt-dlp must not pick one up from the environment.
        args.push("--proxy".into());
        args.push(self.proxy.clone().unwrap_or_default().into());
        match &self.cookies {
            Some(Cookies::Browser(browser)) => args.extend(["--cookies-from-browser".into(), browser.into()]),
            Some(Cookies::File(path)) => args.extend(["--cookies".into(), path.into()]),
            None => {}
        }
        args
    }

    /// Runs yt-dlp and returns what it printed.
    async fn run(&self, args: Vec<OsString>) -> Result<Vec<u8>, VideoError> {
        let mut cmd = process::command(&self.tools.program);
        cmd.args(args).envs(self.tools.env.iter().cloned()).env("PYTHONIOENCODING", "utf-8").env("PYTHONUTF8", "1");
        let output = tokio::time::timeout(self.timeout, process::output(&mut cmd))
            .await
            .map_err(|_| VideoError::Timeout)?
            .map_err(|e| VideoError::Run(e.to_string()))?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !output.status.success() {
            tracing::warn!(target: "video", stderr = %stderr.trim(), "yt-dlp failed");
            return Err(classify(&stderr));
        }
        if !stderr.trim().is_empty() {
            tracing::debug!(target: "video", stderr = %stderr.trim(), "yt-dlp said");
        }
        if let Some(observer) = &self.observer {
            observer.downloaded(output.stdout.len() as u64);
        }
        Ok(output.stdout)
    }

    /// The details of the video at `url`.
    pub async fn info(&self, url: &str) -> Result<Video, VideoError> {
        let url = web_address(url)?;
        let mut args = self.common_args();
        args.extend(["--no-playlist", "--skip-download", "--dump-single-json", "--"].map(OsString::from));
        args.push(url.into());
        let stdout = self.run(args).await?;
        let value: serde_json::Value = serde_json::from_slice(&stdout).map_err(|e| VideoError::Failed(format!("yt-dlp gave an unreadable answer: {e}")))?;
        let info = VideoInfo::from_json(&value).ok_or_else(|| VideoError::Failed("yt-dlp gave no video".into()))?;
        tokio::fs::create_dir_all(&self.work_dir).await.map_err(|e| VideoError::Run(e.to_string()))?;
        let info_file = self.work_dir.join(format!("{}.info.json", file_stem(&info)));
        tokio::fs::write(&info_file, &stdout).await.map_err(|e| VideoError::Run(e.to_string()))?;
        Ok(Video { info, info_file })
    }
}

impl YtDlp {
    /// Searches one site with yt-dlp's own search (`ytsearchN:query`).
    pub async fn search(&self, site: &str, query: &str, count: u32) -> Result<Vec<VideoSearchResult>, VideoError> {
        let site = site.trim().to_lowercase();
        let key = SEARCHABLE_SITES
            .iter()
            .find(|(name, _)| site == *name || site.trim_end_matches(".com").trim_end_matches(".tv").trim_end_matches(".jp") == *name || (*name == "niconico" && site.contains("nicovideo")))
            .map(|(_, key)| *key)
            .ok_or_else(|| VideoError::UnknownSite(SEARCHABLE_SITES.iter().map(|(name, _)| *name).collect::<Vec<_>>().join(", ")))?;
        let query = query.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut args = self.common_args();
        args.extend(["--flat-playlist", "--skip-download", "--dump-single-json", "--"].map(OsString::from));
        args.push(format!("{key}{}:{query}", count.clamp(1, MAX_SITE_RESULTS)).into());
        let stdout = self.run(args).await?;
        let value: serde_json::Value = serde_json::from_slice(&stdout).map_err(|e| VideoError::Failed(format!("yt-dlp gave an unreadable answer: {e}")))?;
        Ok(value
            .get("entries")
            .and_then(|e| e.as_array())
            .map(|entries| entries.iter().filter_map(VideoInfo::from_json).filter(|v| !v.url.is_empty()).map(found).collect())
            .unwrap_or_default())
    }

    /// The cues of one subtitle track, fetched with the saved details.
    pub async fn subtitles(&self, video: &Video, track: &SubtitleTrack) -> Result<Vec<Cue>, VideoError> {
        let stem = file_stem(&video.info);
        let prefix = format!("{stem}.{}.", track.lang);
        let written = |dir: &Path| -> Vec<PathBuf> {
            std::fs::read_dir(dir)
                .map(|entries| entries.flatten().map(|e| e.path()).filter(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with(&prefix))).collect())
                .unwrap_or_default()
        };
        for old in written(&self.work_dir) {
            let _ = std::fs::remove_file(old);
        }
        let mut args = self.common_args();
        args.push("--load-info-json".into());
        args.push(video.info_file.clone().into());
        args.push("--skip-download".into());
        args.push(if track.automatic { "--write-auto-subs" } else { "--write-subs" }.into());
        // A list of patterns: the key must match exactly.
        args.extend(["--sub-langs".into(), regex::escape(&track.lang).into(), "--sub-format".into(), subtitles::format_choice(track).into(), "-o".into()]);
        args.push(self.work_dir.join(format!("{stem}.%(ext)s")).into());
        self.run(args).await?;

        let file = written(&self.work_dir).into_iter().next().ok_or_else(|| VideoError::Failed(format!("yt-dlp wrote no {} subtitles", track.lang)))?;
        let content = tokio::fs::read(&file).await.map_err(|e| VideoError::Run(e.to_string()))?;
        if let Some(observer) = &self.observer {
            observer.downloaded(content.len() as u64);
        }
        let format = file.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
        Ok(subtitles::parse(&format, &String::from_utf8_lossy(&content)))
    }
}

/// A search result from yt-dlp's details of a video.
fn found(info: VideoInfo) -> VideoSearchResult {
    VideoSearchResult {
        url: info.url,
        title: info.title,
        thumbnail: info.thumbnail,
        channel: info.channel,
        site: Some(info.site).filter(|s| !s.is_empty()),
        duration: info.duration,
        published: info.published,
        views: info.views,
        description: info.description,
    }
}

fn runtime(name: &str, path: &Path) -> OsString {
    let mut value = OsString::from(format!("{name}:"));
    value.push(path);
    value
}

/// An absolute http(s) address, so yt-dlp never gets a file path or an option.
fn web_address(url: &str) -> Result<String, VideoError> {
    let parsed = url::Url::parse(url.trim()).map_err(|_| VideoError::BadAddress)?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none_or(str::is_empty) {
        return Err(VideoError::BadAddress);
    }
    Ok(parsed.to_string())
}

/// A file name for the video's files: its site and id, letters and digits only.
fn file_stem(info: &VideoInfo) -> String {
    let clean = |s: &str| s.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).take(80).collect::<String>();
    format!("{}-{}", clean(&info.site), clean(&info.id))
}

/// Turns yt-dlp's last error into a reason the agent can act on.
fn classify(stderr: &str) -> VideoError {
    let message = stderr
        .lines()
        .rev()
        .find_map(|line| line.trim().strip_prefix("ERROR:"))
        .map(|m| m.trim().to_string())
        .or_else(|| stderr.lines().rev().map(str::trim).find(|l| !l.is_empty()).map(String::from))
        .unwrap_or_else(|| "yt-dlp failed".into());
    let lower = message.to_lowercase();
    let has = |needles: &[&str]| needles.iter().any(|n| lower.contains(n));
    if has(&["sign in", "logged-in", "log in", "login", "--cookies", "members-only", "age-restricted", "confirm your age"]) {
        VideoError::LoginRequired(short(&message))
    } else if has(&["http error 429", "too many requests"]) {
        VideoError::RateLimited
    } else if has(&["unsupported url"]) {
        VideoError::Unsupported
    } else if has(&["in your country", "geo restrict", "geo-restrict"]) {
        VideoError::GeoBlocked
    } else if has(&["unavailable", "private", "removed", "not found", "does not exist", "http error 404", "has been terminated"]) {
        VideoError::Unavailable(short(&message))
    } else {
        VideoError::Failed(message)
    }
}

/// The first sentence of an error, without yt-dlp's advice on options.
fn short(message: &str) -> String {
    let message = message.split(" Use --").next().unwrap_or(message);
    message.split(". ").next().unwrap_or(message).trim_end_matches('.').to_string()
}
