//! External tools (yt-dlp, Deno) downloaded from GitHub releases: installed from a fake
//! GitHub, checked against their published checksums, kept in a manifest and updated weekly.
#![cfg(unix)]

use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use query2table_lib::components::{platform, Component, ComponentError, ComponentManager, JsRuntime, Stage};
use sha2::Digest;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const YTDLP_TAG: &str = "2026.10.26";
const DENO_TAG: &str = "v2.6.1";

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", sha2::Sha256::digest(bytes))
}

/// A tool that prints its version, like `yt-dlp --version`.
fn tool(version: &str) -> Vec<u8> {
    format!("#!/bin/sh\necho {version}\n").into_bytes()
}

fn zip_with(entry: &str, body: &[u8]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file(entry, zip::write::SimpleFileOptions::default()).unwrap();
    zip.write_all(body).unwrap();
    zip.finish().unwrap().into_inner()
}

/// Serves the releases of yt-dlp and Deno the way github.com does.
async fn github(ytdlp: &[u8], ytdlp_sum: &str, deno_zip: &[u8], deno_sum: &str) -> MockServer {
    let server = MockServer::start().await;
    for (repo, tag) in [("yt-dlp/yt-dlp", YTDLP_TAG), ("denoland/deno", DENO_TAG)] {
        Mock::given(method("GET"))
            .and(path(format!("/{repo}/releases/latest")))
            .respond_with(ResponseTemplate::new(302).insert_header("location", format!("https://github.com/{repo}/releases/tag/{tag}")))
            .mount(&server)
            .await;
    }
    let assets = platform::assets_for("linux", "x86_64").unwrap();
    let ytdlp_dir = format!("/yt-dlp/yt-dlp/releases/download/{YTDLP_TAG}");
    Mock::given(method("GET"))
        .and(path(format!("{ytdlp_dir}/SHA2-256SUMS")))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("0000  yt-dlp\n{ytdlp_sum}  {}\n", assets.ytdlp)))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{ytdlp_dir}/{}", assets.ytdlp)))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(ytdlp.to_vec()))
        .mount(&server)
        .await;
    let deno_asset = format!("deno-{}.zip", assets.deno_triple);
    let deno_dir = format!("/denoland/deno/releases/download/{DENO_TAG}");
    Mock::given(method("GET"))
        .and(path(format!("{deno_dir}/{deno_asset}.sha256sum")))
        .respond_with(ResponseTemplate::new(200).set_body_string(format!("{deno_sum}  {deno_asset}\n")))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("{deno_dir}/{deno_asset}")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(deno_zip.to_vec()))
        .mount(&server)
        .await;
    server
}

async fn honest_github() -> MockServer {
    let ytdlp = tool(YTDLP_TAG);
    let deno = zip_with("deno", &tool("deno 2.6.1"));
    github(&ytdlp, &sha256(&ytdlp), &deno, &sha256(&deno)).await
}

fn manager(dir: &Path, server: &MockServer) -> ComponentManager {
    ComponentManager::with_platform(dir.to_owned(), platform::assets_for("linux", "x86_64").unwrap(), &server.uri(), None).unwrap()
}

fn files_in(dir: &Path) -> Vec<String> {
    let mut files: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
    files.sort();
    files
}

#[tokio::test]
async fn missing_tools_are_installed_from_their_releases() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);
    assert!(tools.status().iter().all(|s| !s.installed && s.version.is_none()));
    assert!(matches!(tools.tools(), Err(ComponentError::Missing(Component::Ytdlp))));

    let stages = Mutex::new(Vec::new());
    let ready = tools
        .ensure(&|p| {
            let mut stages = stages.lock().unwrap();
            if stages.last() != Some(&(p.component, p.stage)) {
                stages.push((p.component, p.stage));
            }
        })
        .await
        .unwrap();

    assert_eq!(ready.program, dir.path().join("yt-dlp"));
    assert!(ready.prefix.is_empty());
    assert_eq!(ready.js_runtime, JsRuntime::Deno(dir.path().join("deno")));
    let stages = stages.into_inner().unwrap();
    for component in [Component::Ytdlp, Component::Deno] {
        for stage in [Stage::Download, Stage::Verify, Stage::Install] {
            assert!(stages.contains(&(component, stage)), "{component:?} {stage:?} in {stages:?}");
        }
    }
    let status = tools.status();
    assert_eq!(status.iter().find(|s| s.component == Component::Ytdlp).unwrap().version.as_deref(), Some(YTDLP_TAG));
    assert_eq!(status.iter().find(|s| s.component == Component::Deno).unwrap().version.as_deref(), Some(DENO_TAG));
    // Only the tools and the manifest remain: no archive, no temp files.
    assert_eq!(files_in(dir.path()), ["deno", "manifest.json", "yt-dlp"]);

    // The versions survive a restart, and nothing is downloaded twice.
    let requests = server.received_requests().await.unwrap().len();
    let reopened = manager(dir.path(), &server);
    assert!(reopened.status().iter().all(|s| s.installed && s.version.is_some()));
    reopened.ensure(&|_| {}).await.unwrap();
    assert_eq!(server.received_requests().await.unwrap().len(), requests);
}

#[tokio::test]
async fn a_download_that_does_not_match_its_checksum_is_refused() {
    let ytdlp = tool(YTDLP_TAG);
    let deno = zip_with("deno", &tool("deno"));
    let server = github(&ytdlp, &sha256(b"something else"), &deno, &sha256(&deno)).await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);

    let error = tools.install(Component::Ytdlp, &|_| {}).await.unwrap_err();
    assert!(matches!(error, ComponentError::Checksum(ref name) if name == "yt-dlp_linux"), "{error:?}");
    assert!(!tools.status()[0].installed);
    assert_eq!(files_in(dir.path()), Vec::<String>::new());
}

#[tokio::test]
async fn a_tool_that_does_not_run_keeps_the_working_one() {
    let broken = b"#!/bin/sh\necho broken >&2\nexit 1\n".to_vec();
    let deno = zip_with("deno", &tool("deno"));
    let server = github(&broken, &sha256(&broken), &deno, &sha256(&deno)).await;
    let dir = tempfile::tempdir().unwrap();
    let working = tool("2026.01.01");
    std::fs::write(dir.path().join("yt-dlp"), &working).unwrap();
    let tools = manager(dir.path(), &server);

    let error = tools.install(Component::Ytdlp, &|_| {}).await.unwrap_err();
    assert!(error.to_string().contains("broken"), "{error}");
    assert_eq!(std::fs::read(dir.path().join("yt-dlp")).unwrap(), working);
    assert_eq!(files_in(dir.path()), ["yt-dlp"]);
}

#[tokio::test]
async fn an_archive_without_the_tool_is_refused() {
    let ytdlp = tool(YTDLP_TAG);
    let deno = zip_with("README.md", b"no binary");
    let server = github(&ytdlp, &sha256(&ytdlp), &deno, &sha256(&deno)).await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);

    let error = tools.install(Component::Deno, &|_| {}).await.unwrap_err();
    assert!(error.to_string().contains("deno"), "{error}");
    assert_eq!(files_in(dir.path()), Vec::<String>::new());
}

#[tokio::test]
async fn removed_tools_are_missing_again() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);
    tools.ensure(&|_| {}).await.unwrap();

    tools.remove(Component::Deno).unwrap();
    assert!(matches!(tools.tools(), Err(ComponentError::Missing(Component::Deno))));
    let deno = tools.status().into_iter().find(|s| s.component == Component::Deno).unwrap();
    assert!(!deno.installed && deno.version.is_none());
    assert_eq!(files_in(dir.path()), ["manifest.json", "yt-dlp"]);
}

#[tokio::test]
async fn ytdlp_is_updated_at_most_once_a_week() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    // Installed long ago, an older release.
    std::fs::write(dir.path().join("yt-dlp"), tool("2026.01.01")).unwrap();
    let week_ago = chrono::Utc::now().timestamp() - 8 * 24 * 60 * 60;
    std::fs::write(
        dir.path().join("manifest.json"),
        format!(r#"{{"versions": {{"ytdlp": "2026.01.01"}}, "lastUpdateCheck": {week_ago}}}"#),
    )
    .unwrap();
    let tools = manager(dir.path(), &server);

    assert_eq!(tools.auto_update_ytdlp(&|_| {}).await.unwrap().as_deref(), Some(YTDLP_TAG));
    assert_eq!(std::fs::read(dir.path().join("yt-dlp")).unwrap(), tool(YTDLP_TAG));

    // Checked just now: no request until a week has passed.
    let requests = server.received_requests().await.unwrap().len();
    assert_eq!(tools.auto_update_ytdlp(&|_| {}).await.unwrap(), None);
    assert_eq!(server.received_requests().await.unwrap().len(), requests);
}

#[tokio::test]
async fn auto_update_leaves_a_missing_ytdlp_alone() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);
    assert_eq!(tools.auto_update_ytdlp(&|_| {}).await.unwrap(), None);
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn update_check_compares_installed_and_latest_releases() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("yt-dlp"), tool("2026.01.01")).unwrap();
    std::fs::write(dir.path().join("manifest.json"), r#"{"versions": {"ytdlp": "2026.01.01"}}"#).unwrap();
    let tools = manager(dir.path(), &server);

    let updates = tools.check_updates().await.unwrap();
    let ytdlp = updates.iter().find(|u| u.component == Component::Ytdlp).unwrap();
    assert_eq!((ytdlp.current.as_deref(), ytdlp.latest.as_str(), ytdlp.update_available), (Some("2026.01.01"), YTDLP_TAG, true));
    let deno = updates.iter().find(|u| u.component == Component::Deno).unwrap();
    assert_eq!((deno.current.as_deref(), deno.update_available), (None, true));
}

#[tokio::test]
async fn two_installs_at_once_download_once() {
    let server = honest_github().await;
    let dir = tempfile::tempdir().unwrap();
    let tools = manager(dir.path(), &server);

    let (a, b) = tokio::join!(tools.install(Component::Ytdlp, &|_| {}), tools.install(Component::Ytdlp, &|_| {}));
    assert_eq!((a.unwrap(), b.unwrap()), (YTDLP_TAG.to_string(), YTDLP_TAG.to_string()));
    let downloads = server
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.url.path().ends_with("/yt-dlp_linux"))
        .count();
    assert_eq!(downloads, 1);
}
