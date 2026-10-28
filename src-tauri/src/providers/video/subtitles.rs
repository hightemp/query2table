//! Subtitles: which track to read, reading its file, and the text the research agent gets —
//! paragraphs with `[m:ss]` marks under chapter headings, whole or a page at a time.

use std::sync::LazyLock;

use regex::Regex;

use super::{Chapter, SubtitleTrack, VideoInfo};

/// One subtitle line, from its start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cue {
    pub start_ms: u64,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    /// Made by people.
    Manual,
    /// Speech recognition in the video's language.
    Automatic,
    /// A machine translation of the speech recognition.
    Translated,
}

fn base(lang: &str) -> String {
    lang.split(['-', '_']).next().unwrap_or(lang).to_ascii_lowercase()
}

fn is_original(track: &SubtitleTrack) -> bool {
    track.lang.ends_with("-orig")
}

/// The track to read: people's subtitles in the language asked for (or the video's own), then
/// speech recognition in it, then a machine translation into it. Without a wish, any people's
/// subtitles or speech recognition will do.
pub fn choose_track<'a>(info: &'a VideoInfo, wanted: Option<&str>) -> Option<&'a SubtitleTrack> {
    let in_language = |lang: &str| -> Option<&'a SubtitleTrack> {
        let lang = lang.trim();
        let manual = info.tracks.iter().filter(|t| !t.automatic);
        let automatic = || info.tracks.iter().filter(|t| t.automatic);
        manual
            .clone()
            .find(|t| t.lang.eq_ignore_ascii_case(lang))
            .or_else(|| manual.clone().find(|t| base(&t.lang) == base(lang)))
            .or_else(|| automatic().find(|t| is_original(t) && base(&t.lang) == base(lang)))
            .or_else(|| automatic().find(|t| t.lang.eq_ignore_ascii_case(lang)))
            .or_else(|| automatic().find(|t| !is_original(t) && base(&t.lang) == base(lang)))
    };
    match wanted.filter(|w| !w.trim().is_empty()) {
        Some(wanted) => in_language(wanted),
        None => info
            .language
            .as_deref()
            .and_then(in_language)
            .or_else(|| info.tracks.iter().find(|t| !t.automatic))
            .or_else(|| info.tracks.iter().find(|t| t.automatic && is_original(t))),
    }
}

pub fn track_kind(info: &VideoInfo, track: &SubtitleTrack) -> TrackKind {
    if !track.automatic {
        TrackKind::Manual
    } else if is_original(track) || !info.tracks.iter().any(|t| t.automatic && is_original(t)) {
        TrackKind::Automatic
    } else {
        TrackKind::Translated
    }
}

/// The formats asked of yt-dlp, the ones read best first.
pub fn format_choice(_track: &SubtitleTrack) -> &'static str {
    "json3/vtt/srt/best"
}

// --- Reading files ---

/// Reads a subtitle file of the given format (json3, vtt, srt, srv1, ttml).
pub fn parse(format: &str, content: &str) -> Vec<Cue> {
    let content = content.trim_start_matches('\u{feff}');
    match format {
        "json3" => parse_json3(content),
        "vtt" | "srt" => parse_blocks(content),
        _ => parse_xml(content),
    }
}

fn parse_json3(content: &str) -> Vec<Cue> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else { return Vec::new() };
    let events = value.get("events").and_then(|e| e.as_array()).cloned().unwrap_or_default();
    events
        .iter()
        .filter_map(|event| {
            let start_ms = event.get("tStartMs")?.as_u64()?;
            let text: String = event.get("segs")?.as_array()?.iter().filter_map(|s| s.get("utf8")?.as_str()).collect();
            let text = clean(&text);
            (!text.is_empty()).then_some(Cue { start_ms, text })
        })
        .collect()
}

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

/// Text without tags and entities, on one line.
fn clean(text: &str) -> String {
    let text = TAG.replace_all(text, "");
    let text = text.replace("&nbsp;", " ").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'").replace("&apos;", "'").replace("&amp;", "&");
    SPACES.replace_all(&text, " ").trim().to_string()
}

/// `01:02:03.000`, `02:03.000` or `01:02:03,000` in milliseconds.
fn timestamp_ms(value: &str) -> Option<u64> {
    let value = value.trim().replace(',', ".");
    let (clock, fraction) = value.split_once('.').unwrap_or((&value, "0"));
    let mut seconds = 0u64;
    for part in clock.split(':') {
        seconds = seconds * 60 + part.parse::<u64>().ok()?;
    }
    let millis: u64 = format!("{fraction:0<3}")[..3].parse().ok()?;
    Some(seconds * 1000 + millis)
}

/// WebVTT and SRT: a timing line, then text lines up to a blank one. YouTube's automatic
/// captions roll, repeating the line before, so a line just read is not read again.
fn parse_blocks(content: &str) -> Vec<Cue> {
    let mut cues = Vec::new();
    let mut recent: Vec<String> = Vec::new();
    let mut current: Option<(u64, Vec<String>)> = None;
    let mut flush = |current: &mut Option<(u64, Vec<String>)>, cues: &mut Vec<Cue>| {
        let Some((start_ms, lines)) = current.take() else { return };
        let mut fresh = Vec::new();
        for line in lines.iter().map(|l| clean(l)).filter(|l| !l.is_empty()) {
            if !recent.contains(&line) {
                fresh.push(line.clone());
            }
            recent.push(line);
            if recent.len() > 2 {
                recent.remove(0);
            }
        }
        if !fresh.is_empty() {
            cues.push(Cue { start_ms, text: fresh.join(" ") });
        }
    };
    for line in content.lines() {
        if line.contains("-->") {
            flush(&mut current, &mut cues);
            current = line.split("-->").next().and_then(timestamp_ms).map(|start| (start, Vec::new()));
        } else if line.trim().is_empty() {
            flush(&mut current, &mut cues);
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(line.to_string());
        }
    }
    flush(&mut current, &mut cues);
    cues
}

static SRV1: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)<text[^>]*\bstart="([\d.]+)"[^>]*>(.*?)</text>"#).unwrap());
static TTML: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"(?s)<p[^>]*\bbegin="([^"]+)"[^>]*>(.*?)</p>"#).unwrap());

/// srv1 (`<text start="1.36">`) and TTML (`<p begin="00:00:01.360">`).
fn parse_xml(content: &str) -> Vec<Cue> {
    let srv1 = SRV1.captures_iter(content).filter_map(|c| Some(((c[1].parse::<f64>().ok()? * 1000.0) as u64, c[2].to_string())));
    let ttml = TTML.captures_iter(content).filter_map(|c| Some((timestamp_ms(&c[1])?, c[2].replace("<br/>", " ").replace("<br />", " "))));
    srv1.chain(ttml).map(|(start_ms, text)| Cue { start_ms, text: clean(&text) }).filter(|c| !c.text.is_empty()).collect()
}

// --- Times and links ---

/// `m:ss`, or `h:mm:ss` from an hour.
pub fn clock(seconds: u64) -> String {
    let (h, m, s) = (seconds / 3600, seconds / 60 % 60, seconds % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// `40:00`, `1:02:03` or `754` in seconds.
pub fn parse_clock(value: &str) -> Option<u64> {
    let value = value.trim();
    if value.is_empty() || value.split(':').count() > 3 {
        return None;
    }
    value.split(':').try_fold(0u64, |total, part| Some(total * 60 + part.trim().parse::<u64>().ok()?))
}

/// The address of the video at `seconds`, in the site's form.
pub fn time_link(url: &str, seconds: u64) -> String {
    let Ok(mut parsed) = url::Url::parse(url) else { return url.to_string() };
    if seconds == 0 {
        return url.to_string();
    }
    let host = parsed.host_str().unwrap_or_default().trim_start_matches("www.").trim_start_matches("m.").to_string();
    match host.as_str() {
        "youtube.com" | "youtu.be" | "music.youtube.com" => {
            let kept: Vec<(String, String)> = parsed.query_pairs().filter(|(k, _)| k != "t").map(|(k, v)| (k.into_owned(), v.into_owned())).collect();
            parsed.query_pairs_mut().clear().extend_pairs(kept).append_pair("t", &format!("{seconds}s"));
        }
        "vimeo.com" => parsed.set_fragment(Some(&format!("t={seconds}s"))),
        // Media fragments: understood by players that follow the W3C rule.
        _ => parsed.set_fragment(Some(&format!("t={seconds}"))),
    }
    parsed.to_string()
}

// --- The text the agent reads ---

/// A paragraph is closed after this long at a sentence end, and always after the longer limit.
const PARAGRAPH_MS: u64 = 30_000;
const PARAGRAPH_MAX_MS: u64 = 60_000;

#[derive(Debug, Clone)]
struct Paragraph {
    start_ms: u64,
    end_ms: u64,
    /// The chapter this paragraph opens.
    heading: Option<String>,
    text: String,
}

impl Paragraph {
    fn render(&self) -> String {
        let line = format!("[{}] {}", clock(self.start_ms / 1000), self.text);
        match &self.heading {
            Some(heading) => format!("## {heading}\n{line}"),
            None => line,
        }
    }
}

/// A part of the text, and where the next part starts (seconds) when there is more.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptPart {
    pub text: String,
    pub next: Option<u64>,
}

/// A video's subtitles as paragraphs with time marks, under its chapters.
#[derive(Debug, Clone)]
pub struct Transcript {
    paragraphs: Vec<Paragraph>,
}

impl Transcript {
    pub fn new(cues: &[Cue], chapters: &[Chapter]) -> Self {
        let chapter_of = |ms: u64| chapters.iter().rposition(|c| c.start * 1000 <= ms);
        let mut paragraphs: Vec<Paragraph> = Vec::new();
        let mut current_chapter = None;
        let mut last_text: Option<&str> = None;
        for cue in cues {
            let text = cue.text.trim();
            if text.is_empty() || last_text == Some(text) {
                continue;
            }
            last_text = Some(text);
            let chapter = chapter_of(cue.start_ms);
            let new_chapter = chapter.is_some() && chapter != current_chapter;
            let close = match paragraphs.last() {
                None => true,
                Some(p) => {
                    let length = cue.start_ms.saturating_sub(p.start_ms);
                    new_chapter || length >= PARAGRAPH_MAX_MS || (length >= PARAGRAPH_MS && p.text.ends_with(['.', '!', '?', '…', '。', '！', '？']))
                }
            };
            if close {
                paragraphs.push(Paragraph {
                    start_ms: cue.start_ms,
                    end_ms: cue.start_ms,
                    heading: new_chapter.then(|| chapters[chapter.unwrap_or_default()].title.clone()),
                    text: text.to_string(),
                });
            } else if let Some(p) = paragraphs.last_mut() {
                p.text.push(' ');
                p.text.push_str(text);
                p.end_ms = cue.start_ms;
            }
            current_chapter = chapter;
        }
        Self { paragraphs }
    }

    pub fn is_empty(&self) -> bool {
        self.paragraphs.is_empty()
    }

    /// The whole text, cut to `max_chars`.
    pub fn full(&self, max_chars: usize) -> TranscriptPart {
        self.part_from(0, max_chars)
    }

    /// The text from the paragraph holding `from` seconds, up to `max_chars`.
    pub fn page(&self, from: u64, max_chars: usize) -> TranscriptPart {
        let past_end = self.paragraphs.last().is_none_or(|p| p.end_ms / 1000 < from);
        if past_end {
            return TranscriptPart { text: String::new(), next: None };
        }
        let index = self.paragraphs.iter().rposition(|p| p.start_ms / 1000 <= from).unwrap_or(0);
        self.part_from(index, max_chars)
    }

    fn part_from(&self, index: usize, max_chars: usize) -> TranscriptPart {
        let mut text = String::new();
        for (i, paragraph) in self.paragraphs.iter().enumerate().skip(index) {
            let rendered = paragraph.render();
            let separator = if text.is_empty() { 0 } else { 2 };
            if text.len() + separator + rendered.len() > max_chars {
                if text.is_empty() {
                    // A paragraph longer than a page is cut, so every page moves on.
                    let mut end = max_chars.min(rendered.len());
                    while !rendered.is_char_boundary(end) {
                        end -= 1;
                    }
                    text.push_str(&rendered[..end]);
                    let next = self.paragraphs.get(i + 1).map(|p| p.start_ms / 1000);
                    return TranscriptPart { text, next };
                }
                return TranscriptPart { text, next: Some(paragraph.start_ms / 1000) };
            }
            if separator > 0 {
                text.push_str("\n\n");
            }
            text.push_str(&rendered);
        }
        TranscriptPart { text, next: None }
    }
}
