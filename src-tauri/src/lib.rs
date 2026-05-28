mod commands;
mod clipboard;
mod history;
mod hotkey;
mod db;

use tauri::Manager;
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, shortcut, event| {
                    use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
                    if event.state == ShortcutState::Pressed {
                        let is_match = shortcut.matches(Modifiers::SUPER, Code::KeyV)
                            || shortcut.matches(Modifiers::SUPER | Modifiers::SHIFT, Code::KeyV);
                        if is_match {
                            hotkey::show_popup(_app);
                        }
                    }
                })
                .build(),
        )
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

            // Build tray menu
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .item(&settings_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("aPaste")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "settings" => {
                            log::info!("Settings menu clicked");
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

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
