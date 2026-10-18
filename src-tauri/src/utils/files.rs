//! Files the user picked in a dialog. On Android the dialog returns `content://` addresses rather
//! than paths, which only the system can open; the fs plugin handles both.

use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

/// A dialog result as the fs plugin understands it: a URL (`content://`, `file://`) or a path.
pub fn picked(path: &str) -> FilePath {
    path.parse().unwrap_or_else(|_| FilePath::Path(path.into()))
}

pub fn read_text<R: Runtime>(app: &AppHandle<R>, path: &str) -> std::io::Result<String> {
    app.fs().read_to_string(picked(path))
}

/// Writes `bytes` to the picked file, replacing what was there.
pub fn write<R: Runtime>(app: &AppHandle<R>, path: &str, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    let mut file = app.fs().open(picked(path), options)?;
    file.write_all(bytes)?;
    file.flush()
}

/// A folder of its own for files made before they are copied elsewhere; removed when dropped.
pub struct Scratch(pub PathBuf);

impl Scratch {
    pub fn new<R: Runtime>(app: &AppHandle<R>) -> Result<Self, String> {
        let base = app.path().app_cache_dir().map_err(|e| e.to_string())?;
        let dir = base.join(format!("scratch-{}", crate::utils::id::new_id()));
        std::fs::create_dir_all(&dir).map_err(|e| format!("Could not create {}: {e}", dir.display()))?;
        Ok(Self(dir))
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Makes a file with `produce` (which writes to a local path) and puts it at `target`. Paths are
/// written directly; content addresses get a copy of a file made in a scratch folder first.
pub async fn write_through<R, F, Fut>(app: &AppHandle<R>, target: &str, ext: &str, produce: F) -> Result<(), String>
where
    R: Runtime,
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    match picked(target) {
        FilePath::Path(path) => produce(path).await,
        FilePath::Url(_) => {
            let scratch = Scratch::new(app)?;
            let local = scratch.0.join(format!("file.{ext}"));
            produce(local.clone()).await?;
            let bytes = std::fs::read(&local).map_err(|e| format!("Failed to read file: {e}"))?;
            write(app, target, &bytes).map_err(|e| format!("Failed to write file: {e}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_addresses_stay_urls_and_paths_stay_paths() {
        assert!(matches!(picked("content://com.android.providers.downloads.documents/document/12"), FilePath::Url(_)));
        assert!(matches!(picked("/home/user/settings.json"), FilePath::Path(_)));
        assert!(matches!(picked(r"C:\Users\me\settings.json"), FilePath::Path(_)));
    }
}
