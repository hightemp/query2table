//! The Tauri updater with the app's own updater settings (`tauri.conf.json`): it finds a release
//! in a `latest.json` like the one releases publish, accepts the file signed for that version,
//! and rejects a changed file or a signature made for another version.

use serde_json::{json, Value};
use tauri_plugin_updater::UpdaterExt;
use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};

// Signed with a throwaway key by `tauri signer sign --app-version 1.2.3` (see src/updates/signature.rs).
const FILE: &[u8] = b"Query2Table update test file\n";
const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY2QjQwMzAwNjRGNkMwQkIKUldTN3dQWmtBQU8wWnJHT0xKOXlHRkhGRmsyRzlUOFdRdUJib1JvUVJzUzU2SmxwSGcyTEtybnEK";
const SIGNED_123: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTN3dQWmtBQU8wWmxLVXg3KzZ2OTVPYlN3TTBuaU5rSlBzWEVVSGxaUWdkanNFREFDY1FnK2E2MmJZTVIzdmpTdzFrNTRwZXp3MHFNdjVUUDRodk9WQmRubGhteWFqdFF3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkyMzMxMzE4CWZpbGU6YXBwLmFwawl2ZXJzaW9uOjEuMi4zClRzSVE4VktROTNOVE9GWXY1YkdwR2NZbHJjeDlMd2p5UW9pd1RHSHZyNUxrU014K2d3SWRleUhZYmtPbXh5RXdZdmlzVmFPQXlsZ28ybzdkTWNMQkJ3PT0K";

/// The app's updater settings with the test key and the mock server as the release list.
fn updater_settings(server: &MockServer) -> Value {
    let config: Value = serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
    let mut updater = config["plugins"]["updater"].clone();
    assert_eq!(updater["requireSignedVersion"], json!(true), "releases must be bound to their version");
    updater["pubkey"] = json!(KEY);
    updater["endpoints"] = json!([format!("{}/latest.json", server.uri())]);
    updater["dangerousInsecureTransportProtocol"] = json!(true);
    updater
}

fn app(server: &MockServer) -> tauri::App<tauri::test::MockRuntime> {
    let mut context = tauri::test::mock_context(tauri::test::noop_assets());
    context.config_mut().plugins.0.insert("updater".into(), updater_settings(server));
    tauri::test::mock_builder()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .build(context)
        .unwrap()
}

/// A release list like the one published by the release workflow.
async fn publish(server: &MockServer, version: &str, file: &[u8]) {
    let url = format!("{}/Query2Table_{version}_amd64.AppImage", server.uri());
    let manifest = json!({
        "version": version,
        "notes": "### Added\n- Updates",
        "pub_date": "2026-10-20T10:00:00Z",
        "platforms": {
            "linux-x86_64": {"url": url, "signature": SIGNED_123},
            "linux-x86_64-appimage": {"url": url, "signature": SIGNED_123},
            "windows-x86_64": {"url": url, "signature": SIGNED_123},
            "darwin-aarch64": {"url": url, "signature": SIGNED_123},
            "android-universal": {"url": url, "signature": SIGNED_123}
        }
    });
    Mock::given(path("/latest.json")).respond_with(ResponseTemplate::new(200).set_body_json(manifest)).mount(server).await;
    Mock::given(path(format!("/Query2Table_{version}_amd64.AppImage")))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(file.to_vec()))
        .mount(server)
        .await;
}

#[tokio::test]
async fn a_signed_release_is_found_and_its_file_accepted() {
    let server = MockServer::start().await;
    publish(&server, "1.2.3", FILE).await;
    let app = app(&server);
    let update = app.updater_builder().no_proxy().build().unwrap().check().await.unwrap().expect("1.2.3 is newer than 0.1.0");
    assert_eq!((update.current_version.as_str(), update.version.as_str()), ("0.1.0", "1.2.3"));
    assert_eq!(update.body.as_deref(), Some("### Added\n- Updates"));
    let bytes = update.download(|_, _| {}, || {}).await.unwrap();
    assert_eq!(bytes, FILE);
}

#[tokio::test]
async fn a_changed_file_is_rejected() {
    let server = MockServer::start().await;
    publish(&server, "1.2.3", b"Query2Table update test file, changed\n").await;
    let update = app(&server).updater_builder().no_proxy().build().unwrap().check().await.unwrap().unwrap();
    let error = update.download(|_, _| {}, || {}).await.unwrap_err();
    assert!(matches!(error, tauri_plugin_updater::Error::Minisign(_)), "{error}");
}

#[tokio::test]
async fn an_older_signed_file_cannot_pass_for_a_new_version() {
    let server = MockServer::start().await;
    // The file and its signature are genuine, but they were made for 1.2.3.
    publish(&server, "2.0.0", FILE).await;
    let update = app(&server).updater_builder().no_proxy().build().unwrap().check().await.unwrap().unwrap();
    let error = update.download(|_, _| {}, || {}).await.unwrap_err();
    assert!(matches!(error, tauri_plugin_updater::Error::SignedVersionMismatch { .. }), "{error}");
}

#[tokio::test]
async fn the_same_version_is_no_update() {
    let server = MockServer::start().await;
    publish(&server, "0.1.0", FILE).await;
    assert!(app(&server).updater_builder().no_proxy().build().unwrap().check().await.unwrap().is_none());
}
