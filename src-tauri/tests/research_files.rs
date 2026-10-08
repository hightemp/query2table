//! Research with attached files: overview and matching fragments up front, `read_file`,
//! images for models that see them, files-only runs and file sources.

use std::collections::HashMap;
use std::sync::Arc;

use query2table_lib::attachments::store::AttachmentStore;
use query2table_lib::orchestrator::pipeline::{PipelineConfig, PipelineState};
use query2table_lib::orchestrator::research_pipeline::ResearchPipeline;
use query2table_lib::storage::{db::Database, repository::Repository};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn reply(content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": content.to_string()}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }))
}

struct Setup {
    server: MockServer,
    repo: Arc<Repository>,
    store: AttachmentStore,
    dir: tempfile::TempDir,
}

async fn setup() -> Setup {
    let server = MockServer::start().await;
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = AttachmentStore::new(pool.clone(), dir.path().to_path_buf());
    Setup { server, repo: Arc::new(Repository::new(pool)), store, dir }
}

fn config(setup: &Setup, extra: &[(&str, &str)]) -> PipelineConfig {
    let mut settings: HashMap<String, String> = HashMap::from([
        ("llm_provider".into(), "openai_compatible".into()),
        ("openai_base_url".into(), format!("{}/v1", setup.server.uri())),
        ("openai_model".into(), "test-model".into()),
        ("brave_api_key".into(), "test-key".into()),
    ]);
    for (k, v) in extra {
        settings.insert(k.to_string(), v.to_string());
    }
    let mut config = PipelineConfig::from_settings(&settings);
    config.attachments_dir = setup.dir.path().to_path_buf();
    config
}

async fn requests(server: &MockServer) -> Vec<Value> {
    // Only completions: the catalog may also be asked whether the model sees images.
    server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method == wiremock::http::Method::POST)
        .map(|r| r.body_json().unwrap())
        .collect()
}

/// All text the model was sent in one request.
fn text_of(request: &Value) -> String {
    request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| match &m["content"] {
            Value::String(s) => s.clone(),
            Value::Array(parts) => parts.iter().filter_map(|p| p["text"].as_str()).collect::<Vec<_>>().join("\n"),
            _ => String::new(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tokio::test]
async fn the_agent_gets_the_files_and_reads_a_page_it_asks_for() {
    let s = setup().await;
    let report = s
        .store
        .add_bytes("report.pdf", common_pdf(&["Acme revenue reached 12 million dollars in 2025", "The main office of Acme is in Berlin"]))
        .await
        .unwrap();
    let answer = format!("Acme is based in Berlin [report.pdf, p. 2](attachment://{}?page=2).", report.id);
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "read_file", "file": "F1", "page": 2})))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&s.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "answer", "markdown": answer})))
        .with_priority(2)
        .mount(&s.server)
        .await;

    let mut config = config(&s, &[]);
    config.attachments = vec![report.id.clone()];
    let (pipeline, _commands) = ResearchPipeline::new("r1".into(), "How much revenue did Acme make?".into(), config, s.repo.clone(), None);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);

    let sent = requests(&s.server).await;
    let first = text_of(&sent[0]);
    assert!(first.contains("read_file"), "the tool is offered");
    assert!(first.contains("[F1] report.pdf"), "{first}");
    assert!(first.contains(&format!("attachment://{}", report.id)));
    // The fragment matching the question is given up front, with its place.
    assert!(first.contains("report.pdf, page 1") && first.contains("revenue reached 12 million"), "{first}");
    let second = text_of(&sent[1]);
    assert!(second.contains("File content (report.pdf, page 2)") && second.contains("main office of Acme is in Berlin"), "{second}");

    let steps = s.repo.get_research_steps("r1").await.unwrap();
    let read = steps.iter().find(|st| st.step_type == "read").expect("a read step");
    assert_eq!(read.content, "report.pdf, page 2");
    assert_eq!(read.url.as_deref(), Some(format!("attachment://{}?page=2", report.id).as_str()));
    let files = s.store.for_run("r1").await.unwrap();
    assert_eq!(files.len(), 1);
}

#[tokio::test]
async fn pictures_go_to_a_model_that_sees_them() {
    let s = setup().await;
    let photo = s.store.add_bytes("photo.png", common_png()).await.unwrap();
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "answer", "markdown": "A blue square."})))
        .mount(&s.server)
        .await;
    let mut config = config(&s, &[("llm_vision", "on")]);
    config.attachments = vec![photo.id.clone()];
    let (pipeline, _commands) = ResearchPipeline::new("r2".into(), "What is in the picture?".into(), config, s.repo.clone(), None);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    let sent = requests(&s.server).await;
    let images: Vec<&Value> = sent[0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["content"].as_array())
        .flatten()
        .filter(|p| p["type"] == "image_url")
        .collect();
    assert_eq!(images.len(), 1);
    assert!(images[0]["image_url"]["url"].as_str().unwrap().starts_with("data:image/jpeg;base64,"));
}

#[tokio::test]
async fn files_only_runs_need_no_search_and_refuse_web_tools() {
    let s = setup().await;
    let notes = s.store.add_bytes("notes.md", b"# Plans\nPro costs $10 a month.".to_vec()).await.unwrap();
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "search", "query": "pro plan price"})))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&s.server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "answer", "markdown": "Pro costs $10."})))
        .with_priority(2)
        .mount(&s.server)
        .await;
    // No search key at all: a files-only run does not need one.
    let mut config = config(&s, &[("brave_api_key", "")]);
    config.attachments = vec![notes.id.clone()];
    config.web_search = false;
    let (pipeline, _commands) = ResearchPipeline::new("r3".into(), "What does Pro cost?".into(), config, s.repo.clone(), None);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    let sent = requests(&s.server).await;
    let system = sent[0]["messages"][0]["content"].as_str().unwrap();
    assert!(!system.contains("\"action\": \"search\""), "web tools are not offered");
    assert!(system.contains("only the attached files"));
    assert!(text_of(&sent[1]).contains("Web search is turned off for this run"));
    let run = s.repo.get_run("r3").await.unwrap().unwrap();
    assert_eq!(run.status, "completed");
}

#[tokio::test]
async fn a_follow_up_can_bring_new_files_and_keeps_the_earlier_ones() {
    let s = setup().await;
    let first = s.store.add_bytes("first.txt", b"The first file mentions apples.".to_vec()).await.unwrap();
    let second = s.store.add_bytes("second.txt", b"The second file mentions pears.".to_vec()).await.unwrap();
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(reply(json!({"action": "answer", "markdown": "Fruit."})))
        .mount(&s.server)
        .await;
    let mut cfg = config(&s, &[]);
    cfg.attachments = vec![first.id.clone()];
    let (pipeline, _c) = ResearchPipeline::new("r4".into(), "Which fruit?".into(), cfg, s.repo.clone(), None);
    pipeline.run().await.unwrap();
    let mut cfg = config(&s, &[]);
    cfg.attachments = vec![second.id.clone()];
    let (pipeline, _c) = ResearchPipeline::follow_up("r4".into(), "And now?".into(), 1, vec![], cfg, s.repo.clone(), None);
    pipeline.run().await.unwrap();
    let sent = requests(&s.server).await;
    let follow_up = text_of(sent.last().unwrap());
    assert!(follow_up.contains("first.txt") && follow_up.contains("second.txt"), "{follow_up}");
    let turns: Vec<i64> = s.store.for_run("r4").await.unwrap().iter().map(|(t, _)| *t).collect();
    assert_eq!(turns, vec![0, 1]);
}

fn common_pdf(pages: &[&str]) -> Vec<u8> {
    use pdf_extract::content::{Content, Operation};
    use pdf_extract::{dictionary, Document, Object, Stream};
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica" });
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut kids = Vec::new();
    for text in pages {
        let operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("Td", vec![72.into(), 700.into()]),
            Operation::new("Tj", vec![Object::string_literal(*text)]),
            Operation::new("ET", vec![]),
        ];
        let content_id = doc.add_object(Stream::new(dictionary! {}, Content { operations }.encode().unwrap()));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content_id, "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        kids.push(Object::from(page_id));
    }
    let count = kids.len() as i64;
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => kids, "Count" => count }));
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

fn common_png() -> Vec<u8> {
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(40, 30, image::Rgb([20, 40, 200])));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}
