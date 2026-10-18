//! Images in research answers: image search, looking at images, images on read pages, and
//! keeping the answer's pictures with the run.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use query2table_lib::orchestrator::pipeline::{PipelineConfig, PipelineState};
use query2table_lib::orchestrator::research_pipeline::ResearchPipeline;
use query2table_lib::providers::search::manager::SearchManager;
use query2table_lib::providers::search::types::{ImageSearchProvider, ImageSearchResult, SearchError, SearchQuery};
use query2table_lib::storage::{db::Database, repository::Repository};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::MockSearchProvider;

fn reply(content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": content.to_string()}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }))
}

fn png(r: u8, g: u8, b: u8) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(40, 30, image::Rgb([r, g, b])));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

struct Images(Vec<ImageSearchResult>);

#[async_trait]
impl ImageSearchProvider for Images {
    async fn search_images(&self, _query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError> {
        Ok(self.0.clone())
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
    ]);
    for (k, v) in extra {
        settings.insert(k.to_string(), v.to_string());
    }
    PipelineConfig::from_settings(&settings)
}

fn search(results: Vec<ImageSearchResult>) -> SearchManager {
    SearchManager::with_providers(Arc::new(MockSearchProvider::empty()), None, Default::default())
        .with_image_provider(Arc::new(Images(results)))
}

/// Replies the model gives in order; the last one repeats.
async fn script(server: &MockServer, replies: &[Value]) {
    for (i, content) in replies.iter().enumerate() {
        let mock = Mock::given(method("POST")).and(path("/v1/chat/completions")).respond_with(reply(content.clone())).with_priority(i as u8 + 1);
        let mock = if i + 1 < replies.len() { mock.up_to_n_times(1) } else { mock };
        mock.mount(server).await;
    }
}

async fn serve(server: &MockServer, at: &str, bytes: Vec<u8>, content_type: &str) {
    Mock::given(method("GET"))
        .and(path(at))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(bytes).insert_header("content-type", content_type))
        .mount(server)
        .await;
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

/// The last message of a request: its text and how many pictures it carries.
fn last_message(request: &Value) -> (String, usize) {
    let message = request["messages"].as_array().unwrap().last().unwrap();
    match &message["content"] {
        Value::String(s) => (s.clone(), 0),
        Value::Array(parts) => (
            parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join("\n"),
            parts.iter().filter(|p| p["type"] == "image_url").count(),
        ),
        _ => (String::new(), 0),
    }
}

#[tokio::test]
async fn the_agent_finds_looks_at_and_keeps_images() {
    let server = MockServer::start().await;
    let u = server.uri();
    serve(&server, "/img/panda.png", png(200, 60, 20), "image/png").await;
    serve(&server, "/thumb/panda.png", png(200, 60, 20), "image/png").await;
    serve(&server, "/thumb/cat.png", png(90, 90, 90), "image/png").await;
    serve(&server, "/img/fox.png", png(220, 120, 0), "image/png").await;
    serve(
        &server,
        "/zoo/animals.html",
        b"<html><head><title>Zoo animals</title></head><body><p>Foxes live here.</p><img src=\"../img/fox.png\" alt=\"A fox\"></body></html>".to_vec(),
        "text/html",
    )
    .await;
    let results = vec![
        ImageSearchResult {
            image_url: format!("{u}/img/panda.png"),
            thumbnail_url: format!("{u}/thumb/panda.png"),
            title: "Red panda".into(),
            source_url: format!("{u}/pandas"),
            width: Some(800),
            height: Some(600),
        },
        // The original is gone; its thumbnail is kept instead.
        ImageSearchResult {
            image_url: format!("{u}/img/gone.png"),
            thumbnail_url: format!("{u}/thumb/cat.png"),
            title: "Grey cat".into(),
            source_url: format!("{u}/cats"),
            width: None,
            height: None,
        },
    ];
    let answer = format!(
        "A red panda:\n\n![Red panda]({u}/img/panda.png)\n\nA fox ![Fox]({u}/img/fox.png) and a cat ![Cat]({u}/img/gone.png). Made up: ![X]({u}/img/invented.png)"
    );
    script(
        &server,
        &[
            json!({"action": "image_search", "query": "red panda"}),
            json!({"action": "view_image", "url": format!("{u}/img/panda.png")}),
            json!({"action": "fetch", "url": format!("{u}/zoo/animals.html")}),
            json!({"action": "answer", "markdown": answer}),
        ],
    )
    .await;

    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r1".into(), "Show a red panda".into(), config(&server, &[("llm_vision", "on")]), repo.clone(), None);
    assert_eq!(pipeline.with_search(search(results)).run().await.unwrap(), PipelineState::Completed);

    let sent = completions(&server).await;
    let system = sent[0]["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains(r#"{"action": "image_search""#) && system.contains(r#"{"action": "view_image""#), "{system}");
    let (results_text, previews) = last_message(&sent[1]);
    assert!(results_text.contains("1. Red panda (800×600)") && results_text.contains(&format!("Image: {u}/img/panda.png")), "{results_text}");
    assert!(results_text.contains(&format!("Page: {u}/pandas")));
    assert!(results_text.contains("Pictures of results 1, 2 follow"), "{results_text}");
    assert_eq!(previews, 2);
    let (viewed, pictures) = last_message(&sent[2]);
    assert!(viewed.contains(&format!("Image {u}/img/panda.png (40×30 px)")), "{viewed}");
    assert_eq!(pictures, 1);
    let (page, _) = last_message(&sent[3]);
    assert!(page.contains(&format!("![A fox]({u}/img/fox.png)")), "{page}");

    let steps = repo.get_research_steps("r1").await.unwrap();
    let kinds: Vec<(&str, &str)> = steps.iter().map(|s| (s.step_type.as_str(), s.content.as_str())).collect();
    assert_eq!(kinds[..2], [("image_search", "red panda"), ("view_image", "Red panda")]);
    assert_eq!(steps[1].url.as_deref(), Some(format!("{u}/img/panda.png").as_str()));

    let kept = repo.get_research_images("r1").await.unwrap();
    let found: Vec<(&str, Option<&str>, &str)> = kept.iter().map(|k| (k.url.as_str(), k.source_url.as_deref(), k.title.as_str())).collect();
    let (panda, fox, gone) = (format!("{u}/img/panda.png"), format!("{u}/img/fox.png"), format!("{u}/img/gone.png"));
    let (pandas, zoo, cats) = (format!("{u}/pandas"), format!("{u}/zoo/animals.html"), format!("{u}/cats"));
    for expected in [(panda.as_str(), Some(pandas.as_str()), "Red panda"), (fox.as_str(), Some(zoo.as_str()), "A fox"), (gone.as_str(), Some(cats.as_str()), "Grey cat")] {
        assert!(found.contains(&expected), "{expected:?} in {found:?}");
    }
    assert_eq!(kept.len(), 3, "the made-up address is not downloaded");
    assert!(kept.iter().all(|k| k.turn_index == 0 && k.media_type == "image/jpeg" && (k.width, k.height) == (40, 30)));
    let (media_type, data) = repo.get_research_image_data("r1", &panda).await.unwrap().unwrap();
    assert_eq!(media_type, "image/jpeg");
    assert!(data.starts_with(&[0xFF, 0xD8]));
    let logs = repo.get_run_logs("r1", 100).await.unwrap();
    assert!(logs.iter().any(|l| l.message.contains("did not find or read") && l.message.contains("invented.png")));
    assert!(logs.iter().any(|l| l.message == "Kept 3 image(s) of the answer"));
    let requested: Vec<String> = server.received_requests().await.unwrap().iter().map(|r| r.url.path().to_string()).collect();
    assert!(!requested.contains(&"/img/invented.png".to_string()));

    // Deleting the run removes its pictures.
    repo.delete_run("r1").await.unwrap();
    assert!(repo.get_research_images("r1").await.unwrap().is_empty());
}

#[tokio::test]
async fn previews_and_kept_images_follow_their_limits() {
    let server = MockServer::start().await;
    let u = server.uri();
    let mut results = Vec::new();
    for name in ["a", "b", "c"] {
        serve(&server, &format!("/img/{name}.png"), png(10, 200, 10), "image/png").await;
        results.push(ImageSearchResult {
            image_url: format!("{u}/img/{name}.png"),
            thumbnail_url: format!("{u}/img/{name}.png"),
            title: format!("Picture {name}"),
            source_url: format!("{u}/page-{name}"),
            width: None,
            height: None,
        });
    }
    let answer = format!("![A]({u}/img/a.png) ![B]({u}/img/b.png) ![C]({u}/img/c.png)");
    script(&server, &[json!({"action": "image_search", "query": "pictures"}), json!({"action": "answer", "markdown": answer})]).await;
    let repo = repo().await;
    let settings = [("llm_vision", "on"), ("max_inline_images", "2"), ("research_max_images", "1")];
    let (pipeline, _c) = ResearchPipeline::new("r4".into(), "Pictures".into(), config(&server, &settings), repo.clone(), None);
    assert_eq!(pipeline.with_search(search(results)).run().await.unwrap(), PipelineState::Completed);

    // Providers may take only a few pictures per message: "Pictures per request" applies.
    let (text, previews) = last_message(&completions(&server).await[1]);
    assert_eq!(previews, 2);
    assert!(text.contains("Pictures of results 1, 2 follow"), "{text}");
    let kept: Vec<String> = repo.get_research_images("r4").await.unwrap().into_iter().map(|k| k.url).collect();
    assert_eq!(kept, vec![format!("{u}/img/a.png")]);
    let logs = repo.get_run_logs("r4", 100).await.unwrap();
    assert!(logs.iter().any(|l| l.message == "Keeping the first 1 of 3 images of the answer"), "{logs:?}");
}

#[tokio::test]
async fn a_blind_model_gets_image_search_without_looking() {
    let server = MockServer::start().await;
    let u = server.uri();
    serve(&server, "/thumb/panda.png", png(200, 60, 20), "image/png").await;
    let results = vec![ImageSearchResult {
        image_url: format!("{u}/img/panda.png"),
        thumbnail_url: format!("{u}/thumb/panda.png"),
        title: "Red panda".into(),
        source_url: format!("{u}/pandas"),
        width: None,
        height: None,
    }];
    script(
        &server,
        &[
            json!({"action": "image_search", "query": "red panda"}),
            json!({"action": "view_image", "url": format!("{u}/img/panda.png")}),
            json!({"action": "answer", "markdown": "No pictures."}),
        ],
    )
    .await;
    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new("r2".into(), "Show a red panda".into(), config(&server, &[("llm_vision", "off")]), repo.clone(), None);
    assert_eq!(pipeline.with_search(search(results)).run().await.unwrap(), PipelineState::Completed);
    let sent = completions(&server).await;
    let system = sent[0]["messages"][0]["content"].as_str().unwrap();
    assert!(system.contains("image_search") && !system.contains("view_image") && system.contains("You cannot see images"));
    let (results_text, previews) = last_message(&sent[1]);
    assert!(results_text.contains("1. Red panda") && !results_text.contains("Pictures of results"));
    assert_eq!(previews, 0);
    let (refused, pictures) = last_message(&sent[2]);
    assert!(refused.contains("You cannot look at images"), "{refused}");
    assert_eq!(pictures, 0);
    assert!(repo.get_research_steps("r2").await.unwrap().iter().all(|s| s.step_type != "view_image"));
}

#[tokio::test]
async fn the_setting_turns_images_off() {
    let server = MockServer::start().await;
    script(&server, &[json!({"action": "image_search", "query": "red panda"}), json!({"action": "answer", "markdown": "Done."})]).await;
    let repo = repo().await;
    let (pipeline, _c) = ResearchPipeline::new(
        "r3".into(),
        "Show a red panda".into(),
        config(&server, &[("llm_vision", "on"), ("research_images", "false")]),
        repo.clone(),
        None,
    );
    assert_eq!(pipeline.with_search(search(Vec::new())).run().await.unwrap(), PipelineState::Completed);
    let sent = completions(&server).await;
    let system = sent[0]["messages"][0]["content"].as_str().unwrap();
    assert!(!system.contains("image_search") && !system.contains("view_image"));
    assert!(last_message(&sent[1]).0.contains("Image search is not available"));
}
