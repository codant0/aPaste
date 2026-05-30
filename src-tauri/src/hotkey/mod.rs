use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// Parse a hotkey string like "Win+V" or "Win+Shift+V" into a Shortcut.
fn parse_hotkey(s: &str) -> Option<Shortcut> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    let mut mods = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "win" | "super" => mods |= Modifiers::SUPER,
            "shift" => mods |= Modifiers::SHIFT,
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "v" => code = Some(Code::KeyV),
            "c" => code = Some(Code::KeyC),
            "x" => code = Some(Code::KeyX),
            "z" => code = Some(Code::KeyZ),
            "a" => code = Some(Code::KeyA),
            "space" => code = Some(Code::Space),
            _ => return None,
        }
    }

    code.map(|c| Shortcut::new(Some(mods), c))
}

/// Format a Shortcut back to a display string like "Win+Shift+V".
fn format_hotkey(shortcut: &Shortcut) -> String {
    let mut parts = Vec::new();
    if shortcut.mods.contains(Modifiers::SUPER) {
        parts.push("Win");
    }
    if shortcut.mods.contains(Modifiers::SHIFT) {
        parts.push("Shift");
    }
    if shortcut.mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if shortcut.mods.contains(Modifiers::ALT) {
        parts.push("Alt");
    }

    let key = match shortcut.key {
        Code::KeyV => "V",
        Code::KeyC => "C",
        Code::KeyX => "X",
        Code::KeyZ => "Z",
        Code::KeyA => "A",
        Code::Space => "Space",
        _ => "?",
    };
    parts.push(key);
    parts.join("+")
}

/// Register global hotkey from the saved setting, with fallback logic.
pub fn register(app_handle: &AppHandle, conn: &rusqlite::Connection) {
    let saved: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'hotkey'", [], |r| r.get(0))
        .unwrap_or_else(|_| "Win+Shift+V".into());

    let primary = Shortcut::new(Some(Modifiers::SUPER), Code::KeyV);
    let fallback = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);

    // If saved hotkey is the primary (Win+V), try it first
    if saved == "Win+V" {
        match app_handle.global_shortcut().register(primary) {
            Ok(_) => {
                log::info!("Global hotkey registered: Win+V");
                sync_hotkey_setting(conn, "Win+V");
                return;
            }
            Err(e) => {
                log::warn!("Win+V unavailable ({}), trying Win+Shift+V", e);
                if let Err(e2) = app_handle.global_shortcut().register(fallback) {
                    log::error!("Win+Shift+V also unavailable: {}", e2);
                } else {
                    log::info!("Global hotkey registered: Win+Shift+V");
                    sync_hotkey_setting(conn, "Win+Shift+V");
                }
                return;
            }
        }
    }

    // Otherwise, try the saved hotkey, then fallbacks
    if let Some(shortcut) = parse_hotkey(&saved) {
        match app_handle.global_shortcut().register(shortcut) {
            Ok(_) => {
                log::info!("Global hotkey registered: {}", saved);
                sync_hotkey_setting(conn, &saved);
                return;
            }
            Err(e) => {
                log::warn!("{} unavailable ({}), trying fallbacks", saved, e);
            }
        }
    }

    // Fallback chain: Win+V → Win+Shift+V
    match app_handle.global_shortcut().register(primary) {
        Ok(_) => {
            log::info!("Global hotkey registered: Win+V");
            sync_hotkey_setting(conn, "Win+V");
        }
        Err(_) => {
            if let Err(e) = app_handle.global_shortcut().register(fallback) {
                log::error!("All hotkey registrations failed: {}", e);
            } else {
                log::info!("Global hotkey registered: Win+Shift+V");
                sync_hotkey_setting(conn, "Win+Shift+V");
            }
        }
    }
}

/// Update the hotkey setting in DB to match the actually registered hotkey.
fn sync_hotkey_setting(conn: &rusqlite::Connection, hotkey: &str) {
    let _ = conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('hotkey', ?1)",
        rusqlite::params![hotkey],
    );
}

/// Unregister current hotkey and register a new one.
pub fn re_register(app_handle: &AppHandle, new_hotkey: &str) -> Result<String, String> {
    let global = app_handle.global_shortcut();

    // Unregister all known shortcuts
    for s in &[
        Shortcut::new(Some(Modifiers::SUPER), Code::KeyV),
        Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV),
    ] {
        let _ = global.unregister(*s);
    }

    // Try to register the requested hotkey
    let shortcut = parse_hotkey(new_hotkey)
        .ok_or_else(|| format!("Invalid hotkey: {}", new_hotkey))?;

    global
        .register(shortcut)
        .map_err(|e| format!("Failed to register {}: {}", new_hotkey, e))?;

    let actual = format_hotkey(&shortcut);
    log::info!("Global hotkey re-registered: {}", actual);
    Ok(actual)
}

/// Show/hide the main window (toggle behavior).
pub fn toggle_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            show_window(app, &window);
        }
    }
}

fn show_window(app: &AppHandle, window: &tauri::WebviewWindow) {
    // Position at bottom-right of the primary monitor
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let w_width = 340.0;
        let w_height = 520.0;
        let x = (size.width as f64 / scale) - w_width - 20.0;
        let y = (size.height as f64 / scale) - w_height - 60.0;
        let _ = window.set_position(tauri::LogicalPosition::new(x, y));
    }

    let _ = window.show();
    let _ = window.set_focus();
    let _ = app.emit("popup-shown", ());
}

/// Legacy function for compatibility.
pub fn show_popup(app: &AppHandle) {
    toggle_popup(app);
}
