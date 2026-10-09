//! Links and images with attached files: the files shape the searches and the ranking, and an
//! attached picture is the reference found images are compared with.

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use query2table_lib::attachments::store::AttachmentStore;
use query2table_lib::orchestrator::image_pipeline::ImagePipeline;
use query2table_lib::orchestrator::link_pipeline::LinkPipeline;
use query2table_lib::orchestrator::pipeline::{PipelineConfig, PipelineState};
use query2table_lib::providers::search::manager::SearchManager;
use query2table_lib::providers::search::types::{ImageSearchProvider, ImageSearchResult, SearchError, SearchQuery, SearchResult};
use serde_json::{json, Value};
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::*;

fn reply(content: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": content}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }))
}

fn png(r: u8, g: u8, b: u8) -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(32, 24, image::Rgb([r, g, b])));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

fn config(server: &MockServer, dir: &std::path::Path, extra: &[(&str, &str)]) -> PipelineConfig {
    let mut settings: std::collections::HashMap<String, String> = [
        ("llm_provider", "openai_compatible".to_string()),
        ("openai_base_url", format!("{}/v1", server.uri())),
        ("openai_model", "vision-model".to_string()),
        ("brave_api_key", "unused".to_string()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    for (k, v) in extra {
        settings.insert(k.to_string(), v.to_string());
    }
    let mut config = PipelineConfig::from_settings(&settings);
    config.attachments_dir = dir.to_path_buf();
    config
}

/// Completion requests with the given text, in order.
async fn completions(server: &MockServer, containing: &str) -> Vec<Value> {
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path() == "/v1/chat/completions")
        .map(|r| r.body_json::<Value>().unwrap())
        .filter(|body| body.to_string().contains(containing))
        .collect()
}

struct Images(Vec<ImageSearchResult>);

#[async_trait]
impl ImageSearchProvider for Images {
    async fn search_images(&self, _query: SearchQuery) -> Result<Vec<ImageSearchResult>, SearchError> {
        Ok(self.0.clone())
    }
}

#[tokio::test]
async fn found_images_are_compared_with_the_attached_reference() {
    let server = MockServer::start().await;
    for (name, bytes) in [("/thumb/near.png", png(200, 20, 20)), ("/thumb/far.png", png(20, 20, 200))] {
        Mock::given(method("GET")).and(path(name)).respond_with(ResponseTemplate::new(200).set_body_bytes(bytes).insert_header("content-type", "image/png")).mount(&server).await;
    }
    Mock::given(method("POST")).and(body_string_contains("image search query generator")).respond_with(reply(r#"{"queries": ["red tractor"]}"#)).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("describe images")).respond_with(reply("A red tractor in a field")).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("strict image relevance judge")).respond_with(reply("[0.8, 0.9]")).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("reference picture")).respond_with(reply("[0.2, 0.95]")).mount(&server).await;

    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::new(repo.pool().clone(), dir.path().to_path_buf());
    let reference = store.add_bytes("tractor.png", png(210, 30, 30)).await.unwrap();
    let mut config = config(&server, dir.path(), &[("llm_vision", "on")]);
    config.attachments = vec![reference.id.clone()];
    let candidates = vec![
        ImageSearchResult { image_url: format!("{}/full/near.png", server.uri()), thumbnail_url: format!("{}/thumb/near.png", server.uri()), title: "Tractor".into(), source_url: "https://a.example".into(), width: None, height: None },
        ImageSearchResult { image_url: format!("{}/full/far.png", server.uri()), thumbnail_url: format!("{}/thumb/far.png", server.uri()), title: "Blue tractor".into(), source_url: "https://b.example".into(), width: None, height: None },
    ];
    let search = SearchManager::with_providers(Arc::new(MockSearchProvider::new()), None, config.search.clone())
        .with_image_provider(Arc::new(Images(candidates)));
    let (mut pipeline, _tx) = ImagePipeline::new("img".into(), "Pictures like this one".into(), config, repo.clone(), None);
    pipeline.set_search(Arc::new(search));
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);

    // The query planner knows what the reference shows.
    let planned = completions(&server, "image search query generator").await;
    assert!(planned[0].to_string().contains("A red tractor in a field"));
    // The comparison sends the reference and the candidates as pictures.
    let compared = completions(&server, "reference picture").await;
    assert_eq!(compared.len(), 1);
    let pictures = compared[0].to_string().matches("data:image/").count();
    assert_eq!(pictures, 3, "one reference and two candidates");
    let stored = repo.get_image_results("img").await.unwrap();
    assert_eq!(stored.len(), 1, "the candidate unlike the reference is dropped");
    assert!(stored[0].image_url.ends_with("/full/near.png"));
    assert_eq!(stored[0].relevance_score, Some(0.95));
}

#[tokio::test]
async fn without_a_model_for_images_the_ranking_stays_textual() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(body_string_contains("image search query generator")).respond_with(reply(r#"{"queries": ["red tractor"]}"#)).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("strict image relevance judge")).respond_with(reply("[0.8]")).mount(&server).await;
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::new(repo.pool().clone(), dir.path().to_path_buf());
    let reference = store.add_bytes("tractor.png", png(210, 30, 30)).await.unwrap();
    let mut config = config(&server, dir.path(), &[("llm_vision", "off")]);
    config.attachments = vec![reference.id.clone()];
    let search = SearchManager::with_providers(Arc::new(MockSearchProvider::new()), None, config.search.clone()).with_image_provider(Arc::new(Images(vec![
        ImageSearchResult { image_url: "https://x.example/a.png".into(), thumbnail_url: "https://x.example/t.png".into(), title: "Tractor".into(), source_url: String::new(), width: None, height: None },
    ])));
    let (mut pipeline, _tx) = ImagePipeline::new("img2".into(), "Red tractors".into(), config, repo.clone(), None);
    pipeline.set_search(Arc::new(search));
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    assert!(completions(&server, "reference picture").await.is_empty());
    assert_eq!(repo.get_image_results("img2").await.unwrap().len(), 1);
    let logs = repo.get_run_logs("img2", 200).await.unwrap();
    assert!(logs.iter().any(|l| l.message.contains("not compared with the attached picture")), "the user learns why");
}

#[tokio::test]
async fn link_searches_and_ranking_know_the_attached_files() {
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/page")).respond_with(ResponseTemplate::new(200).set_body_string("<html><body><h1>Rust async runtimes</h1><p>Tokio and smol compared in depth.</p></body></html>").insert_header("content-type", "text/html")).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("web search queries")).respond_with(reply(r#"{"queries": ["rust async runtime comparison"]}"#)).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("Page URL")).respond_with(reply(r#"{"relevance": 0.9, "description": "A comparison of runtimes.", "reason": "Matches the paper's topic."}"#)).mount(&server).await;
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::new(repo.pool().clone(), dir.path().to_path_buf());
    let paper = store.add_bytes("paper.md", b"# Async runtimes in Rust\nWe compare Tokio, smol and async-std.".to_vec()).await.unwrap();
    let mut config = config(&server, dir.path(), &[]);
    config.attachments = vec![paper.id.clone()];
    let search = SearchManager::with_providers(
        Arc::new(MockSearchProvider::new().with_results(vec![SearchResult { title: "Runtimes".into(), url: format!("{}/page", server.uri()), snippet: "".into() }])),
        None,
        config.search.clone(),
    );
    let (mut pipeline, _tx) = LinkPipeline::new("links".into(), "Find articles like this paper".into(), config, repo.clone(), None);
    pipeline.set_search(Arc::new(search));
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    let planned = completions(&server, "web search queries").await;
    assert!(planned[0].to_string().contains("paper.md") && planned[0].to_string().contains("Tokio"));
    let ranked = completions(&server, "Page URL").await;
    assert!(ranked[0].to_string().contains("paper.md"), "the ranking knows what the files are about");
    assert_eq!(repo.get_link_results("links").await.unwrap().len(), 1);
}

#[tokio::test]
async fn comparing_can_be_turned_off_in_settings() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(body_string_contains("image search query generator")).respond_with(reply(r#"{"queries": ["red tractor"]}"#)).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("describe images")).respond_with(reply("A red tractor")).mount(&server).await;
    Mock::given(method("POST")).and(body_string_contains("strict image relevance judge")).respond_with(reply("[0.8]")).mount(&server).await;
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::new(repo.pool().clone(), dir.path().to_path_buf());
    let reference = store.add_bytes("tractor.png", png(210, 30, 30)).await.unwrap();
    let mut config = config(&server, dir.path(), &[("llm_vision", "on"), ("image_compare_max", "0")]);
    config.attachments = vec![reference.id.clone()];
    let search = SearchManager::with_providers(Arc::new(MockSearchProvider::new()), None, config.search.clone()).with_image_provider(Arc::new(Images(vec![
        ImageSearchResult { image_url: "https://x.example/a.png".into(), thumbnail_url: "https://x.example/t.png".into(), title: "Tractor".into(), source_url: String::new(), width: None, height: None },
    ])));
    let (mut pipeline, _tx) = ImagePipeline::new("img3".into(), "Red tractors".into(), config, repo.clone(), None);
    pipeline.set_search(Arc::new(search));
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    assert!(completions(&server, "reference picture").await.is_empty());
    let logs = repo.get_run_logs("img3", 200).await.unwrap();
    assert!(logs.iter().any(|l| l.message.contains("comparing is turned off in Settings")));
    assert_eq!(repo.get_image_results("img3").await.unwrap().len(), 1);
}
