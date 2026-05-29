use crate::history::manager;
use crate::AppState;
use sha2::{Sha256, Digest};

use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, OpenClipboard,
    AddClipboardFormatListener, RemoveClipboardFormatListener,
};
use windows::Win32::Foundation::{HWND, HGLOBAL};
use windows::Win32::System::Memory::{GlobalLock, GlobalUnlock};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, CW_USEDEFAULT,
    DefWindowProcW, DispatchMessageW, GetForegroundWindow, GetMessageW,
    GetWindowTextW, RegisterClassExW, TranslateMessage, WNDCLASSEXW,
    HWND_MESSAGE, MSG, WINDOW_STYLE,
    WM_CLIPBOARDUPDATE, WM_DESTROY, WS_EX_LEFT,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let hinstance = GetModuleHandleW(None).expect("GetModuleHandleW failed");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(message_window_proc),
                hInstance: hinstance.into(),
                lpszClassName: windows::core::w!("aPasteMonitor"),
                ..Default::default()
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_LEFT,
                windows::core::w!("aPasteMonitor"),
                windows::core::w!(""),
                WINDOW_STYLE(0),
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND_MESSAGE,
                None,
                hinstance,
                Some(&app as *const _ as *const std::ffi::c_void),
            );

            let hwnd = match hwnd {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Failed to create clipboard monitor window: {:?}", e);
                    return;
                }
            };

            AddClipboardFormatListener(hwnd).expect("AddClipboardFormatListener failed");

            let app_ref = app.clone();

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
                if msg.message == WM_CLIPBOARDUPDATE {
                    handle_clipboard_change(&app_ref);
                }
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

unsafe extern "system" fn message_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    if msg == WM_DESTROY {
        let _ = RemoveClipboardFormatListener(hwnd);
        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn handle_clipboard_change(app: &AppHandle) {
    unsafe {
        if let Err(e) = OpenClipboard(None) {
            log::error!("OpenClipboard failed: {:?}", e);
            return;
        }

        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32);
        if let Ok(handle) = handle {
            if !handle.0.is_null() {
                let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
                if !ptr.is_null() {
                    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                    let slice = std::slice::from_raw_parts(ptr, len);
                    if let Ok(text) = String::from_utf16(slice) {
                        if !text.trim().is_empty() {
                            let source_app = get_foreground_window_title();
                            let mut hasher = Sha256::new();
                            hasher.update(text.as_bytes());
                            let hash = hex::encode(&hasher.finalize()[..8]);

                            let state = app.state::<AppState>();
                            let conn = state.db.lock().unwrap();
                            manager::add_item(&conn, &text, &hash, source_app.as_deref())
                                .unwrap_or_else(|e| log::error!("Failed to save clipboard: {}", e));

                            // Notify frontend
                            let _ = app.emit("clipboard-changed", text);
                        }
                    }
                }
                let _ = GlobalUnlock(HGLOBAL(handle.0));
            }
        }

        let _ = CloseClipboard();
    }
}

fn get_foreground_window_title() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}
