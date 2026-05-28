use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

/// Register global hotkeys: tries Win+V, falls back to Win+Shift+V.
///
/// On Windows, Win+V is often reserved by the system clipboard history.
/// If registration fails, we fall back to Win+Shift+V automatically.
pub fn register(app_handle: AppHandle) {
    let primary = Shortcut::new(Some(Modifiers::SUPER), Code::KeyV);
    let fallback = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);

    match app_handle.global_shortcut().register(primary) {
        Ok(_) => log::info!("Global hotkey registered: Win+V"),
        Err(e) => {
            log::warn!(
                "Win+V unavailable ({}), falling back to Win+Shift+V",
                e
            );
            app_handle
                .global_shortcut()
                .register(fallback)
                .expect("Failed to register fallback hotkey Win+Shift+V");
            log::info!("Global hotkey registered: Win+Shift+V");
        }
    }
}

/// Show the main window positioned at the bottom-right corner of the primary monitor.
///
/// Called from the global shortcut plugin handler whenever our hotkey is pressed.
/// Emits a "popup-shown" event so the frontend can re-focus the search input.
pub fn show_popup(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        // Position at bottom-right of the primary monitor
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let w_width = 680.0;
            let w_height = 480.0;
            let x = (size.width as f64 / scale) - w_width - 20.0;
            let y = (size.height as f64 / scale) - w_height - 60.0;
            let _ = window.set_position(tauri::LogicalPosition::new(x, y));
        }

        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("popup-shown", ());
    }
}
