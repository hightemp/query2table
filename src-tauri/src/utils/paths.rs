//! Where the app keeps its data. Desktop builds use the long-standing per-user folders; mobile
//! builds use the app's private storage, which Tauri reports once the app starts.

use std::path::PathBuf;
use std::sync::OnceLock;

static APP_DATA: OnceLock<PathBuf> = OnceLock::new();

/// Uses `dir` for all app data (mobile: the app's private storage). Set once, at startup.
pub fn set_app_data_dir(dir: PathBuf) {
    let _ = APP_DATA.set(dir);
}

/// The app's own data folder when the platform gives one (mobile), otherwise None.
pub fn app_data_override() -> Option<&'static PathBuf> {
    APP_DATA.get()
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_first_folder_set_wins() {
        // Only one test may touch the global; desktop builds never set it.
        super::set_app_data_dir("/data/app".into());
        super::set_app_data_dir("/other".into());
        assert_eq!(super::app_data_override().unwrap().to_str(), Some("/data/app"));
    }
}
