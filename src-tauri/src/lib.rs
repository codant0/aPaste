mod commands;
mod clipboard;
mod history;
mod hotkey;
mod db;

use tauri::Manager;

#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("apaste.db");
            let conn = db::connection::open(&db_path)
                .expect("failed to open database");
            db::migrate::run(&conn).expect("failed to run migrations");

            let state = AppState {
                db: std::sync::Arc::new(std::sync::Mutex::new(conn)),
            };
            app.manage(state);

            // Start clipboard monitor
            clipboard::monitor::start(app.handle().clone());

            // Register global hotkey
            hotkey::register(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_history,
            commands::get_recent,
            commands::delete_item,
            commands::clear_all,
            commands::paste_item,
            commands::get_settings,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
