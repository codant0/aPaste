use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
};
use windows::Win32::Foundation::{HANDLE, GlobalFree};

pub fn write_text_and_paste(text: &str) -> Result<(), String> {
    write_to_clipboard(text)?;
    simulate_ctrl_v();
    Ok(())
}

fn write_to_clipboard(text: &str) -> Result<(), String> {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        if !OpenClipboard(None).is_ok() {
            return Err("OpenClipboard failed".into());
        }

        let _ = EmptyClipboard();

        let size = (wide.len() * 2) as usize;
        let hglobal = GlobalAlloc(GMEM_MOVEABLE, size)
            .map_err(|e| format!("GlobalAlloc failed: {:?}", e))?;

        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            let _ = GlobalFree(hglobal);
            let _ = CloseClipboard();
            return Err("GlobalLock failed".into());
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());

        let _ = GlobalUnlock(hglobal);

        let result = SetClipboardData(
            CF_UNICODETEXT.0 as u32,
            HANDLE(hglobal.0),
        );

        let _ = CloseClipboard();

        if result.is_err() {
            let _ = GlobalFree(hglobal);
            return Err(format!("SetClipboardData failed: {:?}", result));
        }
    }

    Ok(())
}

fn simulate_ctrl_v() {
    std::thread::sleep(std::time::Duration::from_millis(30));

    unsafe {
        let mut inputs: [INPUT; 4] = std::mem::zeroed();

        // Ctrl down
        inputs[0].r#type = INPUT_KEYBOARD;
        inputs[0].Anonymous.ki.wVk = VK_CONTROL;

        // V down
        inputs[1].r#type = INPUT_KEYBOARD;
        inputs[1].Anonymous.ki.wVk = VK_V;

        // V up
        inputs[2].r#type = INPUT_KEYBOARD;
        inputs[2].Anonymous.ki.wVk = VK_V;
        inputs[2].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        // Ctrl up
        inputs[3].r#type = INPUT_KEYBOARD;
        inputs[3].Anonymous.ki.wVk = VK_CONTROL;
        inputs[3].Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;

        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}
