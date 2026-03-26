pub mod commands;
pub mod orchestrator;
pub mod roles;
pub mod providers;
pub mod storage;
pub mod export;
pub mod utils;

use storage::db::Database;
use std::sync::Arc;
use tauri::{
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Manager,
};

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

    let run_controller = commands::run::RunController::new();

    tracing::info!("Query2Table starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .manage(run_controller)
        .setup(|app| {
            let show_item = MenuItemBuilder::with_id("show", "Show Window").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
            let tray_menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let tray_icon = Image::from_path("icons/32x32.png").unwrap_or_else(|_| {
                Image::from_bytes(include_bytes!("../icons/32x32.png"))
                    .expect("Failed to load tray icon")
            });

            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&tray_menu)
                .tooltip("Query2Table")
                .on_menu_event(move |app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(w) = app.get_webview_window("main") {
                                let _ = w.show();
                                let _ = w.set_focus();
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_setting,
            commands::run::start_run,
            commands::run::cancel_run,
            commands::run::pause_run,
            commands::run::resume_run,
            commands::run::confirm_schema,
            commands::run::get_run,
            commands::run::list_runs,
            commands::run::delete_run,
            commands::run::get_run_logs,
            commands::export::export_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
