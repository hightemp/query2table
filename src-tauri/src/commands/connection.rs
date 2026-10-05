//! "Test connection" checks for Settings, run against the values in the form before they
//! are saved, plus model catalogs for local servers.

use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;

use crate::providers::llm::manager::{LlmBackend, LlmManager};
use crate::providers::llm::ollama::OllamaProvider;
use crate::providers::llm::openai_compatible::OpenAiCompatibleProvider;
use crate::providers::llm::openrouter::OpenRouterProvider;
use crate::providers::search::brave::BraveSearchProvider;
use crate::providers::search::serper::SerperProvider;
use crate::providers::search::types::{SearchProvider, SearchQuery};

/// Outcome of a connection check that reached the service.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ConnectionReport {
    /// Everything needed for a run works.
    pub ok: bool,
    /// English text, used when the interface has no translation for `code`.
    pub message: String,
    /// Names the message for translation; `params` fill it in.
    pub code: Option<String>,
    pub params: HashMap<String, String>,
}

fn report(ok: bool, message: impl Into<String>, code: &str, params: &[(&str, String)]) -> ConnectionReport {
    ConnectionReport {
        ok,
        message: message.into(),
        code: Some(code.to_string()),
        params: params.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
    }
}

/// Ollama lists "llama3:latest" for a model chosen as "llama3".
fn model_listed(models: &[String], model: &str) -> bool {
    let model = model.trim();
    models
        .iter()
        .any(|m| m == model || m.strip_suffix(":latest") == Some(model) || model.strip_suffix(":latest") == Some(m))
}

fn model_report(models: &[String], model: &str, service: &str) -> ConnectionReport {
    let params = [("service", service.to_string()), ("model", model.trim().to_string())];
    if model.trim().is_empty() {
        return report(false, format!("Connected to {service}. Choose a model to finish setup."), "chooseModel", &params);
    }
    if models.is_empty() || model_listed(models, model) {
        report(true, format!("Connected to {service}; “{}” is available.", model.trim()), "modelAvailable", &params)
    } else {
        report(
            false,
            format!("Connected to {service}, but the model “{}” is not on this server.", model.trim()),
            "modelMissing",
            &params,
        )
    }
}

/// Checks the LLM settings without generating anything: credentials and the chosen model.
pub async fn check_llm(
    settings: &HashMap<String, String>,
    openrouter_base: Option<&str>,
) -> Result<ConnectionReport, String> {
    let config = LlmManager::config_from_settings(settings);
    match config.backend {
        LlmBackend::OpenRouter => {
            let mut provider = OpenRouterProvider::new(config.openrouter_api_key.clone()).map_err(|e| e.to_string())?;
            if let Some(base) = openrouter_base {
                provider = provider.with_base_url(base.to_string());
            }
            provider.check_key().await.map_err(|e| e.to_string())?;
            let models = provider.models().await.map_err(|e| e.to_string())?;
            Ok(model_report(&models, &config.openrouter_model, "OpenRouter"))
        }
        LlmBackend::Ollama => {
            let models = OllamaProvider::new(config.ollama_url.clone())
                .map_err(|e| e.to_string())?
                .list_models()
                .await
                .map_err(|e| e.to_string())?;
            Ok(model_report(&models, &config.ollama_model, "Ollama"))
        }
        LlmBackend::OllamaCloud => {
            if config.ollama_cloud_api_key.trim().is_empty() {
                return Err("Ollama Cloud API key is required".into());
            }
            let provider = OllamaProvider::cloud(config.ollama_cloud_url.clone(), config.ollama_cloud_api_key.clone())
                .map_err(|e| e.to_string())?;
            let models = provider.list_models().await.map_err(|e| e.to_string())?;
            let mut result = model_report(&models, &config.ollama_cloud_model, "Ollama Cloud");
            // The catalog is public and Ollama Cloud has no free way to check a key.
            if result.ok {
                result.message.push_str(" The API key is checked on the first model request.");
                result.params.insert("note".into(), "cloudKeyNote".into());
            }
            Ok(result)
        }
        LlmBackend::OpenAiCompatible => {
            let models = OpenAiCompatibleProvider::new(config.openai_base_url.clone(), config.openai_api_key.clone(), true)
                .map_err(|e| e.to_string())?
                .list_models()
                .await
                .map_err(|e| e.to_string())?;
            Ok(model_report(&models, &config.openai_model, "the server"))
        }
    }
}

/// Runs one search with a provider; this is a billable request.
pub async fn check_search_provider(provider: &dyn SearchProvider, name: &str) -> Result<ConnectionReport, String> {
    let results = provider.search(SearchQuery::new("test", 1)).await.map_err(|e| e.to_string())?;
    Ok(report(
        true,
        format!("{name} answered with {} result{}.", results.len(), if results.len() == 1 { "" } else { "s" }),
        "searchOk",
        &[("service", name.to_string()), ("count", results.len().to_string())],
    ))
}

/// Loads `target` through `proxy`.
pub async fn check_proxy(proxy: &str, target: &str) -> Result<ConnectionReport, String> {
    let proxy = reqwest::Proxy::all(proxy.trim()).map_err(|e| format!("Invalid proxy address: {e}"))?;
    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;
    let started = std::time::Instant::now();
    let response = client.get(target).send().await.map_err(|e| {
        if e.is_timeout() {
            "The proxy did not answer within 15 s (timeout).".to_string()
        } else {
            format!("Could not connect through the proxy: {e}")
        }
    })?;
    let status = response.status();
    if status == reqwest::StatusCode::PROXY_AUTHENTICATION_REQUIRED {
        return Err("HTTP 407: the proxy rejected its login or password.".into());
    }
    if !status.is_success() {
        return Err(format!("HTTP {}: the request through the proxy failed.", status.as_u16()));
    }
    let ms = started.elapsed().as_millis().to_string();
    Ok(report(true, format!("The proxy works ({ms} ms)."), "proxyOk", &[("ms", ms.clone())]))
}

#[tauri::command]
pub async fn test_llm_connection(settings: HashMap<String, String>) -> Result<ConnectionReport, String> {
    check_llm(&settings, None).await
}

#[tauri::command]
pub async fn test_search_connection(settings: HashMap<String, String>) -> Result<ConnectionReport, String> {
    let config = crate::providers::search::manager::SearchManager::config_from_settings(&settings);
    match settings.get("search_provider").map(String::as_str) {
        Some("serper") => {
            if config.serper_api_key.trim().is_empty() {
                return Err("Serper API key is required".into());
            }
            check_search_provider(&SerperProvider::new(config.serper_api_key), "Serper").await
        }
        _ => {
            if config.brave_api_key.trim().is_empty() {
                return Err("Brave Search API key is required".into());
            }
            check_search_provider(&BraveSearchProvider::new(config.brave_api_key), "Brave Search").await
        }
    }
}

#[tauri::command]
pub async fn test_proxy(url: String) -> Result<ConnectionReport, String> {
    check_proxy(&url, "https://www.gstatic.com/generate_204").await
}

#[tauri::command]
pub async fn list_ollama_models(base_url: String) -> Result<Vec<String>, String> {
    OllamaProvider::new(base_url)
        .map_err(|e| e.to_string())?
        .list_models()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_openai_models(base_url: String, api_key: String) -> Result<Vec<String>, String> {
    OpenAiCompatibleProvider::new(base_url, api_key, true)
        .map_err(|e| e.to_string())?
        .list_models()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn settings(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    #[tokio::test]
    async fn local_ollama_reports_whether_the_chosen_model_is_installed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"models": [{"name": "llama3:latest"}, {"name": "qwen3:8b"}]})))
            .mount(&server)
            .await;
        let base = settings(&[("llm_provider", "ollama"), ("ollama_url", &server.uri())]);
        let mut ok = base.clone();
        ok.insert("ollama_model".into(), "llama3".into());
        let found = check_llm(&ok, None).await.unwrap();
        assert!(found.ok, "{found:?}");
        assert!(found.message.contains("“llama3” is available"));
        assert_eq!(found.code.as_deref(), Some("modelAvailable"));
        assert_eq!(found.params.get("model").map(String::as_str), Some("llama3"));
        let mut missing = base.clone();
        missing.insert("ollama_model".into(), "mistral".into());
        let report = check_llm(&missing, None).await.unwrap();
        assert!(!report.ok);
        assert!(report.message.contains("“mistral” is not on this server"));
        assert_eq!(report.code.as_deref(), Some("modelMissing"));
        assert_eq!(list_ollama_models(server.uri()).await.unwrap(), ["llama3:latest", "qwen3:8b"]);
    }

    #[tokio::test]
    async fn openai_compatible_servers_list_models_and_reject_bad_keys() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .and(header("authorization", "Bearer good"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "local-model"}]})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({"error": {"message": "bad key"}})))
            .mount(&server)
            .await;
        let base_url = format!("{}/v1", server.uri());
        let good = settings(&[
            ("llm_provider", "openai_compatible"),
            ("openai_base_url", &base_url),
            ("openai_api_key", "good"),
            ("openai_model", "local-model"),
        ]);
        assert!(check_llm(&good, None).await.unwrap().ok);
        assert_eq!(list_openai_models(base_url.clone(), "good".into()).await.unwrap(), ["local-model"]);
        let mut bad = good.clone();
        bad.insert("openai_api_key".into(), "wrong".into());
        let error = check_llm(&bad, None).await.unwrap_err();
        assert!(error.to_lowercase().contains("auth") || error.contains("401"), "{error}");
    }

    #[tokio::test]
    async fn openrouter_checks_the_key_not_just_the_public_catalog() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/key"))
            .and(header("authorization", "Bearer sk-good"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": {"label": "test"}})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/key"))
            .respond_with(ResponseTemplate::new(401).set_body_json(json!({"error": {"message": "No auth credentials found"}})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/models"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "openai/gpt-4.1-mini"}]})))
            .mount(&server)
            .await;
        let good = settings(&[("llm_provider", "openrouter"), ("openrouter_api_key", "sk-good"), ("openrouter_model", "openai/gpt-4.1-mini")]);
        assert!(check_llm(&good, Some(&server.uri())).await.unwrap().ok);
        let bad = settings(&[("llm_provider", "openrouter"), ("openrouter_api_key", "sk-bad")]);
        assert!(check_llm(&bad, Some(&server.uri())).await.is_err());
        let empty = settings(&[("llm_provider", "openrouter")]);
        assert!(check_llm(&empty, Some(&server.uri())).await.unwrap_err().contains("API key"));
    }

    #[tokio::test]
    async fn a_search_check_runs_one_query() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"organic": [{"title": "T", "link": "https://a.example", "snippet": ""}]})))
            .expect(1)
            .mount(&server)
            .await;
        let provider = SerperProvider::new("key").with_base_url(server.uri());
        let report = check_search_provider(&provider, "Serper").await.unwrap();
        assert!(report.ok);
        assert_eq!(report.message, "Serper answered with 1 result.");
        assert_eq!(report.code.as_deref(), Some("searchOk"));
        assert_eq!(report.params.get("count").map(String::as_str), Some("1"));
    }

    #[tokio::test]
    async fn a_proxy_check_loads_a_page_through_the_proxy() {
        let server = MockServer::start().await;
        // A plain-HTTP proxy receives the full target URL; the mock answers it directly.
        Mock::given(method("GET"))
            .and(path("/generate_204"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let report = check_proxy(&server.uri(), "http://probe.invalid/generate_204").await.unwrap();
        assert!(report.ok && report.message.starts_with("The proxy works"), "{report:?}");
        assert_eq!(report.code.as_deref(), Some("proxyOk"));
        assert!(report.params.contains_key("ms"));
        let refusing = MockServer::start().await;
        Mock::given(method("GET")).respond_with(ResponseTemplate::new(407)).mount(&refusing).await;
        assert!(check_proxy(&refusing.uri(), "http://probe.invalid/generate_204").await.unwrap_err().contains("407"));
        assert!(check_proxy("not a url", "http://probe.invalid/").await.unwrap_err().contains("Invalid proxy"));
    }
}
