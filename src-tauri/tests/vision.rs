//! Images in model requests and detection of models that can see them.

use std::collections::HashMap;

use query2table_lib::providers::llm::capabilities::{vision_support, VisionPlan};
use query2table_lib::providers::llm::{manager::LlmManager, ImageInput, Message};
use serde_json::{json, Value};
use wiremock::matchers::{body_partial_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn completion() -> Value {
    json!({"choices": [{"message": {"content": "A red square"}}], "usage": {"prompt_tokens": 900, "completion_tokens": 5}})
}

fn image() -> ImageInput {
    ImageInput { media_type: "image/png".into(), data_base64: "iVBORw0KGgo=".into() }
}

fn settings(pairs: &[(&str, String)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
}

#[tokio::test]
async fn openai_compatible_requests_carry_images_as_content_parts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"messages": [
            {"role": "system", "content": "Describe"},
            {"role": "user", "content": [
                {"type": "text", "text": "What is this?"},
                {"type": "image_url", "image_url": {"url": "data:image/png;base64,iVBORw0KGgo="}}
            ]}
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .expect(1)
        .mount(&server)
        .await;
    let config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "openai_compatible".into()),
        ("openai_base_url", format!("{}/v1", server.uri())),
        ("openai_model", "vl".into()),
    ]));
    let manager = LlmManager::from_config(config).unwrap();
    let reply = manager
        .complete(vec![Message::system("Describe"), Message::user_with_images("What is this?", vec![image()])], false)
        .await
        .unwrap();
    assert_eq!(reply.content, "A red square");
}

#[tokio::test]
async fn text_only_messages_keep_plain_string_content() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion()))
        .mount(&server)
        .await;
    let config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "openai_compatible".into()),
        ("openai_base_url", format!("{}/v1", server.uri())),
        ("openai_model", "m".into()),
    ]));
    LlmManager::from_config(config).unwrap().complete(vec![Message::user("hi")], false).await.unwrap();
    let body: Value = server.received_requests().await.unwrap()[0].body_json().unwrap();
    assert_eq!(body["messages"][0]["content"], json!("hi"));
}

#[tokio::test]
async fn ollama_requests_carry_images_beside_the_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .and(body_partial_json(json!({"messages": [
            {"role": "user", "content": "Read the page", "images": ["iVBORw0KGgo="]}
        ]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "message": {"content": "Page text"}, "done_reason": "stop", "prompt_eval_count": 10, "eval_count": 3
        })))
        .expect(1)
        .mount(&server)
        .await;
    let config = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "ollama".into()),
        ("ollama_url", server.uri()),
        ("ollama_model", "llava".into()),
    ]));
    let reply = LlmManager::from_config(config)
        .unwrap()
        .complete(vec![Message::user_with_images("Read the page", vec![image()])], false)
        .await
        .unwrap();
    assert_eq!(reply.content, "Page text");
}

#[tokio::test]
async fn vision_is_read_from_the_model_catalogs() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_partial_json(json!({"model": "llava:7b"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"capabilities": ["completion", "vision"]})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_partial_json(json!({"model": "llama3"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"capabilities": ["completion"]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [
            {"id": "openai/gpt-4o", "architecture": {"input_modalities": ["text", "image"]}},
            {"id": "deepseek/chat", "architecture": {"input_modalities": ["text"]}},
            {"id": "local"}
        ]})))
        .mount(&server)
        .await;

    let ollama = LlmManager::config_from_settings(&settings(&[("llm_provider", "ollama".into()), ("ollama_url", server.uri())]));
    assert_eq!(vision_support(&ollama, "llava:7b").await, Some(true));
    assert_eq!(vision_support(&ollama, "llama3").await, Some(false));
    assert_eq!(vision_support(&ollama, "missing").await, None);

    let openai = LlmManager::config_from_settings(&settings(&[
        ("llm_provider", "openai_compatible".into()),
        ("openai_base_url", format!("{}/v1", server.uri())),
    ]));
    assert_eq!(vision_support(&openai, "openai/gpt-4o").await, Some(true));
    assert_eq!(vision_support(&openai, "deepseek/chat").await, Some(false));
    assert_eq!(vision_support(&openai, "local").await, None);
}

#[tokio::test]
async fn the_vision_plan_follows_settings_and_detection() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .and(body_partial_json(json!({"model": "llava"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"capabilities": ["vision"]})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/api/show"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"capabilities": ["completion"]})))
        .mount(&server)
        .await;
    let base = [("llm_provider", "ollama".to_string()), ("ollama_url", server.uri())];

    // Auto: the main model sees images, so it reads them itself.
    let plan = VisionPlan::resolve(&settings(&[base[0].clone(), base[1].clone(), ("ollama_model", "llava".into())])).await;
    assert_eq!((plan.main_sees, plan.reader.as_deref()), (true, Some("llava")));

    // A blind main model hands images to the chosen model for images.
    let plan = VisionPlan::resolve(&settings(&[
        base[0].clone(),
        base[1].clone(),
        ("ollama_model", "llama3".into()),
        ("vision_model", "llava".into()),
    ]))
    .await;
    assert_eq!((plan.main_sees, plan.reader.as_deref()), (false, Some("llava")));

    // No model for images: nothing can read them.
    let plan = VisionPlan::resolve(&settings(&[base[0].clone(), base[1].clone(), ("ollama_model", "llama3".into())])).await;
    assert_eq!((plan.main_sees, plan.reader), (false, None));

    // The switch overrides detection.
    let plan = VisionPlan::resolve(&settings(&[
        base[0].clone(),
        base[1].clone(),
        ("ollama_model", "llama3".into()),
        ("llm_vision", "on".into()),
    ]))
    .await;
    assert_eq!((plan.main_sees, plan.reader.as_deref()), (true, Some("llama3")));
}
