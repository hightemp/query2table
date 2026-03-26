pub mod commands;
pub mod orchestrator;
pub mod roles;
pub mod providers;
pub mod storage;
pub mod export;
pub mod utils;

use storage::db::Database;
use std::sync::Arc;

pub struct AppState {
    pub db: Arc<Database>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging early (before Tauri setup) so all startup messages are captured.
    let data_dir = std::env::var("XDG_DATA_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".local").join("share"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let log_dir = data_dir
        .join("com.hightemp.query2table")
        .join("logs");
    let _log_guard = utils::logging::init_logging(log_dir);

    let runtime = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");

    let db = runtime.block_on(async {
        let db = Database::new().await.expect("Failed to initialize database");
        db.migrate().await.expect("Failed to run migrations");
        db
    });

    let app_state = AppState {
        db: Arc::new(db),
    };

    tracing::info!("Query2Table starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_setting,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
