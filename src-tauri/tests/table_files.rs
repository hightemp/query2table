//! Tables built from attached files: the files shape the schema, their fragments are extracted
//! like pages and cited by place, and a files-only run skips the web.

mod common;

use std::sync::Arc;

use query2table_lib::attachments::store::AttachmentStore;
use query2table_lib::orchestrator::pipeline::{Pipeline, PipelineState};
use query2table_lib::providers::llm::manager::LlmManager;
use query2table_lib::providers::search::manager::SearchManager;
use query2table_lib::providers::search::types::{SearchError, SearchResult};

use common::*;

const COMPANIES: &str = "Company;Website;Industry;Employees\nSAP SE;https://www.sap.com;Enterprise Software;107000\nSiemens AG;https://www.siemens.com;Technology;303000\n";

async fn attach(repo: &query2table_lib::storage::repository::Repository, dir: &std::path::Path) -> String {
    let store = AttachmentStore::new(repo.pool().clone(), dir.to_path_buf());
    store.add_bytes("companies.csv", COMPANIES.as_bytes().to_vec()).await.unwrap().id
}

#[tokio::test]
async fn a_files_only_table_reads_the_files_and_never_searches() {
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let file = attach(&repo, dir.path()).await;
    let mut config = test_pipeline_config();
    config.attachments = vec![file.clone()];
    config.attachments_dir = dir.path().to_path_buf();
    config.web_search = false;

    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm.clone(), config.llm.clone()));
    // Any search would fail the run.
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(MockSearchProvider::new().with_error(SearchError::RequestFailed("no web".into()))),
        None,
        config.search.clone(),
    ));
    let (mut pipeline, _tx) = Pipeline::new("t1".into(), "Make a table of the companies in the file".into(), config, repo.clone(), None);
    pipeline.set_providers(llm, search);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);

    assert_eq!(mock_llm.role_calls("search_plan"), 0);
    assert_eq!(mock_llm.role_calls("expand"), 0);
    assert!(repo.get_search_queries_by_run("t1").await.unwrap().is_empty());
    let interpreted = mock_llm.texts_for("interpret").join("\n");
    assert!(interpreted.contains("companies.csv") && interpreted.contains("SAP SE"), "{interpreted}");
    assert!(mock_llm.texts_for("schema").join("\n").contains("Columns: Company | Website | Industry | Employees"));
    let extracted = mock_llm.texts_for("extract").join("\n");
    assert!(extracted.contains("Siemens AG"), "the file fragment is extracted: {extracted}");

    let rows = repo.get_entity_rows_by_run("t1").await.unwrap();
    assert!(!rows.is_empty());
    let sources = repo.get_row_sources(&rows[0].id).await.unwrap();
    assert_eq!(sources[0].url, format!("attachment://{file}?rows=2-3"));
    assert_eq!(sources[0].title.as_deref(), Some("companies.csv, rows 2–3"));
    let run = repo.get_run("t1").await.unwrap().unwrap();
    let stats: serde_json::Value = serde_json::from_str(run.stats.as_deref().unwrap()).unwrap();
    assert_eq!(stats["file_fragments"], 1);
}

#[tokio::test]
async fn files_and_the_web_are_both_sources() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::any())
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .set_body_string("<html><body><h1>German tech</h1><p>SAP SE builds enterprise software. Siemens AG builds trains and much more.</p></body></html>")
                .insert_header("content-type", "text/html"),
        )
        .mount(&server)
        .await;
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let file = attach(&repo, dir.path()).await;
    let mut config = test_pipeline_config();
    config.attachments = vec![file.clone()];
    config.attachments_dir = dir.path().to_path_buf();
    let fetcher = test_fetcher(&config);
    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm.clone(), config.llm.clone()));
    let search = Arc::new(SearchManager::with_providers(
        Arc::new(MockSearchProvider::new().with_results(vec![SearchResult {
            title: "German tech".into(),
            url: format!("{}/page", server.uri()),
            snippet: "tech".into(),
        }])),
        None,
        config.search.clone(),
    ));
    let (mut pipeline, _tx) = Pipeline::new("t2".into(), "Technology companies in Germany".into(), config, repo.clone(), None);
    pipeline.set_providers(llm, search);
    pipeline.set_fetcher(fetcher);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);

    assert!(mock_llm.role_calls("search_plan") > 0);
    assert!(mock_llm.texts_for("search_plan").join("\n").contains("companies.csv"));
    let mut urls = Vec::new();
    for row in repo.get_entity_rows_by_run("t2").await.unwrap() {
        for source in repo.get_row_sources(&row.id).await.unwrap() {
            urls.push(source.url);
        }
    }
    assert!(urls.iter().any(|u| u.starts_with("attachment://")), "{urls:?}");
    assert!(urls.iter().any(|u| u.starts_with("http")), "{urls:?}");
    // Every planned query is saved with what it found.
    let queries = repo.get_run_queries("t2").await.unwrap();
    assert!(!queries.is_empty());
    assert!(queries.iter().all(|q| q.status == "completed"), "{queries:?}");
    assert!(queries.iter().any(|q| q.result_count > 0));
}

#[tokio::test]
async fn a_russian_query_gets_a_russian_table() {
    let (repo, _db) = setup_test_db().await;
    let dir = tempfile::tempdir().unwrap();
    let file = attach(&repo, dir.path()).await;
    let mut config = test_pipeline_config();
    config.attachments = vec![file];
    config.attachments_dir = dir.path().to_path_buf();
    config.web_search = false;
    let mock_llm = Arc::new(MockLlmProvider::new());
    let llm = Arc::new(LlmManager::with_provider(mock_llm.clone(), config.llm.clone()));
    let search = Arc::new(SearchManager::with_providers(Arc::new(MockSearchProvider::new()), None, config.search.clone()));
    let (mut pipeline, _tx) = Pipeline::new("ru".into(), "Сделай таблицу компаний из файла".into(), config, repo.clone(), None);
    pipeline.set_providers(llm, search);
    assert_eq!(pipeline.run().await.unwrap(), PipelineState::Completed);
    // The fixture interpreter reply names no language: the Cyrillic query decides.
    assert!(mock_llm.systems_for("schema")[0].contains("write every column name and description in Russian"));
    assert!(mock_llm.systems_for("extract").iter().all(|s| s.contains("notes) in Russian")));
}
