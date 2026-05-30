import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "dark" | "light";

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>("light");

  useEffect(() => {
    invoke<Record<string, string>>("get_settings").then((data) => {
      const t = (data.theme as Theme) || "dark";
      setThemeState(t);
      applyTheme(t);
    });
  }, []);

  const setTheme = useCallback((t: Theme) => {
    setThemeState(t);
    applyTheme(t);
    invoke("update_settings", { settings: { theme: t } });
  }, []);

  return { theme, setTheme };
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  root.classList.remove("dark", "light");
  root.classList.add(theme);
}
