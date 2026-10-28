//! Videos on the sites yt-dlp supports: their details, subtitles and thumbnails.

pub mod subtitles;
pub mod ytdlp;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use ytdlp::{Cookies, Video, YtDlp};

/// Why a video could not be read, worded for the research agent.
#[derive(Debug, Clone, thiserror::Error)]
pub enum VideoError {
    #[error("only http(s) video addresses can be read")]
    BadAddress,
    #[error("yt-dlp cannot read videos from this address")]
    Unsupported,
    #[error("searching this site is not possible; sites that can be searched: {0}")]
    UnknownSite(String),
    #[error("the site wants a signed-in user ({0}); browser cookies can be set in Settings › Runs › Research")]
    LoginRequired(String),
    #[error("the site is limiting requests (HTTP 429); try another video")]
    RateLimited,
    #[error("the video is not available: {0}")]
    Unavailable(String),
    #[error("the video is not available in this country")]
    GeoBlocked,
    #[error("the site did not answer in time")]
    Timeout,
    #[error("yt-dlp could not start: {0}")]
    Run(String),
    #[error("{0}")]
    Failed(String),
}

/// A chapter, in seconds from the start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub start: u64,
    pub end: u64,
    pub title: String,
}

/// One subtitle language of a video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleTrack {
    /// The site's key: `en`, `pt-BR`, `en-orig` (YouTube's own speech recognition), or a named
    /// track like `en-IU4AG2iTnVA`.
    pub lang: String,
    pub name: Option<String>,
    /// Made by speech recognition or machine translation, not by people.
    pub automatic: bool,
    /// File formats offered: json3, vtt, srt…
    pub formats: Vec<String>,
}

/// What yt-dlp tells about a video.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub url: String,
    /// yt-dlp's name for the site (`Youtube`, `TedTalk`…).
    pub site: String,
    pub channel: Option<String>,
    pub channel_url: Option<String>,
    /// `YYYY-MM-DD`.
    pub published: Option<String>,
    /// Seconds.
    pub duration: Option<u64>,
    pub views: Option<u64>,
    pub language: Option<String>,
    pub live: bool,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub chapters: Vec<Chapter>,
    pub tracks: Vec<SubtitleTrack>,
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty()).map(String::from)
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(|v| v.as_u64().or_else(|| v.as_f64().filter(|f| *f >= 0.0).map(|f| f.round() as u64)))
}

/// `20091025` → `2009-10-25`, else the date of the Unix `timestamp`.
fn published(value: &Value) -> Option<String> {
    if let Some(date) = text(value, "upload_date").filter(|d| d.len() == 8 && d.chars().all(|c| c.is_ascii_digit())) {
        return Some(format!("{}-{}-{}", &date[..4], &date[4..6], &date[6..]));
    }
    let timestamp = value.get("timestamp").or_else(|| value.get("release_timestamp")).and_then(Value::as_i64)?;
    chrono::DateTime::from_timestamp(timestamp, 0).map(|t| t.format("%Y-%m-%d").to_string())
}

/// The `thumbnail` yt-dlp picked, else the listed one it prefers most, then the widest.
fn thumbnail(value: &Value) -> Option<String> {
    text(value, "thumbnail").or_else(|| {
        value
            .get("thumbnails")?
            .as_array()?
            .iter()
            .filter(|t| t.get("url").and_then(Value::as_str).is_some())
            .max_by_key(|t| (t.get("preference").and_then(Value::as_i64).unwrap_or(-1000), number(t, "width").unwrap_or(0)))
            .and_then(|t| text(t, "url"))
    })
}

fn tracks(value: &Value, key: &str, automatic: bool) -> Vec<SubtitleTrack> {
    let Some(map) = value.get(key).and_then(Value::as_object) else { return Vec::new() };
    map.iter()
        // YouTube lists a live stream's chat as subtitles.
        .filter(|(lang, _)| lang.as_str() != "live_chat")
        .filter_map(|(lang, files)| {
            let files = files.as_array()?;
            let mut formats: Vec<String> = Vec::new();
            for ext in files.iter().filter_map(|f| text(f, "ext")) {
                if !formats.contains(&ext) {
                    formats.push(ext);
                }
            }
            (!formats.is_empty()).then(|| SubtitleTrack { lang: lang.clone(), name: files.iter().find_map(|f| text(f, "name")), automatic, formats })
        })
        .collect()
}

impl VideoInfo {
    pub fn from_json(value: &Value) -> Option<Self> {
        let id = text(value, "id")?;
        let chapters = value
            .get("chapters")
            .and_then(Value::as_array)
            .map(|list| {
                list.iter()
                    .filter_map(|c| Some(Chapter { start: number(c, "start_time")?, end: number(c, "end_time")?, title: text(c, "title").unwrap_or_default() }))
                    .collect()
            })
            .unwrap_or_default();
        let mut all_tracks = tracks(value, "subtitles", false);
        all_tracks.extend(tracks(value, "automatic_captions", true));
        Some(Self {
            title: text(value, "title").or_else(|| text(value, "fulltitle")).unwrap_or_else(|| id.clone()),
            url: text(value, "webpage_url").or_else(|| text(value, "original_url")).or_else(|| text(value, "url")).unwrap_or_default(),
            site: text(value, "extractor_key").or_else(|| text(value, "ie_key")).or_else(|| text(value, "extractor")).unwrap_or_default(),
            channel: text(value, "channel").or_else(|| text(value, "uploader")).or_else(|| text(value, "creator")),
            channel_url: text(value, "channel_url").or_else(|| text(value, "uploader_url")),
            published: published(value),
            duration: number(value, "duration"),
            views: number(value, "view_count"),
            language: text(value, "language"),
            live: value.get("is_live").and_then(Value::as_bool).unwrap_or(false) || text(value, "live_status").as_deref() == Some("is_live"),
            thumbnail: thumbnail(value),
            description: text(value, "description"),
            chapters,
            tracks: all_tracks,
            id,
        })
    }
}
