use query2table_lib::commands::settings::list_openrouter_models;

// Keep proxy initialization in a separate test executable so local HTTP mocks stay isolated.
#[test]
#[ignore = "Queries the live public OpenRouter catalog using the app's environment proxy setup"]
fn openrouter_catalog_live_endpoint() {
    query2table_lib::providers::http::proxy::init_and_strip_env();
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let models = list_openrouter_models(String::new()).await.unwrap();
        assert!(!models.is_empty());
        println!("OpenRouter returned {} model IDs", models.len());
    });
}
