//! Videos in research: the agent searches videos on every site and on one site, reads a
//! video's details and subtitles a part at a time, and answers with a video card.
#![cfg(unix)]

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use query2table_lib::components::{JsRuntime, Tools};
use query2table_lib::orchestrator::pipeline::{PipelineConfig, PipelineState};
use query2table_lib::orchestrator::research_pipeline::ResearchPipeline;
use query2table_lib::providers::search::manager::SearchManager;
use query2table_lib::providers::search::types::{SearchError, SearchQuery, VideoSearchProvider, VideoSearchResult};
use query2table_lib::storage::{db::Database, repository::Repository};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::MockSearchProvider;

const VIDEO: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
const THUMBNAIL: &str = "https://i.ytimg.com/vi_webp/dQw4w9WgXcQ/maxresdefault.webp";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video").join(name)
}

fn reply(content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": content.to_string()}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }))
}

async fn script(server: &MockServer, replies: &[Value]) {
    for (i, content) in replies.iter().enumerate() {
        let mock = Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(reply(content.clone())).with_priority(i as u8 + 1);
        let mock = if i + 1 < replies.len() { mock.up_to_n_times(1) } else { mock };
        mock.mount(server).await;
    }
}

async fn completions(server: &MockServer) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/v1/chat/completions")
        .map(|r| r.body_json().unwrap())
        .collect()
}

fn last_text(request: &Value) -> String {
    let message = request["messages"].as_array().unwrap().last().unwrap();
    match &message["content"] {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join("\n"),
        _ => String::new(),
    }
}

async fn repo() -> Arc<Repository> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    Arc::new(Repository::new(pool))
}

fn config(server: &MockServer, extra: &[(&str, &str)]) -> PipelineConfig {
    let mut settings: HashMap<String, String> = HashMap::from([
        ("llm_provider".into(), "openai_compatible".into()),
        ("openai_base_url".into(), format!("{}/v1", server.uri())),
        ("openai_model".into(), "test-model".into()),
        ("brave_api_key".into(), "test-key".into()),
        ("rate_limit_per_domain_ms".into(), "10".into()),
        ("llm_vision".into(), "off".into()),
    ]);
    for (k, v) in extra {
        settings.insert(k.to_string(), v.to_string());
    }
    PipelineConfig::from_settings(&settings)
}

struct Videos(Vec<VideoSearchResult>);

#[async_trait]
impl VideoSearchProvider for Videos {
    async fn search_videos(&self, _query: SearchQuery) -> Result<Vec<VideoSearchResult>, SearchError> {
        Ok(self.0.clone())
    }
}

struct Failing;

#[async_trait]
impl VideoSearchProvider for Failing {
    async fn search_videos(&self, _query: SearchQuery) -> Result<Vec<VideoSearchResult>, SearchError> {
        Err(SearchError::RequestFailed("videos are not in this plan".into()))
    }
}

fn web_result() -> VideoSearchResult {
    VideoSearchResult {
        url: VIDEO.into(),
        title: "Rick Astley - Never Gonna Give You Up".into(),
        thumbnail: Some("https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg".into()),
        channel: Some("Rick Astley".into()),
        site: Some("YouTube".into()),
        duration: Some(213),
        published: Some("2009-10-25".into()),
        views: None,
        description: None,
    }
}

fn search(provider: Arc<dyn VideoSearchProvider>) -> SearchManager {
    SearchManager::with_providers(Arc::new(MockSearchProvider::empty()), None, Default::default()).with_video_providers(provider, None)
}

/// A yt-dlp answering from recordings: a site search, a video's details, or its English
/// subtitles written where `-o` says. Every call's arguments are appended to `calls.txt`.
fn fake_ytdlp(dir: &Path) -> Tools {
    fake_ytdlp_with(dir, &fixture("youtube.info.json"))
}

/// The recorded details with the thumbnail served by `server`.
fn info_with_thumbnail(dir: &Path, thumbnail: &str) -> PathBuf {
    let info = std::fs::read_to_string(fixture("youtube.info.json")).unwrap().replace(THUMBNAIL, thumbnail);
    let path = dir.join("info.json");
    std::fs::write(&path, info).unwrap();
    path
}

fn png(r: u8, g: u8, b: u8) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(64, 36, image::Rgb([r, g, b])));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn fake_ytdlp_with(dir: &Path, info: &Path) -> Tools {
    use std::os::unix::fs::PermissionsExt;
    let program = dir.join("yt-dlp");
    let body = format!(
        r#"#!/bin/sh
echo "$*" >> '{calls}'
prev=""; out=""
for a in "$@"; do [ "$prev" = "-o" ] && out="$a"; prev="$a"; done
case "$*" in
  *--load-info-json*) cp '{subs}' "$(dirname "$out")/Youtube-dQw4w9WgXcQ.en.json3" ;;
  *--flat-playlist*) cat '{search}' ;;
  *) cat '{info}' ;;
esac
"#,
        calls = dir.join("calls.txt").display(),
        subs = fixture("youtube.en.json3").display(),
        search = fixture("youtube_search.json").display(),
        info = info.display(),
    );
    std::fs::write(&program, body).unwrap();
    std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
    Tools { program, prefix: Vec::new(), env: Vec::new(), js_runtime: JsRuntime::Deno(dir.join("deno")) }
}

fn calls(dir: &Path) -> Vec<String> {
    std::fs::read_to_string(dir.join("calls.txt")).unwrap_or_default().lines().map(String::from).collect()
}

#[tokio::test]
async fn the_agent_finds_reads_and_shows_videos() {
    let server = MockServer::start().await;
    let u = server.uri();
    let thumbnail = format!("{u}/vi/maxres.png");
    Mock::given(method("GET")).and(path("/vi/maxres.png")).respond_with(ResponseTemplate::new(200).set_body_bytes(png(200, 30, 30)).insert_header("content-type", "image/png")).mount(&server).await;
    let tools_dir = tempfile::tempdir().unwrap();
    let answer = format!(
        "The song:\n\n[![Never Gonna Give You Up]({thumbnail})]({VIDEO})\n\nThe chorus starts at [0:43]({VIDEO}&t=43s).\n\n[![Made up](https://example.com/x.jpg)](https://www.youtube.com/watch?v=madeup)"
    );
    script(
        &server,
        &[
            json!({"action": "video_search", "query": "never gonna give you up"}),
            json!({"action": "video_search", "query": "rust async tutorial", "site": "youtube"}),
            json!({"action": "video", "url": VIDEO}),
            json!({"action": "video", "url": VIDEO, "from": "1:00"}),
            json!({"action": "answer", "markdown": answer}),
        ],
    )
    .await;

    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r1".into(), "Find the song video".into(), config(&server, &[]), repo.clone(), None);
    let tools = fake_ytdlp_with(tools_dir.path(), &info_with_thumbnail(tools_dir.path(), &thumbnail));
    let state = pipeline.with_search(search(Arc::new(Videos(vec![web_result()])))).with_video_tools(tools).run().await.unwrap();
    assert_eq!(state, PipelineState::Completed);

    let sent = completions(&server).await;
    let system = sent[0]["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains(r#"{"action": "video_search""#) && system.contains(r#"{"action": "video", "url""#), "{system}");

    let web = last_text(&sent[1]);
    assert!(web.starts_with("Video results:\n1. Rick Astley - Never Gonna Give You Up\n"), "{web}");
    assert!(web.contains(&format!("Video: {VIDEO}")) && web.contains("Thumbnail: https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg"), "{web}");
    assert!(web.contains("Rick Astley · 3:33 · 2009-10-25 · YouTube"), "{web}");

    let site = last_text(&sent[2]);
    assert!(site.contains("1. Async Rust explained in 20 minutes") && site.contains("Let's Get Rusty · 19:15"), "{site}");

    let read = last_text(&sent[3]);
    assert!(read.starts_with("Video: Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)\n"), "{read}");
    assert!(read.contains(&format!("Thumbnail: {thumbnail}")), "{read}");
    assert!(read.contains("Subtitles: en (made by people)."), "{read}");
    assert!(read.contains("[0:01] [♪♪♪] ♪ We're no strangers to love ♪"), "{read}");

    let later = last_text(&sent[4]);
    assert!(later.starts_with(&format!("Video: Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)\nAddress: {VIDEO}\n")), "{later}");
    assert!(!later.contains("Subtitles:"), "{later}");

    // The details and subtitles were fetched once: the later part came from the first read.
    let calls = calls(tools_dir.path());
    assert_eq!(calls.iter().filter(|c| c.contains("--dump-single-json") && !c.contains("--flat-playlist")).count(), 1, "{calls:#?}");
    assert_eq!(calls.iter().filter(|c| c.contains("--load-info-json")).count(), 1, "{calls:#?}");
    assert!(calls.iter().any(|c| c.ends_with("-- ytsearch10:rust async tutorial")), "{calls:#?}");

    let steps = repo.get_research_steps("r1").await.unwrap();
    let kinds: Vec<(&str, &str, Option<&str>)> = steps.iter().map(|s| (s.step_type.as_str(), s.content.as_str(), s.url.as_deref())).collect();
    assert_eq!(
        kinds,
        [
            ("video_search", "never gonna give you up", None),
            ("video_search", "rust async tutorial (youtube)", None),
            ("video", "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)", Some(VIDEO)),
            ("video", "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster) · from 1:00", Some(VIDEO)),
        ]
    );

    // The card's thumbnail is kept with the video's details; the made-up card is not downloaded.
    let kept = repo.get_research_images("r1").await.unwrap();
    assert_eq!(kept.len(), 1, "{kept:?}");
    let card = &kept[0];
    assert_eq!(card.url, thumbnail);
    assert_eq!(card.video_url.as_deref(), Some(VIDEO));
    assert_eq!(card.title, "Rick Astley - Never Gonna Give You Up (Official Video) (4K Remaster)");
    assert_eq!(card.channel.as_deref(), Some("Rick Astley"));
    assert_eq!(card.duration, Some(213));
    assert_eq!(card.site.as_deref(), Some("Youtube"));
    assert_eq!((card.width, card.height), (64, 36));
    let logs = repo.get_run_logs("r1", 100).await.unwrap();
    assert!(logs.iter().any(|l| l.message.contains("did not find or read") && l.message.contains("madeup")), "{:?}", logs.iter().map(|l| &l.message).collect::<Vec<_>>());
    let requested: Vec<String> = server.received_requests().await.unwrap().iter().map(|r| r.url.to_string()).collect();
    assert!(!requested.iter().any(|r| r.contains("example.com")));
}

#[tokio::test]
async fn a_failed_web_video_search_falls_back_to_youtube() {
    let server = MockServer::start().await;
    let tools_dir = tempfile::tempdir().unwrap();
    script(&server, &[json!({"action": "video_search", "query": "rust async"}), json!({"action": "answer", "markdown": "Done"})]).await;
    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r2".into(), "q".into(), config(&server, &[]), repo.clone(), None);
    pipeline.with_search(search(Arc::new(Failing))).with_video_tools(fake_ytdlp(tools_dir.path())).run().await.unwrap();

    let sent = completions(&server).await;
    let reply = last_text(&sent[1]);
    assert!(reply.contains("The web video search failed (Search request failed: videos are not in this plan); these are YouTube results:"), "{reply}");
    assert!(reply.contains("1. Async Rust explained in 20 minutes"), "{reply}");
}

#[tokio::test]
async fn a_video_that_cannot_be_read_is_an_error_step() {
    let server = MockServer::start().await;
    let tools_dir = tempfile::tempdir().unwrap();
    script(&server, &[json!({"action": "video", "url": "file:///etc/passwd"}), json!({"action": "answer", "markdown": "Done"})]).await;
    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r3".into(), "q".into(), config(&server, &[]), repo.clone(), None);
    pipeline.with_search(search(Arc::new(Videos(vec![])))).with_video_tools(fake_ytdlp(tools_dir.path())).run().await.unwrap();

    let sent = completions(&server).await;
    assert_eq!(last_text(&sent[1]), "Could not read the video: only http(s) video addresses can be read");
    let steps = repo.get_research_steps("r3").await.unwrap();
    assert_eq!(steps[0].step_type, "error");
    assert!(steps[0].content.contains("file:///etc/passwd"));
    assert!(calls(tools_dir.path()).is_empty());
}

#[tokio::test]
async fn videos_can_be_turned_off() {
    let server = MockServer::start().await;
    script(&server, &[json!({"action": "video_search", "query": "q"}), json!({"action": "answer", "markdown": "Done"})]).await;
    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r4".into(), "q".into(), config(&server, &[("research_videos", "false")]), repo, None);
    pipeline.with_search(search(Arc::new(Videos(vec![web_result()])))).run().await.unwrap();

    let sent = completions(&server).await;
    assert!(!sent[0]["messages"][0]["content"].as_str().unwrap().contains("video_search"));
    assert_eq!(last_text(&sent[1]), "Videos are turned off for this run.");
}
