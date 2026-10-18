//! App updates: finds a newer release in the signed `latest.json` of the GitHub releases, and
//! installs it after checking its signature. Desktop uses the Tauri updater; on Android the app
//! downloads the APK, checks it the same way and opens the system installer.

pub mod signature;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State};

/// A release newer than the running app.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    /// What changed (Markdown).
    pub notes: Option<String>,
    /// Release time, in seconds since 1970.
    pub date: Option<i64>,
    /// How the update installs: appimage, deb, rpm, msi, nsis, app, apk.
    pub installer: Option<String>,
}

/// Why checking or installing failed. `kind`: check, download, signature, install.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateError {
    pub kind: &'static str,
    pub message: String,
}

impl UpdateError {
    fn new(kind: &'static str, message: impl ToString) -> Self {
        Self { kind, message: message.to_string() }
    }
}

/// How an install went: `restart` (desktop: installed, restart to use it), `installer`
/// (Android: the system installer is open), `permission` (Android: allow installing apps first).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Installed {
    Restart,
    Installer,
    Permission,
}

/// Download progress, sent as the `update:progress` event.
#[derive(Debug, Clone, Serialize)]
struct Progress {
    downloaded: u64,
    total: Option<u64>,
}

const PROGRESS: &str = "update:progress";

/// The update found by the last check, kept for installing.
#[derive(Default)]
pub struct UpdateState(tokio::sync::Mutex<Option<Pending>>);

enum Pending {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    Desktop(tauri_plugin_updater::Update),
    #[allow(dead_code)]
    Android(AndroidRelease),
}

/// The Android entry of `latest.json`.
pub const ANDROID_PLATFORM: &str = "android-universal";

/// An APK release announced in `latest.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct AndroidRelease {
    pub version: String,
    pub notes: Option<String>,
    pub date: Option<i64>,
    pub url: String,
    pub signature: String,
}

/// The APK release in `manifest` when it is newer than `current`. Plain HTTP is refused unless
/// `insecure` (the updater's `dangerousInsecureTransportProtocol`, for local tests).
pub fn android_release(manifest: &serde_json::Value, current: &str, insecure: bool) -> Result<Option<AndroidRelease>, String> {
    let version = manifest["version"].as_str().ok_or("The release list has no version")?.trim_start_matches('v').to_string();
    let newer = semver::Version::parse(&version).map_err(|e| format!("Bad release version {version}: {e}"))?
        > semver::Version::parse(current.trim_start_matches('v')).map_err(|e| format!("Bad app version {current}: {e}"))?;
    if !newer {
        return Ok(None);
    }
    let platform = &manifest["platforms"][ANDROID_PLATFORM];
    let url = platform["url"].as_str().ok_or("The release has no Android app")?;
    let signature = platform["signature"].as_str().ok_or("The Android app of the release is not signed")?;
    if !url.starts_with("https://") && !(insecure && url.starts_with("http://")) {
        return Err(format!("The Android app is not offered over HTTPS: {url}"));
    }
    Ok(Some(AndroidRelease {
        version,
        notes: manifest["notes"].as_str().map(str::to_string).filter(|n| !n.trim().is_empty()),
        date: manifest["pub_date"]
            .as_str()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|d| d.timestamp()),
        url: url.to_string(),
        signature: signature.to_string(),
    }))
}

/// The `plugins.updater` settings of `tauri.conf.json` the Android updates use.
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
struct UpdaterConfig {
    public_key: String,
    endpoint: Option<String>,
    /// `dangerousInsecureTransportProtocol`: plain HTTP allowed (local tests only).
    insecure: bool,
}

#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
fn updater_config<R: Runtime>(app: &AppHandle<R>) -> UpdaterConfig {
    let config = app.config().plugins.0.get("updater").cloned().unwrap_or_default();
    UpdaterConfig {
        public_key: config["pubkey"].as_str().unwrap_or_default().to_string(),
        endpoint: config["endpoints"][0].as_str().map(str::to_string),
        insecure: config["dangerousInsecureTransportProtocol"].as_bool().unwrap_or(false),
    }
}

/// Looks for a newer release; remembers it for `install_update`.
#[tauri::command]
pub async fn check_update<R: Runtime>(app: AppHandle<R>, state: State<'_, UpdateState>) -> Result<Option<UpdateInfo>, UpdateError> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_updater::UpdaterExt;
        // `make dev` runs a build that is not installed; an update would replace that binary.
        if cfg!(debug_assertions) {
            return Err(UpdateError::new("check", "Development builds do not update"));
        }
        // The app's proxy (Settings › Network) applies to updates like to every other request.
        let builder = app.updater_builder();
        let builder = match crate::providers::http::proxy::proxy_url().and_then(|u| url::Url::parse(&u).ok()) {
            Some(proxy) => builder.proxy(proxy),
            None => builder.no_proxy(),
        };
        let updater = builder.build().map_err(|e| UpdateError::new("check", e))?;
        let found = updater.check().await.map_err(|e| UpdateError::new("check", e))?;
        let info = found.as_ref().map(|update| UpdateInfo {
            version: update.version.clone(),
            current_version: update.current_version.clone(),
            notes: update.body.clone().filter(|n| !n.trim().is_empty()),
            date: update.date.map(|d| d.unix_timestamp()),
            installer: tauri::utils::platform::bundle_type().map(|b| format!("{b:?}").to_lowercase()),
        });
        *state.0.lock().await = found.map(Pending::Desktop);
        Ok(info)
    }
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        let config = updater_config(&app);
        let endpoint = config.endpoint.ok_or_else(|| UpdateError::new("check", "No update address is configured"))?;
        if !endpoint.starts_with("https://") && !config.insecure {
            return Err(UpdateError::new("check", "The update address must use HTTPS"));
        }
        let client = http_client().map_err(|e| UpdateError::new("check", e))?;
        let response = client.get(&endpoint).send().await.map_err(|e| UpdateError::new("check", e))?;
        if !response.status().is_success() {
            return Err(UpdateError::new("check", format!("The release list answered HTTP {}", response.status())));
        }
        let manifest: serde_json::Value = response.json().await.map_err(|e| UpdateError::new("check", e))?;
        let current = app.package_info().version.to_string();
        let release = android_release(&manifest, &current, config.insecure).map_err(|e| UpdateError::new("check", e))?;
        let info = release.as_ref().map(|r| UpdateInfo {
            version: r.version.clone(),
            current_version: current.clone(),
            notes: r.notes.clone(),
            date: r.date,
            installer: Some("apk".into()),
        });
        *state.0.lock().await = release.map(Pending::Android);
        Ok(info)
    }
}

/// Downloads the update found by `check_update`, checks its signature and installs it.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>, state: State<'_, UpdateState>) -> Result<Installed, UpdateError> {
    let mut pending = state.0.lock().await;
    match pending.as_ref() {
        None => Err(UpdateError::new("check", "No update was found; check for updates again")),
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        Some(Pending::Desktop(update)) => {
            let mut downloaded = 0u64;
            let events = app.clone();
            let result = update
                .download_and_install(
                    move |chunk, total| {
                        downloaded += chunk as u64;
                        let _ = events.emit(PROGRESS, Progress { downloaded, total });
                    },
                    || {},
                )
                .await;
            match result {
                Ok(()) => {
                    *pending = None;
                    Ok(Installed::Restart)
                }
                Err(e) => Err(desktop_error(e)),
            }
        }
        Some(Pending::Android(release)) => {
            let release = release.clone();
            let installed = install_apk(&app, &release).await?;
            if installed == Installed::Installer {
                *pending = None;
            }
            Ok(installed)
        }
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn desktop_error(error: tauri_plugin_updater::Error) -> UpdateError {
    use tauri_plugin_updater::Error;
    let kind = match &error {
        Error::Minisign(_) | Error::Base64(_) | Error::SignatureUtf8(_) | Error::SignedVersionMismatch { .. } | Error::MissingSignedVersion => {
            "signature"
        }
        Error::Reqwest(_) | Error::Network(_) | Error::Http(_) => "download",
        _ => "install",
    };
    UpdateError::new(kind, error)
}

#[allow(dead_code)]
fn http_client() -> Result<reqwest::Client, String> {
    crate::providers::http::apply_proxy(reqwest::Client::builder().timeout(std::time::Duration::from_secs(600)))
        .build()
        .map_err(|e| e.to_string())
}

/// Largest APK accepted.
#[allow(dead_code)]
const APK_LIMIT: u64 = 400 * 1024 * 1024;

/// Downloads the APK, reporting progress (downloaded, total), and checks its signature;
/// returns its bytes.
#[cfg_attr(not(any(target_os = "android", target_os = "ios")), allow(dead_code))]
pub(crate) async fn download_verified(
    release: &AndroidRelease,
    public_key: &str,
    mut progress: impl FnMut(u64, Option<u64>),
) -> Result<Vec<u8>, UpdateError> {
    let client = http_client().map_err(|e| UpdateError::new("download", e))?;
    let mut response = client.get(&release.url).send().await.map_err(|e| UpdateError::new("download", e))?;
    if !response.status().is_success() {
        return Err(UpdateError::new("download", format!("The download answered HTTP {}", response.status())));
    }
    let total = response.content_length();
    if total.is_some_and(|t| t > APK_LIMIT) {
        return Err(UpdateError::new("download", "The update file is too large"));
    }
    let mut bytes = Vec::with_capacity(total.unwrap_or(0) as usize);
    while let Some(chunk) = response.chunk().await.map_err(|e| UpdateError::new("download", e))? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() as u64 > APK_LIMIT {
            return Err(UpdateError::new("download", "The update file is too large"));
        }
        progress(bytes.len() as u64, total);
    }
    signature::verify(&bytes, &release.signature, public_key, &release.version).map_err(|e| UpdateError::new("signature", e))?;
    Ok(bytes)
}

#[cfg(target_os = "android")]
async fn install_apk<R: Runtime>(app: &AppHandle<R>, release: &AndroidRelease) -> Result<Installed, UpdateError> {
    use tauri::Manager;
    let key = updater_config(app).public_key;
    let dir = app.path().app_cache_dir().map_err(|e| UpdateError::new("install", e))?.join("updates");
    let path = dir.join(format!("Query2Table_{}.apk", release.version));
    // A copy downloaded and checked before (e.g. before allowing installs) is used again.
    let ready = match tokio::fs::read(&path).await {
        Ok(bytes) => signature::verify(&bytes, &release.signature, &key, &release.version).is_ok(),
        Err(_) => false,
    };
    if !ready {
        let events = app.clone();
        let bytes = download_verified(release, &key, move |downloaded, total| {
            let _ = events.emit(PROGRESS, Progress { downloaded, total });
        })
        .await?;
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.map_err(|e| UpdateError::new("install", e))?;
        tokio::fs::write(&path, bytes).await.map_err(|e| UpdateError::new("install", e))?;
    }
    match crate::utils::android_files::install_apk(app, &path) {
        Ok(true) => Ok(Installed::Installer),
        Ok(false) => Ok(Installed::Permission),
        Err(e) => Err(UpdateError::new("install", e)),
    }
}

#[cfg(not(target_os = "android"))]
#[allow(dead_code)]
async fn install_apk<R: Runtime>(_app: &AppHandle<R>, _release: &AndroidRelease) -> Result<Installed, UpdateError> {
    Err(UpdateError::new("install", "Android updates install only on Android"))
}

/// Restarts the app after an update was installed.
#[tauri::command]
pub fn restart_app<R: Runtime>(app: AppHandle<R>) {
    app.restart();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest(version: &str) -> serde_json::Value {
        json!({
            "version": version,
            "notes": "## Added\n- Updates",
            "pub_date": "2026-10-20T10:00:00Z",
            "platforms": {
                "linux-x86_64": {"url": "https://github.com/x/app.AppImage", "signature": "sig-linux"},
                ANDROID_PLATFORM: {"url": "https://github.com/x/Query2Table_1.0.0_android.apk", "signature": "sig-apk"}
            }
        })
    }

    #[test]
    fn a_newer_release_offers_its_apk() {
        let release = android_release(&manifest("v1.0.0"), "0.9.0", false).unwrap().unwrap();
        assert_eq!(
            release,
            AndroidRelease {
                version: "1.0.0".into(),
                notes: Some("## Added\n- Updates".into()),
                date: Some(1_792_490_400),
                url: "https://github.com/x/Query2Table_1.0.0_android.apk".into(),
                signature: "sig-apk".into(),
            }
        );
    }

    #[test]
    fn the_same_or_older_version_is_no_update() {
        assert_eq!(android_release(&manifest("0.9.0"), "0.9.0", false), Ok(None));
        assert_eq!(android_release(&manifest("0.8.5"), "0.9.0", false), Ok(None));
        assert_eq!(android_release(&manifest("0.10.0"), "0.9.0", false).unwrap().unwrap().version, "0.10.0");
    }

    #[test]
    fn a_release_without_a_signed_https_apk_is_refused() {
        let mut unsigned = manifest("1.0.0");
        unsigned["platforms"][ANDROID_PLATFORM].as_object_mut().unwrap().remove("signature");
        assert_eq!(android_release(&unsigned, "0.9.0", false).unwrap_err(), "The Android app of the release is not signed");
        let mut plain = manifest("1.0.0");
        plain["platforms"][ANDROID_PLATFORM]["url"] = json!("http://example.com/app.apk");
        assert!(android_release(&plain, "0.9.0", false).unwrap_err().contains("HTTPS"));
        // Only a local test build allows plain HTTP.
        assert_eq!(android_release(&plain, "0.9.0", true).unwrap().unwrap().url, "http://example.com/app.apk");
        let mut desktop_only = manifest("1.0.0");
        desktop_only["platforms"].as_object_mut().unwrap().remove(ANDROID_PLATFORM);
        assert_eq!(android_release(&desktop_only, "0.9.0", false).unwrap_err(), "The release has no Android app");
        assert!(android_release(&json!({"platforms": {}}), "0.9.0", false).is_err());
    }

    // The signed test file of `signature::tests`.
    const FILE: &[u8] = b"Query2Table update test file\n";
    const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDY2QjQwMzAwNjRGNkMwQkIKUldTN3dQWmtBQU8wWnJHT0xKOXlHRkhGRmsyRzlUOFdRdUJib1JvUVJzUzU2SmxwSGcyTEtybnEK";
    const SIGNED_123: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHNpZ25hdHVyZSBmcm9tIHRhdXJpIHNlY3JldCBrZXkKUlVTN3dQWmtBQU8wWmxLVXg3KzZ2OTVPYlN3TTBuaU5rSlBzWEVVSGxaUWdkanNFREFDY1FnK2E2MmJZTVIzdmpTdzFrNTRwZXp3MHFNdjVUUDRodk9WQmRubGhteWFqdFF3PQp0cnVzdGVkIGNvbW1lbnQ6IHRpbWVzdGFtcDoxNzkyMzMxMzE4CWZpbGU6YXBwLmFwawl2ZXJzaW9uOjEuMi4zClRzSVE4VktROTNOVE9GWXY1YkdwR2NZbHJjeDlMd2p5UW9pd1RHSHZyNUxrU014K2d3SWRleUhZYmtPbXh5RXdZdmlzVmFPQXlsZ28ybzdkTWNMQkJ3PT0K";

    async fn serve(body: &[u8]) -> (wiremock::MockServer, AndroidRelease) {
        use wiremock::{matchers::path, Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(path("/app.apk")).respond_with(ResponseTemplate::new(200).set_body_bytes(body.to_vec())).mount(&server).await;
        let release = AndroidRelease {
            version: "1.2.3".into(),
            notes: None,
            date: None,
            url: format!("{}/app.apk", server.uri()),
            signature: SIGNED_123.into(),
        };
        (server, release)
    }

    #[tokio::test]
    async fn the_downloaded_apk_is_checked_before_use() {
        let (_server, release) = serve(FILE).await;
        let mut reported = Vec::new();
        let bytes = download_verified(&release, KEY, |done, total| reported.push((done, total))).await.unwrap();
        assert_eq!(bytes, FILE);
        assert_eq!(reported.last(), Some(&(FILE.len() as u64, Some(FILE.len() as u64))));

        let (_server, release) = serve(b"Query2Table update test file with something added\n").await;
        let error = download_verified(&release, KEY, |_, _| {}).await.unwrap_err();
        assert_eq!(error.kind, "signature");
        let (_server, mut release) = serve(FILE).await;
        release.version = "1.2.4".into();
        assert_eq!(download_verified(&release, KEY, |_, _| {}).await.unwrap_err().kind, "signature");
        release.url = format!("{}/missing.apk", _server.uri());
        assert_eq!(download_verified(&release, KEY, |_, _| {}).await.unwrap_err().kind, "download");
    }

    #[test]
    fn errors_reach_the_interface_with_their_kind() {
        let error = serde_json::to_value(UpdateError::new("signature", "bad")).unwrap();
        assert_eq!(error, json!({"kind": "signature", "message": "bad"}));
        assert_eq!(serde_json::to_value(Installed::Permission).unwrap(), json!("permission"));
    }
}
