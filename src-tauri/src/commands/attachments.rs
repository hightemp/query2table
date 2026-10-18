//! IPC for attaching files to a query and reading them back.

use serde::Serialize;
use tauri::State;
#[cfg(not(target_os = "android"))]
use tauri_plugin_opener::OpenerExt;

use crate::attachments::retrieve;
use crate::attachments::store::{AttachError, AttachmentStore, DEFAULT_MAX_MB};
use crate::attachments::{AttachmentInfo, Locator};
use crate::AppState;

async fn store(state: &AppState) -> AttachmentStore {
    let max_mb = state
        .db
        .get_setting("attachment_max_mb")
        .await
        .ok()
        .flatten()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(DEFAULT_MAX_MB);
    AttachmentStore::new(state.db.pool().clone(), AttachmentStore::default_dir()).with_max_mb(max_mb)
}

/// The outcome for one file: the attachment, or why it was refused.
#[derive(Debug, Serialize)]
pub struct AttachResult {
    pub attachment: Option<AttachmentInfo>,
    pub error: Option<AttachError>,
}

impl From<Result<AttachmentInfo, AttachError>> for AttachResult {
    fn from(result: Result<AttachmentInfo, AttachError>) -> Self {
        match result {
            Ok(attachment) => Self { attachment: Some(attachment), error: None },
            Err(error) => Self { attachment: None, error: Some(error) },
        }
    }
}

/// Attaches files chosen in the file dialog or dropped on the window.
#[tauri::command]
pub async fn add_attachments(state: State<'_, AppState>, paths: Vec<String>) -> Result<Vec<AttachResult>, String> {
    let store = store(&state).await;
    let mut out = Vec::new();
    for path in paths {
        out.push(store.add_path(std::path::Path::new(&path)).await.into());
    }
    Ok(out)
}

/// Attaches file content (pasted, dropped, or picked on Android). The raw body is the file and the
/// `x-file-name` header (URI-encoded) names it. Android's IPC cannot carry raw bytes, so there the
/// body is `{ name, data }` with the file in base64.
#[tauri::command]
pub async fn add_attachment_data(state: State<'_, AppState>, request: tauri::ipc::Request<'_>) -> Result<AttachResult, String> {
    let header_name = request
        .headers()
        .get("x-file-name")
        .and_then(|v| v.to_str().ok())
        .map(|v| url::form_urlencoded::parse(format!("n={v}").as_bytes()).next().map(|(_, n)| n.into_owned()).unwrap_or_default());
    let (name, bytes) = attachment_body(request.body(), header_name)?;
    Ok(store(&state).await.add_bytes(&name, bytes).await.into())
}

/// The file name and content of an `add_attachment_data` request.
fn attachment_body(body: &tauri::ipc::InvokeBody, header_name: Option<String>) -> Result<(String, Vec<u8>), String> {
    use base64::Engine;
    let (name, bytes) = match body {
        tauri::ipc::InvokeBody::Raw(bytes) => (header_name, bytes.clone()),
        tauri::ipc::InvokeBody::Json(value) => {
            let data = value.get("data").and_then(|d| d.as_str()).ok_or("Expected file content")?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .map_err(|e| format!("The file content is damaged: {e}"))?;
            let name = value.get("name").and_then(|n| n.as_str()).map(str::to_string).or(header_name);
            (name, bytes)
        }
    };
    let name = name.filter(|n| !n.is_empty()).unwrap_or_else(|| "pasted".to_string());
    Ok((name, bytes))
}

/// Attachments by id, e.g. to restore a saved draft; unknown ids are skipped.
#[tauri::command]
pub async fn get_attachments(state: State<'_, AppState>, ids: Vec<String>) -> Result<Vec<AttachmentInfo>, String> {
    let store = store(&state).await;
    store.touch_drafts(&ids).await.map_err(|e| e.to_string())?;
    store.get(&ids).await.map_err(|e| e.to_string())
}

/// Removes a draft attachment (files already used by a run stay).
#[tauri::command]
pub async fn remove_attachment(state: State<'_, AppState>, id: String) -> Result<(), String> {
    store(&state).await.remove_draft(&id).await.map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct RunAttachment {
    pub turn_index: i64,
    pub attachment: AttachmentInfo,
}

#[tauri::command]
pub async fn get_run_attachments(state: State<'_, AppState>, run_id: String) -> Result<Vec<RunAttachment>, String> {
    Ok(store(&state)
        .await
        .for_run(&run_id)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(turn_index, attachment)| RunAttachment { turn_index, attachment })
        .collect())
}

/// Opens an attached file in its usual application, under its original name.
#[tauri::command]
pub async fn open_attachment(app: tauri::AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let (file_name, stored) = store(&state)
        .await
        .stored_path(&id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("The file is no longer available")?;
    // Another app gets a copy under the original name. On Android it lives in the app's cache,
    // which the FileProvider shares; the system temp folder is not writable there.
    #[cfg(target_os = "android")]
    let base = {
        use tauri::Manager;
        app.path().app_cache_dir().map_err(|e| e.to_string())?
    };
    #[cfg(not(target_os = "android"))]
    let base = std::env::temp_dir();
    let copy = opened_copy_path(&base, &id, &file_name);
    if let Some(folder) = copy.parent() {
        tokio::fs::create_dir_all(folder).await.map_err(|e| e.to_string())?;
    }
    tokio::fs::copy(&stored, &copy).await.map_err(|e| format!("Could not open the file: {e}"))?;
    #[cfg(target_os = "android")]
    {
        let ext = file_name.rsplit_once('.').map(|(_, ext)| ext).unwrap_or("");
        let mime = match crate::utils::android_files::mime_for_extension(ext) {
            // Unknown types let the user pick any app.
            "application/octet-stream" => "*/*",
            mime => mime,
        };
        crate::utils::android_files::view(&app, &copy, mime).map_err(|e| format!("Could not open the file: {e}"))
    }
    #[cfg(not(target_os = "android"))]
    app.opener()
        .open_path(copy.to_string_lossy().into_owned(), None::<&str>)
        .map_err(|e| format!("Could not open the file: {e}"))
}

/// Where an attachment is copied to be opened in another app: its own folder under `base`, so the
/// copy keeps the original name.
fn opened_copy_path(base: &std::path::Path, id: &str, file_name: &str) -> std::path::PathBuf {
    base.join("query2table-files").join(id).join(file_name)
}

#[derive(Debug, Serialize)]
pub struct AttachmentFragment {
    pub attachment_id: String,
    pub file_name: String,
    pub locator: Locator,
    pub text: String,
}

/// The text at an `attachment://` source, for the sources panel.
#[tauri::command]
pub async fn get_attachment_fragment(state: State<'_, AppState>, url: String) -> Result<Option<AttachmentFragment>, String> {
    let Some((id, place)) = Locator::from_url(&url) else { return Ok(None) };
    let hits = retrieve::at(state.db.pool(), &id, &place, 3).await.map_err(|e| e.to_string())?;
    let Some(first) = hits.first() else { return Ok(None) };
    Ok(Some(AttachmentFragment {
        attachment_id: id,
        file_name: first.file_name.clone(),
        locator: place,
        text: hits.iter().map(|h| h.text.as_str()).collect::<Vec<_>>().join("\n\n"),
    }))
}

#[derive(Debug, Serialize)]
pub struct VisionStatus {
    /// The main model gets images directly.
    pub main_sees: bool,
    /// What the provider's catalog says about the main model (None: unknown).
    pub detected: Option<bool>,
    /// The model that reads images and scans; None when no model can.
    pub reader: Option<String>,
}

/// Who would read images with the saved settings, optionally with unsaved edits applied.
#[tauri::command]
pub async fn get_vision_status(
    state: State<'_, AppState>,
    overrides: Option<std::collections::HashMap<String, String>>,
) -> Result<VisionStatus, String> {
    let mut settings: std::collections::HashMap<String, String> =
        state.db.get_all_settings().await.map_err(|e| e.to_string())?.into_iter().collect();
    settings.extend(overrides.unwrap_or_default());
    let plan = crate::providers::llm::capabilities::VisionPlan::resolve(&settings).await;
    Ok(VisionStatus { main_sees: plan.main_sees, detected: plan.detected, reader: plan.reader })
}

#[cfg(test)]
mod body_tests {
    use tauri::ipc::InvokeBody;

    #[test]
    fn opened_copies_keep_the_name_in_a_folder_of_their_own() {
        let path = super::opened_copy_path(std::path::Path::new("/cache"), "att1", "Report 2025.pdf");
        assert_eq!(path, std::path::Path::new("/cache/query2table-files/att1/Report 2025.pdf"));
    }

    #[test]
    fn file_content_comes_raw_or_in_base64() {
        let raw = InvokeBody::Raw(b"abc".to_vec());
        assert_eq!(super::attachment_body(&raw, Some("a.txt".into())).unwrap(), ("a.txt".to_string(), b"abc".to_vec()));
        assert_eq!(super::attachment_body(&raw, None).unwrap().0, "pasted");
        let json = InvokeBody::Json(serde_json::json!({ "name": "notes.txt", "data": "aGVsbG8=" }));
        assert_eq!(super::attachment_body(&json, None).unwrap(), ("notes.txt".to_string(), b"hello".to_vec()));
        assert!(super::attachment_body(&InvokeBody::Json(serde_json::json!([1, 2])), None).is_err());
        assert!(super::attachment_body(&InvokeBody::Json(serde_json::json!({ "data": "%%%" })), None).unwrap_err().contains("damaged"));
    }
}
