//! Finding videos: the web video search of Brave and Serper (any site), and yt-dlp's own
//! search on one site.

use std::path::{Path, PathBuf};

use query2table_lib::providers::search::brave::BraveSearchProvider;
use query2table_lib::providers::search::serper::SerperProvider;
use query2table_lib::providers::search::manager::{SearchBackend, SearchConfig};
use query2table_lib::providers::search::{SearchManager, SearchQuery, VideoSearchProvider};
use serde_json::json;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn brave_videos() -> serde_json::Value {
    json!({
        "type": "videos",
        "query": {"original": "rust async"},
        "results": [
            {
                "type": "video_result",
                "url": "https://www.youtube.com/watch?v=wXtngLBkK4Q",
                "title": "Async Rust explained in 20 minutes",
                "description": "Learn async Rust.",
                "age": "2 years ago",
                "page_age": "2023-03-14T16:00:00",
                "thumbnail": {"src": "https://imgs.search.brave.com/abc", "original": "https://i.ytimg.com/vi/wXtngLBkK4Q/hqdefault.jpg"},
                "video": {"duration": "19:15", "views": 250000, "creator": "Let's Get Rusty", "publisher": "YouTube"},
                "meta_url": {"hostname": "www.youtube.com"}
            },
            {
                "type": "video_result",
                "url": "https://vimeo.com/123",
                "title": "A long talk",
                "thumbnail": {"src": "https://imgs.search.brave.com/def"},
                "video": {"duration": "1:02:03"},
                "meta_url": {"hostname": "vimeo.com"}
            },
            {"type": "video_result", "title": "No address"}
        ]
    })
}

fn serper_videos() -> serde_json::Value {
    json!({
        "searchParameters": {"q": "rust async", "type": "videos"},
        "videos": [
            {
                "title": "Crust of Rust: async/await",
                "link": "https://www.youtube.com/watch?v=ThjvMReOXYM",
                "snippet": "We dig into async/await.",
                "imageUrl": "https://i.ytimg.com/vi/ThjvMReOXYM/mqdefault.jpg",
                "duration": "2:34:01",
                "source": "YouTube",
                "channel": "Jon Gjengset",
                "date": "Mar 4, 2021",
                "position": 1
            },
            {"title": "Bare", "link": "https://rutube.ru/video/abc/", "position": 2}
        ]
    })
}

#[tokio::test]
async fn brave_finds_videos_on_any_site() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/search"))
        .and(query_param("q", "rust async"))
        .and(query_param("count", "5"))
        .and(header("X-Subscription-Token", "key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(brave_videos()))
        .mount(&server)
        .await;
    let brave = BraveSearchProvider::new("key").with_base_url(server.uri());

    let results = brave.search_videos(SearchQuery::new("rust async", 5)).await.unwrap();
    assert_eq!(results.len(), 2, "a result without an address is dropped");
    let first = &results[0];
    assert_eq!(first.url, "https://www.youtube.com/watch?v=wXtngLBkK4Q");
    assert_eq!(first.title, "Async Rust explained in 20 minutes");
    assert_eq!(first.duration, Some(19 * 60 + 15));
    assert_eq!(first.channel.as_deref(), Some("Let's Get Rusty"));
    assert_eq!(first.site.as_deref(), Some("YouTube"));
    assert_eq!(first.views, Some(250000));
    assert_eq!(first.published.as_deref(), Some("2023-03-14"));
    assert_eq!(first.thumbnail.as_deref(), Some("https://i.ytimg.com/vi/wXtngLBkK4Q/hqdefault.jpg"));
    assert_eq!(first.description.as_deref(), Some("Learn async Rust."));
    let second = &results[1];
    assert_eq!(second.duration, Some(3723));
    assert_eq!(second.site.as_deref(), Some("vimeo.com"));
    assert_eq!(second.thumbnail.as_deref(), Some("https://imgs.search.brave.com/def"));
    assert!(VideoSearchProvider::received_bytes(&brave) > 0);
}

#[tokio::test]
async fn brave_asks_for_at_most_fifty_videos() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/videos/search"))
        .and(query_param("count", "50"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": []})))
        .expect(1)
        .mount(&server)
        .await;
    let brave = BraveSearchProvider::new("key").with_base_url(server.uri());
    assert!(brave.search_videos(SearchQuery::new("q", 500)).await.unwrap().is_empty());
}

#[tokio::test]
async fn serper_finds_videos_on_any_site() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/videos"))
        .and(header("X-API-KEY", "key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serper_videos()))
        .mount(&server)
        .await;
    let serper = SerperProvider::new("key").with_base_url(server.uri());

    let results = serper.search_videos(SearchQuery::new("rust async", 5)).await.unwrap();
    assert_eq!(results.len(), 2);
    let first = &results[0];
    assert_eq!(first.url, "https://www.youtube.com/watch?v=ThjvMReOXYM");
    assert_eq!(first.duration, Some(2 * 3600 + 34 * 60 + 1));
    assert_eq!(first.channel.as_deref(), Some("Jon Gjengset"));
    assert_eq!(first.site.as_deref(), Some("YouTube"));
    assert_eq!(first.published.as_deref(), Some("2021-03-04"));
    assert_eq!(first.thumbnail.as_deref(), Some("https://i.ytimg.com/vi/ThjvMReOXYM/mqdefault.jpg"));
    assert_eq!(results[1].site.as_deref(), Some("rutube.ru"));
    let body: serde_json::Value = serde_json::from_slice(&server.received_requests().await.unwrap()[0].body).unwrap();
    assert_eq!((body["q"].as_str(), body["num"].as_u64()), (Some("rust async"), Some(5)));
}

#[tokio::test]
async fn the_manager_falls_back_to_the_other_provider() {
    let brave = MockServer::start().await;
    Mock::given(method("GET")).and(path("/videos/search")).respond_with(ResponseTemplate::new(401)).mount(&brave).await;
    let serper = MockServer::start().await;
    Mock::given(method("POST")).and(path("/videos")).respond_with(ResponseTemplate::new(200).set_body_json(serper_videos())).mount(&serper).await;

    let config = SearchConfig { primary: SearchBackend::Brave, brave_api_key: "b".into(), serper_api_key: "s".into(), ..SearchConfig::default() };
    let manager = SearchManager::from_config(config)
        .unwrap()
        .with_video_providers(
            std::sync::Arc::new(BraveSearchProvider::new("b").with_base_url(brave.uri())),
            Some(std::sync::Arc::new(SerperProvider::new("s").with_base_url(serper.uri()))),
        );
    let results = manager.search_videos_with_count("rust async", 5).await.unwrap();
    assert_eq!(results[0].channel.as_deref(), Some("Jon Gjengset"));
}

// --- yt-dlp's own search on one site ---

#[cfg(unix)]
mod site_search {
    use super::*;
    use query2table_lib::components::{JsRuntime, Tools};
    use query2table_lib::providers::video::{VideoError, YtDlp};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/video").join(name)
    }

    fn printing(dir: &Path, name: &str) -> Tools {
        use std::os::unix::fs::PermissionsExt;
        let program = dir.join("yt-dlp");
        std::fs::write(&program, format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\ncat '{}'\n", dir.join("args.txt").display(), fixture(name).display())).unwrap();
        std::fs::set_permissions(&program, std::fs::Permissions::from_mode(0o755)).unwrap();
        Tools { program, prefix: Vec::new(), env: Vec::new(), js_runtime: JsRuntime::Deno(dir.join("deno")) }
    }

    #[tokio::test]
    async fn youtube_is_searched_by_ytdlp() {
        let dir = tempfile::tempdir().unwrap();
        let ytdlp = YtDlp::new(printing(dir.path(), "youtube_search.json"), dir.path().join("work")).with_proxy(None);

        let results = ytdlp.search("YouTube", "rust async\ntutorial", 3).await.unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Async Rust explained in 20 minutes");
        assert_eq!(results[0].url, "https://www.youtube.com/watch?v=wXtngLBkK4Q");
        assert_eq!(results[0].duration, Some(1155));
        assert_eq!(results[0].channel.as_deref(), Some("Let's Get Rusty"));
        assert_eq!(results[0].site.as_deref(), Some("Youtube"));
        assert!(results[0].thumbnail.as_deref().unwrap().starts_with("https://i.ytimg.com/vi/wXtngLBkK4Q/"));

        let args: Vec<String> = std::fs::read_to_string(dir.path().join("args.txt")).unwrap().lines().map(String::from).collect();
        assert!(args.contains(&"--flat-playlist".to_string()));
        // The query is one line, after "--".
        assert_eq!(&args[args.len() - 2..], ["--", "ytsearch3:rust async tutorial"]);
    }

    #[tokio::test]
    async fn each_site_has_its_search_key() {
        let dir = tempfile::tempdir().unwrap();
        let ytdlp = YtDlp::new(printing(dir.path(), "youtube_search.json"), dir.path().join("work")).with_proxy(None);
        for (site, key) in [("bilibili", "bilisearch"), ("niconico", "nicosearch"), ("soundcloud", "scsearch"), ("youtube", "ytsearch")] {
            ytdlp.search(site, "q", 50).await.unwrap();
            let args = std::fs::read_to_string(dir.path().join("args.txt")).unwrap();
            // At most twenty results.
            assert!(args.ends_with(&format!("--\n{key}20:q\n")), "{site}: {args}");
        }
        let error = ytdlp.search("example.com", "q", 3).await.unwrap_err();
        assert!(matches!(error, VideoError::UnknownSite(ref sites) if sites.contains("youtube") && sites.contains("bilibili")), "{error:?}");
    }
}
