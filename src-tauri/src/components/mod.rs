//! External tools the app downloads for the user's system: yt-dlp, and Deno, the JavaScript
//! engine yt-dlp needs for YouTube. They come from their GitHub releases, are checked against
//! the published SHA-256 sums and tried once before they replace a working copy, and live in
//! `<data>/tools` with a manifest of their versions. On Android, Python and QuickJS come with the
//! APK (`bundled.rs`) and only the yt-dlp zipapp is downloaded.

mod bundled;
pub mod fetch;
mod fsutil;
pub mod platform;
pub mod process;

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub use bundled::Bundled;
use fetch::Http;
use platform::Assets;

const YTDLP_REPO: &str = "yt-dlp/yt-dlp";
const DENO_REPO: &str = "denoland/deno";
const GITHUB: &str = "https://github.com";

/// The version shown for tools inside the Android APK.
const BUNDLED_VERSION: &str = "bundled";
const MANIFEST_FILE: &str = "manifest.json";
/// yt-dlp follows the sites it reads; its releases are checked at most this often.
const AUTO_UPDATE_INTERVAL_SECS: i64 = 7 * 24 * 60 * 60;
/// The first start of a PyInstaller build unpacks itself, which can be slow on a cold disk.
const SMOKE_TEST_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Component {
    Ytdlp,
    Deno,
}

impl Component {
    pub const ALL: [Component; 2] = [Self::Ytdlp, Self::Deno];

    fn repo(self) -> &'static str {
        match self {
            Self::Ytdlp => YTDLP_REPO,
            Self::Deno => DENO_REPO,
        }
    }

    fn binary(self) -> &'static str {
        match self {
            Self::Ytdlp => "yt-dlp",
            Self::Deno => "deno",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ComponentError {
    #[error("{0} is not available for this system")]
    Unsupported(String),
    #[error("{} is not installed", .0.binary())]
    Missing(Component),
    #[error("network error: {0}")]
    Network(String),
    #[error("GitHub is limiting requests; try again later")]
    RateLimited,
    #[error("the checksum of {0} does not match")]
    Checksum(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

impl Serialize for ComponentError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComponentStatus {
    pub component: Component,
    pub installed: bool,
    pub version: Option<String>,
    /// Comes with the app (Android): never downloaded, updated or removed.
    pub bundled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Download,
    Verify,
    Install,
}

#[derive(Debug, Clone, Serialize)]
pub struct Progress {
    pub component: Component,
    pub stage: Stage,
    /// The release file being handled.
    pub file: String,
    pub downloaded: u64,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub component: Component,
    pub current: Option<String>,
    pub latest: String,
    pub update_available: bool,
}

/// The JavaScript engine yt-dlp is told to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsRuntime {
    Deno(PathBuf),
    QuickJs(PathBuf),
    /// The user's own yt-dlp, left to find an engine itself.
    System,
}

/// How to run yt-dlp: `program [prefix…] <arguments>` with `env` added.
#[derive(Debug, Clone)]
pub struct Tools {
    pub program: PathBuf,
    /// Android: the zipapp that Python runs.
    pub prefix: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub js_runtime: JsRuntime,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct Manifest {
    versions: HashMap<Component, String>,
    last_update_check: Option<i64>,
}

pub type ProgressFn<'a> = &'a (dyn Fn(Progress) + Send + Sync);

/// Downloads, checks, updates and removes the tools in one folder.
pub struct ComponentManager {
    dir: PathBuf,
    assets: Assets,
    /// `https://github.com`, or a test server.
    github: String,
    manifest: Mutex<Manifest>,
    /// One install at a time, so two never write the same temporary files.
    install_lock: tokio::sync::Mutex<()>,
    bundled: Option<Bundled>,
}

/// The app's tools in `<data>/tools`, shared by every run and command so installs never overlap.
/// An error when this system has no tools to download.
pub fn shared() -> Result<&'static ComponentManager, String> {
    static MANAGER: std::sync::OnceLock<Result<ComponentManager, String>> = std::sync::OnceLock::new();
    MANAGER
        .get_or_init(|| ComponentManager::new(crate::storage::db::Database::data_dir().join("tools")).map_err(|e| e.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

impl ComponentManager {
    /// The tools of this system in `dir`. On Android, Python's libraries start unpacking in the
    /// background; `tools()` waits for them.
    pub fn new(dir: PathBuf) -> Result<Self, ComponentError> {
        if cfg!(target_os = "android") {
            let bundled = Bundled::from_env().ok_or_else(|| ComponentError::Other("MainActivity did not set the bundled tool folders".into()))?;
            let background = bundled.clone();
            std::thread::spawn(move || {
                if let Err(e) = background.prepare() {
                    tracing::error!(error = %e, "unpacking the bundled tools failed");
                }
            });
            Self::with_platform(dir, platform::zipapp(), GITHUB, Some(bundled))
        } else {
            Self::with_platform(dir, platform::current()?, GITHUB, None)
        }
    }

    /// The tools of the given system, downloaded from `github`.
    pub fn with_platform(dir: PathBuf, assets: Assets, github: &str, bundled: Option<Bundled>) -> Result<Self, ComponentError> {
        fsutil::cleanup(&dir);
        let manifest = std::fs::read(dir.join(MANIFEST_FILE)).ok().and_then(|raw| serde_json::from_slice(&raw).ok()).unwrap_or_default();
        Ok(Self { dir, assets, github: github.to_string(), manifest: Mutex::new(manifest), install_lock: tokio::sync::Mutex::new(()), bundled })
    }

    /// A client for the current proxy setting, so a changed setting applies at once.
    fn http(&self) -> Result<Http, ComponentError> {
        Http::new(&self.github)
    }

    fn manifest(&self) -> std::sync::MutexGuard<'_, Manifest> {
        self.manifest.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn update_manifest(&self, f: impl FnOnce(&mut Manifest)) -> Result<(), ComponentError> {
        let mut manifest = self.manifest();
        f(&mut manifest);
        std::fs::create_dir_all(&self.dir)?;
        let tmp = self.tmp_path(MANIFEST_FILE);
        std::fs::write(&tmp, serde_json::to_vec_pretty(&*manifest).map_err(|e| ComponentError::Other(e.to_string()))?)?;
        std::fs::rename(tmp, self.dir.join(MANIFEST_FILE))?;
        Ok(())
    }

    fn binary_path(&self, c: Component) -> PathBuf {
        self.dir.join(format!("{}{}", c.binary(), self.assets.exe_ext))
    }

    fn tmp_path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{}{name}", fsutil::TMP_PREFIX))
    }

    /// Comes with the app: never downloaded, updated or removed.
    fn is_bundled(&self, c: Component) -> bool {
        self.bundled.is_some() && c == Component::Deno
    }

    fn is_installed(&self, c: Component) -> bool {
        match (&self.bundled, c) {
            (Some(b), Component::Deno) => b.quickjs().is_file(),
            _ => self.binary_path(c).is_file(),
        }
    }

    fn has_release(&self, c: Component, tag: &str) -> bool {
        self.is_installed(c) && self.manifest().versions.get(&c).map(String::as_str) == Some(tag)
    }

    pub fn status(&self) -> Vec<ComponentStatus> {
        let manifest = self.manifest();
        Component::ALL
            .into_iter()
            .map(|component| {
                let installed = self.is_installed(component);
                let bundled = self.is_bundled(component);
                let version = if bundled { Some(BUNDLED_VERSION.to_string()) } else { manifest.versions.get(&component).cloned() };
                ComponentStatus { component, installed, version: version.filter(|_| installed), bundled }
            })
            .collect()
    }

    /// The user's own yt-dlp at `program`, with our Deno when it is installed.
    pub fn custom_tools(&self, program: PathBuf) -> Tools {
        let js_runtime = match &self.bundled {
            Some(b) if b.quickjs().is_file() => JsRuntime::QuickJs(b.quickjs()),
            None if self.is_installed(Component::Deno) => JsRuntime::Deno(self.binary_path(Component::Deno)),
            _ => JsRuntime::System,
        };
        Tools { program, prefix: Vec::new(), env: Vec::new(), js_runtime }
    }

    /// How to run yt-dlp; fails while a tool is missing.
    pub fn tools(&self) -> Result<Tools, ComponentError> {
        if let Some(missing) = Component::ALL.into_iter().find(|c| !self.is_installed(*c)) {
            return Err(ComponentError::Missing(missing));
        }
        Ok(match &self.bundled {
            Some(b) => {
                b.prepare()?;
                Tools { program: b.python(), prefix: vec![self.binary_path(Component::Ytdlp).into()], env: b.env(), js_runtime: JsRuntime::QuickJs(b.quickjs()) }
            }
            None => Tools { program: self.binary_path(Component::Ytdlp), prefix: Vec::new(), env: Vec::new(), js_runtime: JsRuntime::Deno(self.binary_path(Component::Deno)) },
        })
    }

    /// Installs the missing tools, then tells how to run yt-dlp.
    pub async fn ensure(&self, on_progress: ProgressFn<'_>) -> Result<Tools, ComponentError> {
        for c in Component::ALL {
            if !self.is_installed(c) {
                self.install(c, on_progress).await?;
            }
        }
        self.tools()
    }

    /// Installs (or reinstalls) the latest release of `c`; returns its version.
    pub async fn install(&self, c: Component, on_progress: ProgressFn<'_>) -> Result<String, ComponentError> {
        if self.is_bundled(c) {
            return Ok(BUNDLED_VERSION.into());
        }
        let (_guard, waited) = match self.install_lock.try_lock() {
            Ok(guard) => (guard, false),
            Err(_) => (self.install_lock.lock().await, true),
        };
        let tag = self.http()?.latest_tag(c.repo()).await?;
        // The install we waited for may have just put this release in place.
        if waited && self.has_release(c, &tag) {
            return Ok(tag);
        }
        tracing::info!(component = ?c, tag, "installing a tool");
        std::fs::create_dir_all(&self.dir)?;
        match c {
            Component::Ytdlp => self.install_ytdlp(&tag, on_progress).await?,
            Component::Deno => self.install_deno(&tag, on_progress).await?,
        }
        self.update_manifest(|m| {
            m.versions.insert(c, tag.clone());
        })?;
        tracing::info!(component = ?c, tag, "tool installed");
        Ok(tag)
    }

    /// Removes a downloaded tool; it is downloaded again when needed.
    pub fn remove(&self, c: Component) -> Result<(), ComponentError> {
        if self.is_bundled(c) {
            return Err(ComponentError::Other(format!("{} comes with the app", c.binary())));
        }
        fsutil::remove(&self.binary_path(c))?;
        self.update_manifest(|m| {
            m.versions.remove(&c);
        })
    }

    /// Downloads one release file into a temporary file and checks its SHA-256.
    async fn fetch_asset(&self, c: Component, tag: &str, asset: &str, expected: &str, on_progress: ProgressFn<'_>) -> Result<PathBuf, ComponentError> {
        let http = self.http()?;
        let tmp = self.tmp_path(asset);
        let progress = |stage, downloaded, total| on_progress(Progress { component: c, stage, file: asset.to_string(), downloaded, total });
        let result = async {
            http.download(&http.asset_url(c.repo(), tag, asset), &tmp, |downloaded, total| progress(Stage::Download, downloaded, total)).await?;
            progress(Stage::Verify, 0, None);
            fetch::verify(&tmp, expected, asset).await
        }
        .await;
        match result {
            Ok(()) => Ok(tmp),
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }

    async fn install_ytdlp(&self, tag: &str, on_progress: ProgressFn<'_>) -> Result<(), ComponentError> {
        let c = Component::Ytdlp;
        let asset = self.assets.ytdlp;
        let http = self.http()?;
        let sums = http.get_text(&http.asset_url(c.repo(), tag, "SHA2-256SUMS")).await?;
        let expected = fetch::find_checksum(&sums, asset).ok_or_else(|| ComponentError::Checksum(format!("{asset} (missing from SHA2-256SUMS)")))?;
        let tmp = self.fetch_asset(c, tag, asset, &expected, on_progress).await?;
        self.finish(c, asset, &tmp, on_progress).await
    }

    async fn install_deno(&self, tag: &str, on_progress: ProgressFn<'_>) -> Result<(), ComponentError> {
        let c = Component::Deno;
        let asset = format!("deno-{}.zip", self.assets.deno_triple);
        let http = self.http()?;
        let sums = http.get_text(&http.asset_url(c.repo(), tag, &format!("{asset}.sha256sum"))).await?;
        let expected = fetch::find_checksum(&sums, &asset).ok_or_else(|| ComponentError::Checksum(format!("{asset} (missing checksum)")))?;
        let zip_path = self.fetch_asset(c, tag, &asset, &expected, on_progress).await?;

        let entry = format!("deno{}", self.assets.exe_ext);
        let tmp = self.tmp_path(&entry);
        let extracted = {
            let (zip_path, tmp, entry) = (zip_path.clone(), tmp.clone(), entry.clone());
            tokio::task::spawn_blocking(move || extract_zip_entry(&zip_path, &entry, &tmp)).await.map_err(|e| ComponentError::Other(e.to_string()))?
        };
        let _ = std::fs::remove_file(&zip_path);
        if let Err(e) = extracted {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        self.finish(c, &asset, &tmp, on_progress).await
    }

    /// Tries the new tool once and puts it in place; a tool that does not run is thrown away
    /// and the working one kept.
    async fn finish(&self, c: Component, asset: &str, tmp: &Path, on_progress: ProgressFn<'_>) -> Result<(), ComponentError> {
        on_progress(Progress { component: c, stage: Stage::Install, file: asset.to_string(), downloaded: 0, total: None });
        let result = async {
            if let Some(b) = &self.bundled {
                b.prepare()?;
            }
            fsutil::prepare(tmp).await?;
            smoke_test(self.version_command(tmp), c).await?;
            fsutil::replace(tmp, &self.binary_path(c))
        }
        .await;
        if result.is_err() {
            let _ = std::fs::remove_file(tmp);
        }
        result
    }

    /// `<tool> --version`, or `python <zipapp> --version` on Android.
    fn version_command(&self, path: &Path) -> tokio::process::Command {
        let mut cmd = match &self.bundled {
            Some(b) => {
                let mut cmd = process::command(&b.python());
                cmd.arg(path).envs(b.env());
                cmd
            }
            None => process::command(path),
        };
        cmd.arg("--version");
        cmd
    }

    pub async fn check_updates(&self) -> Result<Vec<UpdateInfo>, ComponentError> {
        let mut result = Vec::with_capacity(Component::ALL.len());
        for component in Component::ALL {
            if self.is_bundled(component) {
                continue;
            }
            let latest = self.http()?.latest_tag(component.repo()).await?;
            let current = self.status().into_iter().find(|s| s.component == component).and_then(|s| s.version);
            result.push(UpdateInfo { component, update_available: current.as_deref() != Some(latest.as_str()), current, latest });
        }
        self.update_manifest(|m| m.last_update_check = Some(now()))?;
        Ok(result)
    }

    /// Updates an installed yt-dlp when its releases were last checked over a week ago. Returns
    /// the new version when one was installed.
    pub async fn auto_update_ytdlp(&self, on_progress: ProgressFn<'_>) -> Result<Option<String>, ComponentError> {
        if !self.is_installed(Component::Ytdlp) {
            return Ok(None);
        }
        if now() - self.manifest().last_update_check.unwrap_or(0) < AUTO_UPDATE_INTERVAL_SECS {
            return Ok(None);
        }
        let latest = self.http()?.latest_tag(YTDLP_REPO).await?;
        self.update_manifest(|m| m.last_update_check = Some(now()))?;
        let current = self.manifest().versions.get(&Component::Ytdlp).cloned();
        if current.as_deref() == Some(latest.as_str()) {
            return Ok(None);
        }
        tracing::info!(?current, latest, "updating yt-dlp");
        self.install(Component::Ytdlp, on_progress).await.map(Some)
    }
}

/// Runs a new tool once before it replaces the current one, so a broken download, a missing
/// loader or an antivirus block fails the install and keeps the working version.
async fn smoke_test(mut cmd: tokio::process::Command, c: Component) -> Result<(), ComponentError> {
    let name = c.binary();
    let output = tokio::time::timeout(SMOKE_TEST_TIMEOUT, process::output(&mut cmd))
        .await
        .map_err(|_| ComponentError::Other(format!("{name} did not respond")))??;
    if !output.status.success() {
        return Err(ComponentError::Other(format!("{name} failed to run: {}", String::from_utf8_lossy(&output.stderr).trim())));
    }
    Ok(())
}

fn extract_zip_entry(zip_path: &Path, entry: &str, dest: &Path) -> Result<(), ComponentError> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(zip_path)?).map_err(|e| ComponentError::Other(format!("bad archive: {e}")))?;
    let mut src = archive.by_name(entry).map_err(|e| ComponentError::Other(format!("{entry} is not in the archive: {e}")))?;
    let mut out = std::fs::File::create(dest)?;
    std::io::copy(&mut src, &mut out)?;
    out.sync_all()?;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn script(path: &Path, body: &str) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    /// The Android layout: native libraries from the APK, with a fake Python that runs the
    /// "zipapp" as a shell script.
    fn android_manager(dir: &Path) -> ComponentManager {
        let lib_dir = dir.join("lib");
        std::fs::create_dir_all(&lib_dir).unwrap();
        let mut zip = zip::ZipWriter::new(std::fs::File::create(lib_dir.join("libpython.zip.so")).unwrap());
        zip.add_directory("usr/lib/", zip::write::SimpleFileOptions::default()).unwrap();
        zip.finish().unwrap();
        script(&lib_dir.join("libpython.so"), "exec /bin/sh \"$@\"");
        script(&lib_dir.join("libqjs.so"), "true");
        let bundled = Bundled::new(lib_dir, dir.join("packages"));
        ComponentManager::with_platform(dir.join("tools"), platform::zipapp(), "http://unused", Some(bundled)).unwrap()
    }

    /// On Android QuickJS comes with the APK; only the yt-dlp zipapp is downloaded, and Python
    /// runs it.
    #[tokio::test]
    async fn android_runs_the_ytdlp_zipapp_with_bundled_tools() {
        let dir = tempfile::tempdir().unwrap();
        let manager = android_manager(dir.path());
        let status = manager.status();
        let deno = status.iter().find(|s| s.component == Component::Deno).unwrap();
        assert!(deno.installed && deno.bundled);
        assert_eq!(deno.version.as_deref(), Some(BUNDLED_VERSION));
        assert!(!status.iter().find(|s| s.component == Component::Ytdlp).unwrap().installed);
        assert!(matches!(manager.tools(), Err(ComponentError::Missing(Component::Ytdlp))));
        // A bundled tool needs no download, so this works offline.
        assert_eq!(manager.install(Component::Deno, &|_| {}).await.unwrap(), BUNDLED_VERSION);
        assert!(manager.remove(Component::Deno).is_err());

        std::fs::create_dir_all(dir.path().join("tools")).unwrap();
        let zipapp = dir.path().join("tools/yt-dlp");
        std::fs::write(&zipapp, "echo 2026.10.26").unwrap();
        let tools = manager.tools().unwrap();
        // Python's libraries are unpacked before anything runs.
        assert!(dir.path().join("packages/python/usr/lib").is_dir());
        let lib = dir.path().join("lib");
        assert_eq!(tools.program, lib.join("libpython.so"));
        assert_eq!(tools.prefix, vec![zipapp.into_os_string()]);
        assert_eq!(tools.js_runtime, JsRuntime::QuickJs(lib.join("libqjs.so")));
        assert!(tools.env.iter().any(|(k, _)| k == "PYTHONHOME"));
    }

    /// A new zipapp is tried through Python before it replaces the working one.
    #[tokio::test]
    async fn android_tries_ytdlp_through_python() {
        let dir = tempfile::tempdir().unwrap();
        let manager = android_manager(dir.path());
        std::fs::create_dir_all(dir.path().join("tools")).unwrap();
        let tmp = manager.tmp_path("yt-dlp");
        std::fs::write(&tmp, "exit 1").unwrap();
        assert!(manager.finish(Component::Ytdlp, "yt-dlp", &tmp, &|_| {}).await.is_err());
        assert!(!manager.binary_path(Component::Ytdlp).exists());

        std::fs::write(&tmp, "[ \"$1\" = --version ] && echo 2026.10.26").unwrap();
        manager.finish(Component::Ytdlp, "yt-dlp", &tmp, &|_| {}).await.unwrap();
        assert!(manager.binary_path(Component::Ytdlp).is_file());
    }

    /// The checksum files of the latest releases list the file of every supported system, in a
    /// form we can read: `cargo test checksums_for_every_system -- --ignored`.
    #[tokio::test]
    #[ignore = "network"]
    async fn checksums_for_every_system() {
        let http = Http::new(GITHUB).unwrap();
        let ytdlp_tag = http.latest_tag(YTDLP_REPO).await.unwrap();
        let ytdlp_sums = http.get_text(&http.asset_url(YTDLP_REPO, &ytdlp_tag, "SHA2-256SUMS")).await.unwrap();
        let deno_tag = http.latest_tag(DENO_REPO).await.unwrap();
        let mut missing = Vec::new();
        for (os, arch) in platform::SUPPORTED {
            let assets = platform::assets_for(os, arch).unwrap();
            if fetch::find_checksum(&ytdlp_sums, assets.ytdlp).is_none() {
                missing.push(format!("{os}/{arch}: {}", assets.ytdlp));
            }
            let deno_asset = format!("deno-{}.zip", assets.deno_triple);
            let deno_sums = http.get_text(&http.asset_url(DENO_REPO, &deno_tag, &format!("{deno_asset}.sha256sum"))).await.unwrap();
            if fetch::find_checksum(&deno_sums, &deno_asset).is_none() {
                missing.push(format!("{os}/{arch}: {deno_asset} in {deno_sums:?}"));
            }
        }
        if fetch::find_checksum(&ytdlp_sums, platform::zipapp().ytdlp).is_none() {
            missing.push("android: yt-dlp".into());
        }
        assert!(missing.is_empty(), "no checksum for:\n{}", missing.join("\n"));
    }

    /// Real downloads for this system: `cargo test live_install -- --ignored`.
    #[tokio::test]
    #[ignore = "network"]
    async fn live_install() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ComponentManager::new(dir.path().to_owned()).unwrap();
        let tools = manager.ensure(&|_| {}).await.unwrap();
        let version = process::output(process::command(&tools.program).arg("--version")).await.unwrap();
        assert!(version.status.success());
        println!("{}", String::from_utf8_lossy(&version.stdout));
        assert!(manager.status().iter().all(|s| s.installed && s.version.is_some()));
    }
}
