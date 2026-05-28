import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useHotkey() {
  useEffect(() => {
    const appWindow = getCurrentWindow();

    const unlistenShow = listen("popup-shown", () => {
      appWindow.show();
      appWindow.setFocus();
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        appWindow.hide();
      }
    };
    window.addEventListener("keydown", handleKeyDown);

    // Hide window on blur (click outside)
    const unlistenBlur = appWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        appWindow.hide();
      }
    });

    return () => {
      unlistenShow.then((fn) => fn());
      unlistenBlur.then((fn) => fn());
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
