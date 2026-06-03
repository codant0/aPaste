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

    // Hide window on blur (click outside), but not during drag.
    // Delay 150ms to let system dialogs take focus without hiding the window.
    let blurTimer: ReturnType<typeof setTimeout> | null = null;
    const unlistenBlur = appWindow.onFocusChanged(({ payload: focused }) => {
      if (blurTimer) {
        clearTimeout(blurTimer);
        blurTimer = null;
      }
      if (!focused && !dragging?.current) {
        blurTimer = setTimeout(async () => {
          const isFocused = await appWindow.isFocused();
          if (!isFocused) {
            appWindow.hide();
          }
        }, 150);
      }
    });

    return () => {
      unlistenShow.then((fn) => fn());
      unlistenBlur.then((fn) => fn());
    };
  }, [dragging]);
}
