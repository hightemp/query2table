pub mod attachments;
pub mod commands;
pub mod orchestrator;
pub mod roles;
pub mod providers;
pub mod storage;
pub mod export;
pub mod utils;

use storage::db::Database;
use std::sync::Arc;
use tauri::Manager;

pub struct AppState {
    pub db: Arc<Database>,
}

/// Keeps the log writer alive on mobile, where logging starts once the app knows its folders.
#[cfg(mobile)]
struct LogGuard(#[allow(dead_code)] tracing_appender::non_blocking::WorkerGuard);

/// Tray menu items, relabelled when the interface language changes.
#[cfg(desktop)]
pub struct TrayMenu(pub std::sync::Mutex<Option<(tauri::menu::MenuItem<tauri::Wry>, tauri::menu::MenuItem<tauri::Wry>)>>);

/// Relabels the tray menu for the `ui_language` setting.
#[cfg(desktop)]
pub fn apply_tray_language(app: &tauri::AppHandle, setting: &str) {
    let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|name| std::env::var(name).ok().filter(|value| !value.is_empty()));
    let (show, quit) = utils::i18n::tray_labels(utils::i18n::resolve_language(Some(setting), locale.as_deref()));
    if let Some(tray) = app.try_state::<TrayMenu>() {
        if let Some((show_item, quit_item)) = tray.0.lock().ok().and_then(|items| items.clone()) {
            let _ = show_item.set_text(show);
            let _ = quit_item.set_text(quit);
        }
    }
}

/// There is no tray on mobile; the interface language needs no relabelling there.
#[cfg(mobile)]
pub fn apply_tray_language(_app: &tauri::AppHandle, _setting: &str) {}

/// Opens the database, upgrades it and tidies what earlier sessions left behind.
async fn open_database() -> Result<Database, String> {
    let db = Database::new().await.map_err(|e| format!("Failed to initialize database: {e}"))?;
    db.migrate().await.map_err(|e| format!("Failed to run migrations: {e}"))?;
    // Runs deleted in History can be restored only until the app closes.
    if let Err(e) = storage::repository::Repository::new(db.pool().clone()).purge_deleted_runs(None).await {
        tracing::warn!(error = %e, "Could not purge deleted runs");
    }
    // Drop attached files no run uses: unsent drafts older than a day and files of purged runs.
    let attachments = attachments::store::AttachmentStore::new(db.pool().clone(), attachments::store::AttachmentStore::default_dir());
    if let Err(e) = attachments.cleanup(24 * 3600).await {
        tracing::warn!(error = %e, "Could not clean up attached files");
    }
    Ok(db)
}

/// The tray icon with Show and Quit, in the saved interface language.
#[cfg(desktop)]
fn build_tray(app: &tauri::App, lang: utils::i18n::Lang) -> tauri::Result<()> {
    use tauri::{
        image::Image,
        menu::{MenuBuilder, MenuItemBuilder},
        tray::TrayIconBuilder,
    };
    let (show_label, quit_label) = utils::i18n::tray_labels(lang);
    let show_item = MenuItemBuilder::with_id("show", show_label).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", quit_label).build(app)?;
    app.manage(TrayMenu(std::sync::Mutex::new(Some((show_item.clone(), quit_item.clone())))));
    let tray_menu = MenuBuilder::new(app).item(&show_item).separator().item(&quit_item).build()?;

    let tray_icon = Image::from_path("icons/32x32.png").unwrap_or_else(|_| {
        Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("Failed to load tray icon")
    });

    TrayIconBuilder::new()
        .icon(tray_icon)
        .menu(&tray_menu)
        .tooltip("Query2Table")
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Capture HTTP_PROXY/HTTPS_PROXY for our reqwest clients and strip them
    // from the environment so the Tauri WebView (webkit2gtk) doesn't try to
    // route the dev-server / devtools traffic through the proxy.
    providers::http::proxy::init_and_strip_env();

    // Desktop logs start before anything else so all startup messages are captured; mobile
    // logging starts in `setup`, once the app knows its private storage.
    #[cfg(desktop)]
    let _log_guard = utils::logging::init_logging(utils::logging::log_dir());

    let run_controller = commands::run::RunController::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .manage(run_controller)
        .setup(|app| {
            #[cfg(mobile)]
            {
                utils::paths::set_app_data_dir(app.path().app_data_dir()?);
                app.manage(LogGuard(utils::logging::init_logging(utils::logging::log_dir())));
            }
            tracing::info!("Query2Table starting");

            let db = tauri::async_runtime::block_on(open_database())?;
            let settings: std::collections::HashMap<String, String> = tauri::async_runtime::block_on(db.get_all_settings())
                .map(|rows| rows.into_iter().collect())
                .unwrap_or_default();
            // Apply the user-selected proxy (if any) before any HTTP client is built.
            if let Some(url) = settings.get("active_proxy_url").filter(|url| !url.trim().is_empty()) {
                providers::http::set_runtime_proxy(Some(url.clone()));
            }
            app.manage(AppState { db: Arc::new(db) });

            // The tray menu is built before the frontend loads, in the saved interface language.
            #[cfg(desktop)]
            build_tray(app, utils::i18n::current_language(&settings))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::get_app_paths,
            commands::settings::copy_app_path,
            commands::settings::open_app_folder,
            commands::settings::update_setting,
            commands::settings::get_setting,
            commands::settings::list_ollama_cloud_models,
            commands::settings::list_openrouter_models,
            commands::run::start_run,
            commands::run::cancel_run,
            commands::run::pause_run,
            commands::run::resume_run,
            commands::run::confirm_schema,
            commands::run::get_run,
            commands::run::list_runs,
            commands::run::delete_run,
            commands::run::get_run_logs,
            commands::run::get_run_issues,
            commands::run::dismiss_run_notices,
            commands::run::get_run_schema,
            commands::run::get_run_rows,
            commands::run::get_row_sources,
            commands::settings::copy_text,
            commands::settings::paste_text,
            commands::attachments::add_attachments,
            commands::attachments::add_attachment_data,
            commands::attachments::get_attachments,
            commands::attachments::remove_attachment,
            commands::attachments::get_run_attachments,
            commands::attachments::open_attachment,
            commands::attachments::get_attachment_fragment,
            commands::attachments::get_vision_status,
            commands::run::get_image_results,
            commands::run::get_link_results,
            commands::run::set_link_visited,
            commands::run::set_link_hidden,
            commands::run::get_research_result,
            commands::run::ask_follow_up,
            commands::run::get_run_queries,
            commands::images::proxy_image,
            commands::images::save_image,
            commands::images::save_images,
            commands::export::export_run,
            commands::export::export_runs,
            commands::settings::update_settings,
            commands::settings::export_settings,
            commands::settings::read_settings_file,
            commands::connection::test_llm_connection,
            commands::connection::test_search_connection,
            commands::connection::test_proxy,
            commands::connection::list_ollama_models,
            commands::connection::list_openai_models,
            commands::run::list_history,
            commands::run::delete_runs,
            commands::run::restore_runs,
            commands::run::purge_runs,
            commands::run::rename_run,
            commands::run::pin_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
