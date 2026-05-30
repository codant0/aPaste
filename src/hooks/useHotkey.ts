import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useHotkey(dragging?: { current: boolean }) {
  useEffect(() => {
    const appWindow = getCurrentWindow();

    const unlistenShow = listen("popup-shown", () => {
      appWindow.show();
      appWindow.setFocus();
    });

    // Hide window on blur (click outside), but not during drag
    const unlistenBlur = appWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused && !dragging?.current) {
        appWindow.hide();
      }
    });

    return () => {
      unlistenShow.then((fn) => fn());
      unlistenBlur.then((fn) => fn());
    };
  }, [dragging]);
}
