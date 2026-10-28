//! Running yt-dlp for a video's details: the arguments it gets, how its JSON is read, and how
//! its failures become errors the research agent can act on. A fake yt-dlp records its
//! arguments and prints answers recorded from the real one.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use query2table_lib::components::{JsRuntime, Tools};
use query2table_lib::providers::accounting::{BudgetExceeded, PriceQuote, RequestInfo, UsageObserver};
use query2table_lib::providers::llm::types::LlmUsage;
use query2table_lib::providers::video::{Cookies, VideoError, YtDlp};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video").join(name)
}

/// A yt-dlp that writes its arguments to `args.txt`, one a line, and runs `body`.
fn fake_ytdlp(dir: &Path, body: &str) -> Tools {
    use std::os::unix::fs::PermissionsExt;
    let program = dir.join("yt-dlp");
    let args = dir.join("args.txt");
    std::fs::write(&program, format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n{body}\n", args.display())).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
    Tools { program, prefix: Vec::new(), env: Vec::new(), js_runtime: JsRuntime::Deno(dir.join("deno")) }
}

fn printing(dir: &Path, fixture_name: &str) -> Tools {
    fake_ytdlp(dir, &format!("cat '{}'", fixture(fixture_name).display()))
}

fn failing(dir: &Path, stderr: &str) -> Tools {
    fake_ytdlp(dir, &format!("printf '%s\\n' 'WARNING: something minor' '{stderr}' >&2\nexit 1"))
}

fn args(dir: &Path) -> Vec<String> {
    std::fs::read_to_string(dir.join("args.txt")).unwrap().lines().map(String::from).collect()
}

fn ytdlp(dir: &Path, tools: Tools) -> YtDlp {
    YtDlp::new(tools, dir.join("work")).with_proxy(None)
}

#[tokio::test]
async fn details_of_a_youtube_video() {
    let dir = tempfile::tempdir().unwrap();
    let video = ytdlp(dir.path(), printing(dir.path(), "youtube.info.json")).info("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await.unwrap();
    let info = &video.info;

    assert_eq!(info.id, "dQw4w9WgXcQ");
    assert_eq!(info.title, "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)");
    assert_eq!(info.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    assert_eq!(info.site, "Youtube");
    assert_eq!(info.channel.as_deref(), Some("Rick Astley"));
    assert_eq!(info.duration, Some(213));
    assert_eq!(info.published.as_deref(), Some("2009-10-25"));
    assert_eq!(info.language.as_deref(), Some("en"));
    assert!(info.views.unwrap() > 1_000_000_000);
    assert!(!info.live);
    assert_eq!(info.thumbnail.as_deref(), Some("https://i.ytimg.com/vi_webp/dQw4w9WgXcQ/maxresdefault.webp"));
    assert!(info.description.as_deref().unwrap().contains("Rick Astley"));

    let manual = info.tracks.iter().find(|t| t.lang == "en" && !t.automatic).unwrap();
    assert_eq!(manual.name.as_deref(), Some("English"));
    assert!(manual.formats.contains(&"json3".to_string()));
    let original = info.tracks.iter().find(|t| t.lang == "en-orig").unwrap();
    assert!(original.automatic);
    // Automatic tracks in other languages are YouTube's machine translations.
    assert!(info.tracks.iter().any(|t| t.lang == "ru" && t.automatic));
    assert!(info.tracks.iter().all(|t| t.lang != "live_chat"));

    // The full answer is kept for fetching subtitles later without asking the site again.
    let saved: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&video.info_file).unwrap()).unwrap();
    assert_eq!(saved["id"], "dQw4w9WgXcQ");
    assert!(video.info_file.starts_with(dir.path().join("work")));
}

#[tokio::test]
async fn chapters_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let info = ytdlp(dir.path(), printing(dir.path(), "youtube_chapters.info.json")).info("https://www.youtube.com/watch?v=ThjvMReOXYM").await.unwrap().info;
    assert_eq!(info.chapters.len(), 24);
    assert_eq!((info.chapters[0].start, info.chapters[0].end, info.chapters[0].title.as_str()), (0, 289, "Introduction"));
    assert_eq!(info.chapters[1].title, "Fixing the video title");
    // A named manual track keeps its full key.
    assert!(info.tracks.iter().any(|t| t.lang.starts_with("en-") && !t.automatic));
}

#[tokio::test]
async fn details_from_another_site() {
    let dir = tempfile::tempdir().unwrap();
    let info = ytdlp(dir.path(), printing(dir.path(), "ted.info.json"))
        .info("https://www.ted.com/talks/sir_ken_robinson_do_schools_kill_creativity")
        .await
        .unwrap()
        .info;
    assert_eq!(info.site, "TedTalk");
    assert_eq!(info.title, "Do schools kill creativity?");
    assert_eq!(info.channel.as_deref(), Some("Sir Ken Robinson"));
    assert_eq!(info.duration, Some(1151));
    assert_eq!(info.published.as_deref(), Some("2006-06-27"));
    assert!(info.thumbnail.is_some());
    let en = info.tracks.iter().find(|t| t.lang == "en").unwrap();
    assert!(!en.automatic);
    assert_eq!(en.formats, ["vtt"]);
    assert!(info.tracks.iter().all(|t| !t.automatic));
}

#[tokio::test]
async fn ytdlp_gets_the_proxy_the_js_engine_and_only_this_video() {
    let dir = tempfile::tempdir().unwrap();
    let tools = printing(dir.path(), "ted.info.json");
    YtDlp::new(tools, dir.path().join("work"))
        .with_proxy(Some("http://127.0.0.1:8888".into()))
        .info("https://www.ted.com/talks/x")
        .await
        .unwrap();
    let args = args(dir.path());
    let pair = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    assert_eq!(pair("--proxy").as_deref(), Some("http://127.0.0.1:8888"));
    assert!(args.contains(&"--no-js-runtimes".to_string()));
    assert_eq!(pair("--js-runtimes"), Some(format!("deno:{}", dir.path().join("deno").display())));
    for flag in ["--ignore-config", "--no-playlist", "--skip-download", "--dump-single-json"] {
        assert!(args.contains(&flag.to_string()), "{flag} in {args:?}");
    }
    // The address comes last, after "--", so it is never read as an option.
    assert_eq!(&args[args.len() - 2..], ["--", "https://www.ted.com/talks/x"]);
    assert!(pair("--cookies-from-browser").is_none() && pair("--cookies").is_none());
}

#[tokio::test]
async fn no_proxy_means_a_direct_connection() {
    let dir = tempfile::tempdir().unwrap();
    ytdlp(dir.path(), printing(dir.path(), "ted.info.json")).info("https://www.ted.com/talks/x").await.unwrap();
    let args = args(dir.path());
    let i = args.iter().position(|a| a == "--proxy").unwrap();
    assert_eq!(args[i + 1], "");
}

#[tokio::test]
async fn cookies_come_from_a_browser_or_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let tools = printing(dir.path(), "ted.info.json");
    ytdlp(dir.path(), tools.clone()).with_cookies(Some(Cookies::Browser("firefox".into()))).info("https://www.ted.com/talks/x").await.unwrap();
    let args1 = args(dir.path());
    let i = args1.iter().position(|a| a == "--cookies-from-browser").unwrap();
    assert_eq!(args1[i + 1], "firefox");

    let file = dir.path().join("cookies.txt");
    ytdlp(dir.path(), tools).with_cookies(Some(Cookies::File(file.clone()))).info("https://www.ted.com/talks/x").await.unwrap();
    let args2 = args(dir.path());
    let i = args2.iter().position(|a| a == "--cookies").unwrap();
    assert_eq!(args2[i + 1], file.display().to_string());
}

/// Android: Python runs the zipapp, which comes first among the arguments.
#[tokio::test]
async fn a_prefix_comes_before_the_arguments() {
    let dir = tempfile::tempdir().unwrap();
    let mut tools = printing(dir.path(), "ted.info.json");
    tools.prefix = vec!["/data/app/yt-dlp".into()];
    tools.js_runtime = JsRuntime::QuickJs("/lib/libqjs.so".into());
    ytdlp(dir.path(), tools).info("https://www.ted.com/talks/x").await.unwrap();
    let args = args(dir.path());
    assert_eq!(args[0], "/data/app/yt-dlp");
    let i = args.iter().position(|a| a == "--js-runtimes").unwrap();
    assert_eq!(args[i + 1], "quickjs:/lib/libqjs.so");
}

#[tokio::test]
async fn failures_are_told_apart() {
    type Check = fn(&VideoError) -> bool;
    let cases: [(&str, Check); 6] = [
        ("ERROR: [youtube] jNQXAC9IVRw: Sign in to confirm you’re not a bot. Use --cookies-from-browser or --cookies for the authentication.", |e| matches!(e, VideoError::LoginRequired(_))),
        ("ERROR: [vimeo] 76979871: The web client only works when logged-in. Use --cookies, --cookies-from-browser", |e| matches!(e, VideoError::LoginRequired(_))),
        ("ERROR: Unsupported URL: https://example.com/page", |e| matches!(e, VideoError::Unsupported)),
        ("ERROR: Unable to download JSON metadata: HTTP Error 429: Too Many Requests", |e| matches!(e, VideoError::RateLimited)),
        ("ERROR: [youtube] abc: Video unavailable. This video is private", |e| matches!(e, VideoError::Unavailable(_))),
        ("ERROR: [youtube] abc: The uploader has not made this video available in your country", |e| matches!(e, VideoError::GeoBlocked)),
    ];
    for (stderr, expected) in cases {
        let dir = tempfile::tempdir().unwrap();
        let error = ytdlp(dir.path(), failing(dir.path(), stderr)).info("https://www.youtube.com/watch?v=abc").await.unwrap_err();
        assert!(expected(&error), "{stderr} gave {error:?}");
        assert!(!error.to_string().contains("WARNING"), "{error}");
    }

    let dir = tempfile::tempdir().unwrap();
    let error = ytdlp(dir.path(), failing(dir.path(), "ERROR: [generic] something odd happened")).info("https://example.com/v").await.unwrap_err();
    assert!(matches!(error, VideoError::Failed(ref m) if m == "[generic] something odd happened"), "{error:?}");
}

#[tokio::test]
async fn a_hanging_ytdlp_is_stopped() {
    let dir = tempfile::tempdir().unwrap();
    let started = std::time::Instant::now();
    let error = ytdlp(dir.path(), fake_ytdlp(dir.path(), "sleep 20")).with_timeout(Duration::from_millis(300)).info("https://www.youtube.com/watch?v=abc").await.unwrap_err();
    assert!(matches!(error, VideoError::Timeout), "{error:?}");
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[tokio::test]
async fn only_web_addresses_are_read() {
    let dir = tempfile::tempdir().unwrap();
    let ytdlp = ytdlp(dir.path(), printing(dir.path(), "ted.info.json"));
    for address in ["file:///etc/passwd", "--exec=rm", "not a url", "ftp://example.com/v"] {
        assert!(matches!(ytdlp.info(address).await, Err(VideoError::BadAddress)), "{address}");
    }
    assert!(!dir.path().join("args.txt").exists(), "yt-dlp must not run");
}

#[derive(Default)]
struct Downloads(AtomicU64);

impl UsageObserver for Downloads {
    fn exceeded(&self) -> bool {
        false
    }
    fn begin(&self, _request: RequestInfo) -> Result<u64, BudgetExceeded> {
        Ok(0)
    }
    fn capture(&self, _id: u64, _usage: LlmUsage) {}
    fn finish(&self, _id: u64, _pricing: Option<PriceQuote>) {}
    fn downloaded(&self, bytes: u64) {
        self.0.fetch_add(bytes, Ordering::Relaxed);
    }
}

#[tokio::test]
async fn what_ytdlp_brings_is_counted_as_downloaded() {
    let dir = tempfile::tempdir().unwrap();
    let downloads = Arc::new(Downloads::default());
    ytdlp(dir.path(), printing(dir.path(), "youtube.info.json")).with_observer(downloads.clone()).info("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await.unwrap();
    assert_eq!(downloads.0.load(Ordering::Relaxed), std::fs::metadata(fixture("youtube.info.json")).unwrap().len());
}

/// The real yt-dlp and Deno, kept between runs in the temp folder, through `Q2T_VIDEO_PROXY`
/// when set: `Q2T_VIDEO_PROXY=http://127.0.0.1:8888 cargo test --test video_ytdlp live -- --ignored`.
async fn live_ytdlp() -> YtDlp {
    let dir = std::env::temp_dir().join("q2t-live-tools");
    let tools = query2table_lib::components::ComponentManager::new(dir.clone()).unwrap().ensure(&|_| {}).await.unwrap();
    YtDlp::new(tools, dir.join("work")).with_proxy(std::env::var("Q2T_VIDEO_PROXY").ok())
}

#[tokio::test]
#[ignore = "network"]
async fn live_youtube_details() {
    let video = live_ytdlp().await.info("https://www.youtube.com/watch?v=jNQXAC9IVRw").await.unwrap();
    assert_eq!(video.info.title, "Me at the zoo");
    assert_eq!(video.info.duration, Some(19));
    assert!(video.info.thumbnail.is_some());
    assert!(video.info.tracks.iter().any(|t| t.lang.starts_with("en")), "{:?}", video.info.tracks);
}

#[tokio::test]
#[ignore = "network"]
async fn live_unsupported_page() {
    let error = live_ytdlp().await.info("https://example.com/").await.unwrap_err();
    assert!(matches!(error, VideoError::Unsupported | VideoError::Failed(_)), "{error:?}");
}

/// Details first, then one subtitle track from the saved details: the fake writes the file
/// yt-dlp would.
#[tokio::test]
async fn subtitles_are_fetched_from_the_saved_details() {
    let dir = tempfile::tempdir().unwrap();
    let work = dir.path().join("work");
    let body = format!(
        "case \"$*\" in\n  *--load-info-json*) cp '{}' '{}/Youtube-dQw4w9WgXcQ.en-orig.json3' ;;\n  *) cat '{}' ;;\nesac",
        fixture("youtube.en-orig.json3").display(),
        work.display(),
        fixture("youtube.info.json").display()
    );
    let downloads = Arc::new(Downloads::default());
    let ytdlp = ytdlp(dir.path(), fake_ytdlp(dir.path(), &body)).with_observer(downloads.clone());
    let video = ytdlp.info("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await.unwrap();
    let track = video.info.tracks.iter().find(|t| t.lang == "en-orig").unwrap();

    let cues = ytdlp.subtitles(&video, track).await.unwrap();
    assert_eq!(cues[0].text, "[Music]");
    assert!(cues.len() > 20);

    let args = args(dir.path());
    let pair = |flag: &str| args.iter().position(|a| a == flag).map(|i| args[i + 1].clone());
    assert_eq!(pair("--load-info-json"), Some(video.info_file.display().to_string()));
    assert!(args.contains(&"--write-auto-subs".to_string()) && !args.contains(&"--write-subs".to_string()));
    assert_eq!(pair("--sub-langs").as_deref(), Some("en\\-orig"));
    assert_eq!(pair("--sub-format").as_deref(), Some("json3/vtt/srt/best"));
    assert_eq!(pair("-o"), Some(work.join("Youtube-dQw4w9WgXcQ.%(ext)s").display().to_string()));
    // The site is not asked again.
    assert!(!args.iter().any(|a| a.starts_with("https://")));
    let expected = std::fs::metadata(fixture("youtube.info.json")).unwrap().len() + std::fs::metadata(fixture("youtube.en-orig.json3")).unwrap().len();
    assert_eq!(downloads.0.load(Ordering::Relaxed), expected);
}

#[tokio::test]
async fn a_track_that_yields_no_file_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let body = format!("case \"$*\" in\n  *--load-info-json*) echo 'ERROR: Unable to download video subtitles for '\"'\"'ru'\"'\"': HTTP Error 429: Too Many Requests' >&2; exit 1 ;;\n  *) cat '{}' ;;\nesac", fixture("youtube.info.json").display());
    let ytdlp = ytdlp(dir.path(), fake_ytdlp(dir.path(), &body));
    let video = ytdlp.info("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await.unwrap();
    let track = video.info.tracks.iter().find(|t| t.lang == "ru").unwrap();
    assert!(matches!(ytdlp.subtitles(&video, track).await, Err(VideoError::RateLimited)));
}

#[tokio::test]
#[ignore = "network"]
async fn live_youtube_subtitles() {
    let ytdlp = live_ytdlp().await;
    let video = ytdlp.info("https://www.youtube.com/watch?v=dQw4w9WgXcQ").await.unwrap();
    let track = query2table_lib::providers::video::subtitles::choose_track(&video.info, None).unwrap();
    let cues = ytdlp.subtitles(&video, track).await.unwrap();
    let text = query2table_lib::providers::video::subtitles::Transcript::new(&cues, &video.info.chapters).full(usize::MAX).text;
    println!("{} ({}): {}", track.lang, track.automatic, text.chars().take(300).collect::<String>());
    assert!(text.to_lowercase().contains("strangers to love"), "{text}");
}

/// The user's own yt-dlp without our Deno: yt-dlp keeps its own choice of JavaScript engine.
#[tokio::test]
async fn a_system_js_engine_adds_no_engine_options() {
    let dir = tempfile::tempdir().unwrap();
    let mut tools = printing(dir.path(), "ted.info.json");
    tools.js_runtime = JsRuntime::System;
    ytdlp(dir.path(), tools).info("https://www.ted.com/talks/x").await.unwrap();
    let args = args(dir.path());
    assert!(!args.iter().any(|a| a.contains("js-runtimes")), "{args:?}");
}
