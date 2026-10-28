//! Videos of a research run: searching one site and reading a video's details and subtitles with
//! yt-dlp (downloaded on first use), and the text the agent reads about a video.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::components::Tools;
use crate::providers::accounting::UsageObserver;
use crate::providers::search::VideoSearchResult;
use crate::providers::video::subtitles::{choose_track, clock, time_link, track_kind, TrackKind, Transcript};
use crate::providers::video::{Cookies, SubtitleTrack, Video, VideoInfo, YtDlp};

/// A video the agent came across: in video results or read with `video`.
#[derive(Debug, Clone, PartialEq)]
pub struct SeenVideo {
    pub url: String,
    pub title: String,
    pub thumbnail: Option<String>,
    pub channel: Option<String>,
    /// Seconds.
    pub duration: Option<u64>,
    pub site: Option<String>,
}

impl From<&VideoSearchResult> for SeenVideo {
    fn from(r: &VideoSearchResult) -> Self {
        Self { url: r.url.clone(), title: r.title.clone(), thumbnail: r.thumbnail.clone(), channel: r.channel.clone(), duration: r.duration, site: r.site.clone() }
    }
}

impl From<&VideoInfo> for SeenVideo {
    fn from(v: &VideoInfo) -> Self {
        Self { url: v.url.clone(), title: v.title.clone(), thumbnail: v.thumbnail.clone(), channel: v.channel.clone(), duration: v.duration, site: Some(v.site.clone()).filter(|s| !s.is_empty()) }
    }
}

/// How much subtitle text one `video` call gives.
#[derive(Debug, Clone, Copy)]
pub struct TextBudget {
    /// A part.
    pub page_chars: usize,
    /// The whole text, when asked for.
    pub full_chars: usize,
}

/// What the agent learns from reading a video.
#[derive(Debug, Clone)]
pub struct VideoRead {
    pub info: VideoInfo,
    pub text: String,
}

/// A video read in this run, kept so later parts need no new requests.
struct Opened {
    video: Video,
    track: Option<SubtitleTrack>,
    transcript: Option<Transcript>,
    /// Why the subtitles could not be read.
    subtitle_error: Option<String>,
}

pub struct RunVideos {
    work_dir: PathBuf,
    cookies: Option<Cookies>,
    observer: Option<Arc<dyn UsageObserver>>,
    /// yt-dlp given by a test instead of the installed one.
    tools: Option<Tools>,
    ytdlp: tokio::sync::OnceCell<Result<YtDlp, String>>,
    opened: tokio::sync::Mutex<HashMap<(String, Option<String>), Opened>>,
}

impl RunVideos {
    pub fn new(run_id: &str, cookies: Option<Cookies>) -> Self {
        let id: String = run_id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
        Self {
            work_dir: std::env::temp_dir().join("query2table-videos").join(id),
            cookies,
            observer: None,
            tools: None,
            ytdlp: tokio::sync::OnceCell::new(),
            opened: tokio::sync::Mutex::new(HashMap::new()),
        }
    }

    pub fn with_observer(mut self, observer: Arc<dyn UsageObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Runs this yt-dlp instead of the installed one (tests).
    pub fn with_tools(mut self, tools: Tools) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Runs the user's own yt-dlp (Settings › Application › Components), with our Deno when it is
    /// installed. Not on Android, where only the bundled Python runs programs.
    pub fn with_custom_ytdlp(mut self, program: Option<PathBuf>) -> Self {
        if let (Some(program), None, false) = (program, &self.tools, cfg!(target_os = "android")) {
            self.tools = Some(match crate::components::shared() {
                Ok(manager) => manager.custom_tools(program),
                Err(_) => Tools { program, prefix: Vec::new(), env: Vec::new(), js_runtime: crate::components::JsRuntime::System },
            });
        }
        self
    }

    /// yt-dlp is not ready yet and is downloaded on first use.
    pub fn needs_install(&self) -> bool {
        self.tools.is_none() && !self.ytdlp.initialized() && crate::components::shared().is_ok_and(|m| m.tools().is_err())
    }

    async fn ytdlp(&self) -> Result<&YtDlp, String> {
        let result = self
            .ytdlp
            .get_or_init(|| async {
                let tools = match &self.tools {
                    Some(tools) => tools.clone(),
                    None => {
                        let manager = crate::components::shared().map_err(|e| format!("videos cannot be read on this system: {e}"))?;
                        let last = std::sync::Mutex::new(None);
                        manager
                            .ensure(&|p| {
                                let mut last = last.lock().unwrap_or_else(|e| e.into_inner());
                                if *last != Some((p.component, p.stage)) {
                                    tracing::info!(target: "video", component = ?p.component, stage = ?p.stage, file = %p.file, "installing video tools");
                                    *last = Some((p.component, p.stage));
                                }
                            })
                            .await
                            .map_err(|e| format!("yt-dlp could not be installed: {e}"))?
                    }
                };
                let mut ytdlp = YtDlp::new(tools, self.work_dir.clone()).with_cookies(self.cookies.clone());
                if let Some(observer) = &self.observer {
                    ytdlp = ytdlp.with_observer(observer.clone());
                }
                Ok(ytdlp)
            })
            .await;
        result.as_ref().map_err(Clone::clone)
    }

    /// Searches one site with yt-dlp.
    pub async fn search_site(&self, site: &str, query: &str, count: u32) -> Result<Vec<VideoSearchResult>, String> {
        self.ytdlp().await?.search(site, query, count).await.map_err(|e| e.to_string())
    }

    /// Reads a video: its details, and a part of its subtitles (from `from` seconds) or all of them.
    pub async fn read(&self, url: &str, lang: Option<&str>, from: Option<u64>, full: bool, budget: TextBudget) -> Result<VideoRead, String> {
        let key = (url.to_string(), lang.map(str::to_lowercase));
        let mut opened = self.opened.lock().await;
        if !opened.contains_key(&key) {
            let ytdlp = self.ytdlp().await?;
            // The details of a video read before in another language are reused.
            let known = opened.iter().find(|((u, _), _)| u == url).map(|(_, o)| o.video.clone());
            let video = match known {
                Some(video) => video,
                None => ytdlp.info(url).await.map_err(|e| e.to_string())?,
            };
            let track = choose_track(&video.info, lang).cloned();
            let (transcript, subtitle_error) = match &track {
                Some(track) => match ytdlp.subtitles(&video, track).await {
                    Ok(cues) => (Some(Transcript::new(&cues, &video.info.chapters)).filter(|t| !t.is_empty()), None),
                    Err(e) => (None, Some(e.to_string())),
                },
                None => (None, None),
            };
            opened.insert(key.clone(), Opened { video, track, transcript, subtitle_error });
        }
        let found = &opened[&key];
        Ok(VideoRead { info: found.video.info.clone(), text: describe(found, lang, from, full, budget) })
    }
}

impl Drop for RunVideos {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.work_dir);
    }
}

fn kind_text(kind: TrackKind) -> &'static str {
    match kind {
        TrackKind::Manual => "made by people",
        TrackKind::Automatic => "automatic speech recognition, may contain errors",
        TrackKind::Translated => "machine translation of the speech recognition, may contain errors",
    }
}

/// The text the agent reads about a video.
fn describe(opened: &Opened, wanted: Option<&str>, from: Option<u64>, full: bool, budget: TextBudget) -> String {
    let info = &opened.video.info;
    let mut out = format!("Video: {}\nAddress: {}\n", info.title, info.url);
    let first_part = from.unwrap_or(0) == 0 || full;
    if first_part {
        let mut facts = Vec::new();
        if !info.site.is_empty() {
            facts.push(format!("Site: {}", info.site));
        }
        if let Some(channel) = &info.channel {
            facts.push(format!("Channel: {channel}"));
        }
        if let Some(date) = &info.published {
            facts.push(format!("Published: {date}"));
        }
        if let Some(duration) = info.duration {
            facts.push(format!("Duration: {}", clock(duration)));
        }
        if let Some(views) = info.views {
            facts.push(format!("Views: {views}"));
        }
        if !facts.is_empty() {
            out.push_str(&facts.join(" · "));
            out.push('\n');
        }
        if info.live {
            out.push_str("This is a live stream: its text may be incomplete.\n");
        }
        if let Some(thumbnail) = &info.thumbnail {
            out.push_str(&format!("Thumbnail: {thumbnail}\n"));
        }
        if !info.chapters.is_empty() {
            out.push_str("Chapters:\n");
            for chapter in &info.chapters {
                out.push_str(&format!("- [{}] {}\n", clock(chapter.start), chapter.title));
            }
        }
        if let Some(description) = &info.description {
            out.push_str(&format!("Description:\n{}\n", crate::utils::text::truncate_chars(description, 1500)));
        }
    }

    out.push('\n');
    match (&opened.track, &opened.transcript, &opened.subtitle_error) {
        (Some(track), Some(transcript), _) => {
            if first_part {
                let others = other_languages(info, track);
                out.push_str(&format!("Subtitles: {} ({}).", track.lang, kind_text(track_kind(info, track))));
                if !others.is_empty() {
                    out.push_str(&format!(" Other languages: {others}."));
                }
                out.push('\n');
            }
            let part = if full { transcript.full(budget.full_chars) } else { transcript.page(from.unwrap_or(0), budget.page_chars) };
            if part.text.is_empty() {
                let length = info.duration.map(|d| format!("; the video is {} long", clock(d))).unwrap_or_default();
                out.push_str(&format!("There is no text from {}{length}.\n", clock(from.unwrap_or(0))));
            } else {
                out.push_str(if full { "Text:\n" } else { "Text with time marks:\n" });
                out.push_str(&part.text);
                out.push('\n');
                if let Some(next) = part.next {
                    out.push_str(&format!(
                        "\nMore text follows: {}\n",
                        serde_json::json!({ "action": "video", "url": info.url, "from": clock(next) })
                    ));
                }
            }
            out.push_str(&format!("Link a moment with its time, e.g. 12:34 is {}\n", time_link(&info.url, 754)));
        }
        (Some(track), None, Some(error)) => out.push_str(&format!("The {} subtitles could not be read: {error}. Use the details above.\n", track.lang)),
        (Some(track), None, None) => out.push_str(&format!("The {} subtitles are empty. Use the details above.\n", track.lang)),
        (None, _, _) => match wanted {
            Some(lang) => out.push_str(&format!("There are no {lang} subtitles. Languages: {}.\n", Some(all_languages(info)).filter(|l| !l.is_empty()).unwrap_or_else(|| "none".into()))),
            None => out.push_str("This video has no subtitles. Use the details above.\n"),
        },
    }
    out
}

/// People's subtitles in other languages, and whether machine translations exist.
fn other_languages(info: &VideoInfo, current: &SubtitleTrack) -> String {
    let manual: Vec<&str> = info.tracks.iter().filter(|t| !t.automatic && t.lang != current.lang).map(|t| t.lang.as_str()).collect();
    let mut text = if manual.len() > 15 { format!("{} and {} more", manual[..15].join(", "), manual.len() - 15) } else { manual.join(", ") };
    if info.tracks.iter().any(|t| t.automatic && !t.lang.ends_with("-orig") && t.lang != current.lang) {
        if !text.is_empty() {
            text.push_str("; ");
        }
        text.push_str("machine translations into many languages");
    }
    text
}

fn all_languages(info: &VideoInfo) -> String {
    let mut langs: Vec<&str> = info.tracks.iter().filter(|t| !t.automatic || t.lang.ends_with("-orig")).map(|t| t.lang.as_str()).collect();
    langs.dedup();
    langs.join(", ")
}

/// Video results as a numbered list for the agent.
pub fn format_video_results(results: &[VideoSearchResult]) -> String {
    if results.is_empty() {
        return "No videos found.".to_string();
    }
    let mut out = String::new();
    for (i, r) in results.iter().enumerate() {
        out.push_str(&format!("{}. {}\n   Video: {}\n", i + 1, r.title.trim(), r.url));
        if let Some(thumbnail) = &r.thumbnail {
            out.push_str(&format!("   Thumbnail: {thumbnail}\n"));
        }
        let facts: Vec<String> = [
            r.channel.clone(),
            r.duration.map(clock),
            r.published.clone(),
            r.site.clone(),
            r.views.map(|v| format!("{v} views")),
        ]
        .into_iter()
        .flatten()
        .collect();
        if !facts.is_empty() {
            out.push_str(&format!("   {}\n", facts.join(" · ")));
        }
        if let Some(description) = r.description.as_deref().filter(|d| !d.trim().is_empty()) {
            out.push_str(&format!("   {}\n", crate::utils::text::truncate_chars(description.trim(), 200)));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::video::subtitles::Cue;
    use crate::providers::video::Chapter;

    fn info() -> VideoInfo {
        VideoInfo {
            id: "abc".into(),
            title: "A talk".into(),
            url: "https://www.youtube.com/watch?v=abc".into(),
            site: "Youtube".into(),
            channel: Some("Speaker".into()),
            channel_url: None,
            published: Some("2024-05-01".into()),
            duration: Some(600),
            views: Some(1200),
            language: Some("en".into()),
            live: false,
            thumbnail: Some("https://i.ytimg.com/vi/abc/maxresdefault.jpg".into()),
            description: Some("About things.".into()),
            chapters: vec![Chapter { start: 0, end: 300, title: "Start".into() }, Chapter { start: 300, end: 600, title: "End".into() }],
            tracks: vec![
                SubtitleTrack { lang: "en".into(), name: None, automatic: false, formats: vec!["vtt".into()] },
                SubtitleTrack { lang: "de".into(), name: None, automatic: false, formats: vec!["vtt".into()] },
                SubtitleTrack { lang: "en-orig".into(), name: None, automatic: true, formats: vec!["json3".into()] },
                SubtitleTrack { lang: "fr".into(), name: None, automatic: true, formats: vec!["json3".into()] },
            ],
        }
    }

    fn opened(transcript: bool) -> Opened {
        let info = info();
        let cues: Vec<Cue> = (0..120).map(|i| Cue { start_ms: i * 5000, text: format!("Sentence {i}.") }).collect();
        Opened {
            track: Some(info.tracks[0].clone()),
            transcript: transcript.then(|| Transcript::new(&cues, &info.chapters)),
            subtitle_error: None,
            video: Video { info, info_file: PathBuf::new() },
        }
    }

    const BUDGET: TextBudget = TextBudget { page_chars: 600, full_chars: 100_000 };

    #[test]
    fn the_first_part_tells_about_the_video() {
        let text = describe(&opened(true), None, None, false, BUDGET);
        for expected in [
            "Video: A talk\nAddress: https://www.youtube.com/watch?v=abc",
            "Site: Youtube · Channel: Speaker · Published: 2024-05-01 · Duration: 10:00 · Views: 1200",
            "Thumbnail: https://i.ytimg.com/vi/abc/maxresdefault.jpg",
            "Chapters:\n- [0:00] Start\n- [5:00] End",
            "Description:\nAbout things.",
            "Subtitles: en (made by people). Other languages: de; machine translations into many languages.",
            "## Start\n[0:00] Sentence 0.",
            "Link a moment with its time, e.g. 12:34 is https://www.youtube.com/watch?v=abc&t=754s",
        ] {
            assert!(text.contains(expected), "{expected:?} in\n{text}");
        }
        assert!(text.contains(r#"More text follows: {"action":"video","from":"#), "{text}");
    }

    #[test]
    fn later_parts_skip_the_details() {
        let text = describe(&opened(true), None, Some(300), false, BUDGET);
        assert!(text.starts_with("Video: A talk\nAddress: https://www.youtube.com/watch?v=abc\n"));
        assert!(!text.contains("Chapters:") && !text.contains("Subtitles:"));
        assert!(text.contains("[5:00] Sentence 60."), "{text}");

        let past = describe(&opened(true), None, Some(5000), false, BUDGET);
        assert!(past.contains("There is no text from 1:23:20; the video is 10:00 long."), "{past}");
    }

    #[test]
    fn the_whole_text_at_once() {
        let text = describe(&opened(true), None, None, true, BUDGET);
        assert!(text.contains("Sentence 0.") && text.contains("Sentence 119."));
        assert!(!text.contains("More text follows"));
    }

    #[test]
    fn without_subtitles_the_details_remain() {
        let mut none = opened(false);
        none.track = None;
        assert!(describe(&none, None, None, false, BUDGET).contains("This video has no subtitles. Use the details above."));
        assert!(describe(&none, Some("ja"), None, false, BUDGET).contains("There are no ja subtitles. Languages: en, de, en-orig."));

        let mut failed = opened(false);
        failed.subtitle_error = Some("the site is limiting requests (HTTP 429); try another video".into());
        assert!(describe(&failed, None, None, false, BUDGET).contains("The en subtitles could not be read: the site is limiting requests"));
    }

    #[test]
    fn video_results_for_the_agent() {
        let results = vec![VideoSearchResult {
            url: "https://www.youtube.com/watch?v=abc".into(),
            title: " A talk ".into(),
            thumbnail: Some("https://i.ytimg.com/vi/abc/hq.jpg".into()),
            channel: Some("Speaker".into()),
            site: Some("YouTube".into()),
            duration: Some(3723),
            published: Some("2024-05-01".into()),
            views: Some(5),
            description: Some("About things.".into()),
        }];
        assert_eq!(
            format_video_results(&results),
            "1. A talk\n   Video: https://www.youtube.com/watch?v=abc\n   Thumbnail: https://i.ytimg.com/vi/abc/hq.jpg\n   Speaker · 1:02:03 · 2024-05-01 · YouTube · 5 views\n   About things.\n"
        );
        assert_eq!(format_video_results(&[]), "No videos found.");
    }
}
