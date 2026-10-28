//! Settings › Application › Components: the downloaded tools for videos (yt-dlp, Deno).

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::components::{Component, ComponentManager, ComponentStatus, UpdateInfo};

/// Install progress, `crate::components::Progress`.
pub const PROGRESS: &str = "components:progress";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ComponentsInfo {
    /// Tools can be downloaded for this system.
    pub supported: bool,
    /// Why they cannot.
    pub error: Option<String>,
    pub components: Vec<ComponentStatus>,
}

/// What the Components block shows.
pub fn components_info(manager: Result<&ComponentManager, String>) -> ComponentsInfo {
    match manager {
        Ok(manager) => ComponentsInfo { supported: true, error: None, components: manager.status() },
        Err(error) => ComponentsInfo { supported: false, error: Some(error), components: Vec::new() },
    }
}

#[tauri::command]
pub fn get_components() -> ComponentsInfo {
    components_info(crate::components::shared())
}

/// Downloads (or updates to) the latest release, reporting progress; returns its version.
#[tauri::command]
pub async fn install_component<R: Runtime>(app: AppHandle<R>, component: Component) -> Result<String, String> {
    let manager = crate::components::shared()?;
    manager
        .install(component, &|progress| {
            let _ = app.emit(PROGRESS, &progress);
        })
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_component(component: Component) -> Result<(), String> {
    crate::components::shared()?.remove(component).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_component_updates() -> Result<Vec<UpdateInfo>, String> {
    crate::components::shared()?.check_updates().await.map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::platform;

    #[test]
    fn the_block_lists_the_tools_or_says_why_there_are_none() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ComponentManager::with_platform(dir.path().to_owned(), platform::assets_for("linux", "x86_64").unwrap(), "http://unused", None).unwrap();
        std::fs::write(dir.path().join("yt-dlp"), "").unwrap();
        let info = components_info(Ok(&manager));
        assert!(info.supported && info.error.is_none());
        let installed: Vec<(Component, bool)> = info.components.iter().map(|c| (c.component, c.installed)).collect();
        assert_eq!(installed, [(Component::Ytdlp, true), (Component::Deno, false)]);

        let none = components_info(Err("freebsd/x86_64 is not available for this system".into()));
        assert_eq!(none, ComponentsInfo { supported: false, error: Some("freebsd/x86_64 is not available for this system".into()), components: Vec::new() });
    }

    #[test]
    fn statuses_reach_the_frontend_in_its_shape() {
        let dir = tempfile::tempdir().unwrap();
        let manager = ComponentManager::with_platform(dir.path().to_owned(), platform::assets_for("linux", "x86_64").unwrap(), "http://unused", None).unwrap();
        let json = serde_json::to_value(components_info(Ok(&manager))).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "supported": true,
                "error": null,
                "components": [
                    {"component": "ytdlp", "installed": false, "version": null, "bundled": false},
                    {"component": "deno", "installed": false, "version": null, "bundled": false}
                ]
            })
        );
    }
}
