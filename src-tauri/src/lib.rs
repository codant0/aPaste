mod commands;
mod clipboard;
mod history;
mod hotkey;
mod db;

use tauri::{Emitter, Manager};
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::window::{EffectsBuilder, Effect};

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
                .with_handler(|_app, _shortcut, event| {
                    use tauri_plugin_global_shortcut::ShortcutState;
                    if event.state == ShortcutState::Pressed {
                        hotkey::show_popup(_app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // Apply Windows 11 Mica effect
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_effects(
                    EffectsBuilder::new()
                        .effect(Effect::Mica)
                        .build(),
                );
            }

            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("apaste.db");
            let conn = db::connection::open(&db_path)
                .expect("failed to open database");
            db::migrate::run(&conn).expect("failed to run migrations");

            // Read settings before conn is moved into Mutex
            let autostart: String = conn
                .query_row("SELECT value FROM settings WHERE key = 'autostart'", [], |r| r.get(0))
                .unwrap_or_else(|_| "true".into());

            // Register global hotkey (reads from DB, syncs setting on success)
            hotkey::register(app.handle(), &conn);

            if autostart == "true" {
                set_autostart(true);
            }

            let state = AppState {
                db: std::sync::Arc::new(std::sync::Mutex::new(conn)),
            };
            let state_cleanup = state.clone();
            app.manage(state);

            // Spawn periodic cleanup task (runs every hour)
            std::thread::spawn(move || {
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(3600));

                    let conn = state_cleanup.db.lock().unwrap();
                    let max_items: i64 = conn
                        .query_row("SELECT value FROM settings WHERE key = 'max_items'", [], |r| {
                            r.get::<_, String>(0)
                        })
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1000);

                    let max_days: i64 = conn
                        .query_row("SELECT value FROM settings WHERE key = 'max_days'", [], |r| {
                            r.get::<_, String>(0)
                        })
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(30);

                    if let Err(e) = crate::history::cleanup::run_cleanup(&conn, max_items, max_days) {
                        log::error!("Cleanup failed: {}", e);
                    }
                }
            });

            // Start clipboard monitor
            clipboard::monitor::start(app.handle().clone());

            // Build tray menu
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
            let _separator = tauri::menu::PredefinedMenuItem::separator(app)?;
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
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-settings", ());
                            }
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
            commands::get_count,
            commands::update_hotkey,
            commands::toggle_favorite,
            commands::get_favorites,
            commands::search_favorites,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn set_autostart(enable: bool) {
    unsafe {
        let key_path = windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");

        if enable {
            if let Ok(exe_path) = std::env::current_exe() {
                let path = exe_path.to_string_lossy();
                let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();

                let mut hkey = windows::Win32::System::Registry::HKEY::default();
                if windows::Win32::System::Registry::RegCreateKeyW(
                    windows::Win32::System::Registry::HKEY_CURRENT_USER,
                    key_path,
                    &mut hkey,
                ).is_ok() {
                    let _ = windows::Win32::System::Registry::RegSetValueExW(
                        hkey,
                        windows::core::w!("aPaste"),
                        0,
                        windows::Win32::System::Registry::REG_SZ,
                        Some(std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2)),
                    );
                }
            }
        } else {
            let mut hkey = windows::Win32::System::Registry::HKEY::default();
            if windows::Win32::System::Registry::RegOpenKeyExW(
                windows::Win32::System::Registry::HKEY_CURRENT_USER,
                key_path,
                0,
                windows::Win32::System::Registry::KEY_SET_VALUE,
                &mut hkey,
            ).is_ok() {
                let _ = windows::Win32::System::Registry::RegDeleteValueW(
                    hkey,
                    windows::core::w!("aPaste"),
                );
            }
        }
    }
}
