use query2table_lib::providers::llm::manager::{LlmBackend, LlmConfig, LlmManager};
use query2table_lib::roles::{
    query_expander::QueryExpander, query_interpreter::QueryInterpreter,
    schema_planner::SchemaPlanner, search_planner::SearchPlanner,
};

#[test]
#[ignore = "Calls Ollama Cloud; requires OLLAMA_API_KEY and optionally OLLAMA_MODEL"]
fn robot_channel_query_produces_a_complete_search_plan() {
    query2table_lib::providers::http::proxy::init_and_strip_env();
    let api_key =
        std::env::var("OLLAMA_API_KEY").expect("OLLAMA_API_KEY must be set for this live test");
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-v4.1-flash".into());
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let llm = LlmManager::from_config(LlmConfig {
            backend: LlmBackend::OllamaCloud,
            ollama_cloud_api_key: api_key,
            ollama_cloud_model: model,
            temperature: 0.7,
            max_tokens: 4096,
            ..Default::default()
        }).unwrap();
        let started = std::time::Instant::now();
        let intent = QueryInterpreter::interpret("Найди youtube каналы про роботов", &llm).await.unwrap();
        let schema = SchemaPlanner::plan(&intent, &llm).await.unwrap();
        let plan = SearchPlanner::plan(&intent, &schema, &llm).await.unwrap();
        let expanded = QueryExpander::expand(&plan.queries, &intent.languages, &llm).await.unwrap();
        assert!(!plan.queries.is_empty());
        assert!(!expanded.queries.is_empty());
        println!("Live Ollama JSON pipeline completed in {:.1}s: {} columns, {} planned queries, {} expansions",
            started.elapsed().as_secs_f64(), schema.columns.len(), plan.queries.len(), expanded.queries.len());
    });
}
