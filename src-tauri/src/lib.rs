mod commands;
mod clipboard;
mod history;
mod hotkey;
mod db;

use tauri::{Emitter, Manager};
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::window::{EffectsBuilder, Effect};
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, GetWindowRect, GWLP_WNDPROC,
};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM, RECT};

const WM_NCHITTEST: u32 = 0x0084;
const WM_NCLBUTTONDBLCLK: u32 = 0x00A3;
const HTCAPTION: isize = 2;
const TITLE_BAR_PX: i32 = 36;

static mut ORIG_PROC: isize = 0;

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
            // Apply Windows 11 Mica effect & install drag support
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_effects(
                    EffectsBuilder::new()
                        .effect(Effect::Mica)
                        .build(),
                );

                // Install Win32 drag handler (WM_NCHITTEST → HTCAPTION for title bar)
                if let Ok(hwnd) = window.hwnd() {
                    unsafe {
                        let prev = SetWindowLongPtrW(
                            windows::Win32::Foundation::HWND(hwnd.0),
                            GWLP_WNDPROC,
                            drag_wnd_proc as *const () as isize,
                        );
                        ORIG_PROC = prev;
                    }
                }
            }

            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("apaste.db");
            let conn = db::connection::open(&db_path)
                .expect("failed to open database");
            db::migrate::run(&conn).expect("failed to run migrations");

            // Check autostart setting and register
            let autostart: String = conn
                .query_row("SELECT value FROM settings WHERE key = 'autostart'", [], |r| r.get(0))
                .unwrap_or_else(|_| "true".into());

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

            // Register global hotkey
            hotkey::register(app.handle().clone());

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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

unsafe extern "system" fn drag_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_NCHITTEST {
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.top + TITLE_BAR_PX {
            return LRESULT(HTCAPTION);
        }
    }

    // Swallow double-click on title bar to prevent maximize
    if msg == WM_NCLBUTTONDBLCLK {
        let x = (lparam.0 & 0xFFFF) as i16 as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.top + TITLE_BAR_PX {
            return LRESULT(0);
        }
    }

    if ORIG_PROC != 0 {
        let orig: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT =
            std::mem::transmute(ORIG_PROC);
        orig(hwnd, msg, wparam, lparam)
    } else {
        windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
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
