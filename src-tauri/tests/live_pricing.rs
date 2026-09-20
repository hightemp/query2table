//! Public metadata only: no API keys or inference requests.
use query2table_lib::providers::llm::{ollama::OllamaProvider, LlmProvider, LlmUsage};
#[tokio::test]
#[ignore = "Reads the current public Ollama pricing page"]
async fn official_ollama_pricing_is_readable() {
    query2table_lib::providers::http::proxy::init_and_strip_env();
    let provider = OllamaProvider::new("http://localhost:11434".into()).unwrap();
    let usage = LlmUsage {
        prompt_tokens: Some(1000),
        remote_host: Some("https://ollama.com".into()),
        completion_tokens: Some(100),
        ..Default::default()
    };
    let price = provider
        .pricing("deepseek-v4.1-flash:cloud", &usage, chrono::Utc::now())
        .await
        .expect("official pricing page must parse");
    assert!(price.valid());
    assert!(price.estimate(&usage).unwrap() > 0.0);
    assert!(price.credit_based);
    println!("Public Ollama pricing parsed with source: {}", price.source);
}
