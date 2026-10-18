//! File work Android allows only through its own APIs (`AndroidFilesPlugin.kt`):
//! - saving several files at once: the app cannot write into a folder the user picks, so files go
//!   to the shared Pictures/Query2Table and Download/Query2Table folders instead;
//! - opening an attached file in another app, which needs a content address, not a path.

use tauri::{AppHandle, Runtime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedFolder {
    Pictures,
    Downloads,
}

/// MIME type the system files the export under.
pub fn mime_for_extension(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "csv" => "text/csv",
        "json" => "application/json",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "md" => "text/markdown",
        "txt" => "text/plain",
        "html" | "htm" => "text/html",
        "pdf" => "application/pdf",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application/vnd.ms-excel",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

#[cfg(target_os = "android")]
mod android {
    use super::SharedFolder;
    use serde::{Deserialize, Serialize};
    use tauri::{plugin::{Builder, PluginHandle, TauriPlugin}, AppHandle, Manager, Runtime};

    struct Handle<R: Runtime>(PluginHandle<R>);

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct CreateArgs<'a> {
        name: &'a str,
        mime_type: &'a str,
        folder: &'a str,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ViewArgs<'a> {
        path: &'a str,
        mime_type: &'a str,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ScanArgs<'a> {
        path: &'a str,
        mime_type: &'a str,
    }

    #[derive(Deserialize)]
    struct Created {
        uri: String,
        path: String,
    }

    pub fn init<R: Runtime>() -> TauriPlugin<R> {
        Builder::new("android-files")
            .setup(|app, api| {
                let handle = api.register_android_plugin("com.hightemp.query2table", "AndroidFilesPlugin")?;
                app.manage(Handle(handle));
                Ok(())
            })
            .build()
    }

    pub fn save<R: Runtime>(app: &AppHandle<R>, folder: SharedFolder, name: &str, mime: &str, bytes: &[u8]) -> Result<String, String> {
        let plugin = &app.state::<Handle<R>>().0;
        let folder = match folder {
            SharedFolder::Pictures => "pictures",
            SharedFolder::Downloads => "downloads",
        };
        let created: Created = plugin
            .run_mobile_plugin("create", CreateArgs { name, mime_type: mime, folder })
            .map_err(|e| e.to_string())?;
        crate::utils::files::write(app, &created.uri, bytes).map_err(|e| format!("Could not write {}: {e}", created.path))?;
        if !created.uri.contains("://") {
            let _ = plugin.run_mobile_plugin::<()>("scan", ScanArgs { path: &created.uri, mime_type: mime });
        }
        Ok(created.path)
    }

    pub fn view<R: Runtime>(app: &AppHandle<R>, path: &std::path::Path, mime: &str) -> Result<(), String> {
        let path = path.to_string_lossy();
        app.state::<Handle<R>>()
            .0
            .run_mobile_plugin::<()>("view", ViewArgs { path: &path, mime_type: mime })
            .map_err(|e| e.to_string())
    }
}

#[cfg(target_os = "android")]
pub use android::init;

/// Saves `bytes` as `name` in a shared folder; returns the folder and name to show the user.
#[allow(unused_variables)]
pub fn save<R: Runtime>(app: &AppHandle<R>, folder: SharedFolder, name: &str, bytes: &[u8]) -> Result<String, String> {
    let ext = name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
    #[cfg(target_os = "android")]
    return android::save(app, folder, name, mime_for_extension(ext), bytes);
    #[cfg(not(target_os = "android"))]
    Err("Choose a folder to save to.".into())
}

/// Opens `path` (a file in the app's cache folder) in the app the user picks for its type.
#[cfg(target_os = "android")]
pub fn view<R: Runtime>(app: &AppHandle<R>, path: &std::path::Path, mime: &str) -> Result<(), String> {
    android::view(app, path, mime)
}

#[cfg(test)]
mod tests {
    use super::mime_for_extension;

    #[test]
    fn exports_and_images_get_their_mime_types() {
        assert_eq!(mime_for_extension("XLSX"), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
        assert_eq!(mime_for_extension("md"), "text/markdown");
        assert_eq!(mime_for_extension("jpg"), "image/jpeg");
        assert_eq!(mime_for_extension("pdf"), "application/pdf");
        assert_eq!(mime_for_extension("docx"), "application/vnd.openxmlformats-officedocument.wordprocessingml.document");
        assert_eq!(mime_for_extension("bin"), "application/octet-stream");
    }
}
