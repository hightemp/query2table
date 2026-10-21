//! Charts in research answers: the ```chart scheme (shared examples with the frontend in
//! `tests/fixtures/charts.json`) and the check of answers before they are accepted.

mod common;

use std::collections::HashMap;
use std::sync::Arc;

use query2table_lib::orchestrator::pipeline::{PipelineConfig, PipelineState};
use query2table_lib::orchestrator::research_pipeline::ResearchPipeline;
use query2table_lib::providers::search::manager::SearchManager;
use query2table_lib::roles::chart;
use query2table_lib::storage::{db::Database, repository::Repository};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use common::MockSearchProvider;

/// The block text of an example; `"x": "LABELS_<n>"` stands for n labels.
fn source(example: &Value) -> String {
    if let Some(source) = example["source"].as_str() {
        return source.to_string();
    }
    let mut chart = example["chart"].clone();
    if let Some(count) = chart["x"].as_str().and_then(|x| x.strip_prefix("LABELS_")) {
        let count: usize = count.parse().unwrap();
        chart["x"] = (0..count).map(|i| Value::String(format!("L{i}"))).collect();
    }
    chart.to_string()
}

#[test]
fn chart_blocks_are_checked_like_on_the_frontend() {
    let examples: Vec<Value> = serde_json::from_str(include_str!("fixtures/charts.json")).unwrap();
    assert!(examples.len() > 30);
    for example in &examples {
        let name = example["name"].as_str().unwrap();
        let result = chart::check(&source(example));
        match example["error"].as_str() {
            None => assert!(result.is_ok(), "{name}: {result:?}"),
            Some(expected) => {
                let error = result.expect_err(name);
                assert!(error.starts_with(expected), "{name}: {error:?} should start with {expected:?}");
            }
        }
    }
}

#[test]
fn json_errors_say_where_the_problem_is() {
    let error = chart::check("{\"type\": \"bar\",}").unwrap_err();
    assert!(error.starts_with("not valid JSON: ") && error.contains("column"), "{error}");
}

#[test]
fn counts_are_worded_for_one_and_many() {
    let error = chart::check(r#"{"type":"bar","x":["a"],"series":[{"name":"A","data":[1,2]}]}"#).unwrap_err();
    assert_eq!(error, "series \"A\" has 2 values for 1 label");
    let error = chart::check(r#"{"type":"bar","x":["a","b"],"series":[{"data":[1]}]}"#).unwrap_err();
    assert_eq!(error, "series 1 has 1 value for 2 labels");
}

const BROKEN: &str = "```chart\n{\"type\":\"bar\",\"x\":[\"2020\",\"2021\",\"2022\"],\"series\":[{\"name\":\"RU\",\"data\":[146,145]}]}\n```";
const FIXED: &str = "```chart\n{\"type\":\"bar\",\"x\":[\"2020\",\"2021\",\"2022\"],\"series\":[{\"name\":\"RU\",\"data\":[146,145,144]}]}\n```";

#[test]
fn answers_list_the_problems_of_their_chart_blocks() {
    let markdown = format!(
        "Intro\n\n{FIXED}\n\nText\n\n~~~chart\n{{\"type\": \"bubble\"}}\n~~~\n\n{BROKEN}\n\n```mermaid\ngraph TD; A-->B\n```\n"
    );
    assert_eq!(
        chart::problems(&markdown),
        [
            "Chart 2: unknown type \"bubble\"; use one of bar, line, area, pie, donut, scatter, radar, heatmap",
            "Chart 3: series \"RU\" has 2 values for 3 labels",
        ]
    );
    // A chart shown as an example inside another code block is not drawn, so not checked.
    let example = format!("````markdown\n{BROKEN}\n````\n\n    ```chart\n    {{}}\n    ```\n");
    assert!(chart::problems(&example).is_empty(), "{:?}", chart::problems(&example));
    // An unclosed block runs to the end of the answer.
    assert_eq!(chart::problems("```chart\n{\"type\":\"pie\"").len(), 1);
    assert!(chart::problems("No charts.").is_empty());
}

fn reply(content: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "choices": [{"message": {"content": content.to_string()}}],
        "usage": {"prompt_tokens": 10, "completion_tokens": 5}
    }))
}

/// Replies the model gives in order; the last one repeats.
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

fn last_message(request: &Value) -> String {
    request["messages"].as_array().unwrap().last().unwrap()["content"].as_str().unwrap_or_default().to_string()
}

/// Runs one research turn with the scripted model and one step allowed; returns the stored answer.
async fn research(server: &MockServer) -> (String, Arc<Repository>) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.unwrap();
    Database::with_pool(pool.clone()).await.migrate().await.unwrap();
    let repo = Arc::new(Repository::new(pool));
    let settings: HashMap<String, String> = HashMap::from([
        ("llm_provider".into(), "openai_compatible".into()),
        ("openai_base_url".into(), format!("{}/v1", server.uri())),
        ("openai_model".into(), "test-model".into()),
        ("brave_api_key".into(), "test-key".into()),
    ]);
    let mut config = PipelineConfig::from_settings(&settings);
    config.stop.target_row_count = 1;
    let (pipeline, _c) = ResearchPipeline::new("r1".into(), "Population of Russia".into(), config, repo.clone(), None);
    let search = SearchManager::with_providers(Arc::new(MockSearchProvider::empty()), None, Default::default());
    assert_eq!(pipeline.with_search(search).run().await.unwrap(), PipelineState::Completed);
    let turns = repo.get_research_turns("r1").await.unwrap();
    (turns[0].answer_markdown.clone().unwrap_or_default(), repo)
}

#[tokio::test]
async fn a_broken_chart_is_sent_back_once_to_be_fixed() {
    let server = MockServer::start().await;
    script(
        &server,
        &[
            json!({"action": "answer", "markdown": format!("Population:\n\n{BROKEN}")}),
            json!({"action": "answer", "markdown": format!("Population:\n\n{FIXED}"), "follow_ups": ["And Kazakhstan?"]}),
        ],
    )
    .await;
    let (answer, repo) = research(&server).await;
    assert!(answer.contains("146,145,144"), "{answer}");
    let sent = completions(&server).await;
    // Fixing is allowed past the step limit.
    assert_eq!(sent.len(), 2);
    let request = last_message(&sent[1]);
    assert!(request.contains("Chart 1: series \"RU\" has 2 values for 3 labels") && request.contains("answer again"), "{request}");
    let turns = repo.get_research_turns("r1").await.unwrap();
    assert!(turns[0].follow_ups.contains("Kazakhstan"), "{}", turns[0].follow_ups);
}

#[tokio::test]
async fn a_chart_still_broken_after_the_fix_keeps_the_answer() {
    let server = MockServer::start().await;
    script(&server, &[json!({"action": "answer", "markdown": format!("First:\n\n{BROKEN}")}), json!({"action": "answer", "markdown": format!("Second:\n\n{BROKEN}")})]).await;
    let (answer, repo) = research(&server).await;
    assert!(answer.starts_with("Second:"), "{answer}");
    assert_eq!(completions(&server).await.len(), 2, "only one fix is asked for");
    let logs = repo.get_run_logs("r1", 100).await.unwrap();
    assert!(logs.iter().any(|l| l.level == "WARN" && l.message.contains("Chart 1")), "{:?}", logs.iter().map(|l| &l.message).collect::<Vec<_>>());
}

#[tokio::test]
async fn a_failed_fix_keeps_the_first_answer() {
    let server = MockServer::start().await;
    script(&server, &[json!({"action": "answer", "markdown": format!("First:\n\n{BROKEN}")}), json!({"action": "search", "query": "more"})]).await;
    let (answer, _repo) = research(&server).await;
    assert!(answer.starts_with("First:"), "{answer}");
}

#[tokio::test]
async fn answers_with_good_charts_are_accepted_at_once() {
    let server = MockServer::start().await;
    script(&server, &[json!({"action": "answer", "markdown": format!("Population:\n\n{FIXED}")})]).await;
    let (answer, _repo) = research(&server).await;
    assert!(answer.contains("146,145,144"));
    assert_eq!(completions(&server).await.len(), 1);
}
