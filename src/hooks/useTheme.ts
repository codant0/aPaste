import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export type Theme = "dark" | "light";

const DEFAULT_ACCENT = "#f43f5e";

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>("light");
  const [accentColor, setAccentColorState] = useState(DEFAULT_ACCENT);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings").then((data) => {
      const t = (data.theme as Theme) || "light";
      setThemeState(t);
      applyTheme(t);

      const accent = data.accent_color || DEFAULT_ACCENT;
      setAccentColorState(accent);
      applyAccentColor(accent);
    });
  }, []);

  const setTheme = useCallback((t: Theme) => {
    setThemeState(t);
    applyTheme(t);
    invoke("update_settings", { settings: { theme: t } });
  }, []);

  const setAccentColor = useCallback((color: string) => {
    setAccentColorState(color);
    applyAccentColor(color);
    invoke("update_settings", { settings: { accent_color: color } });
  }, []);

  return { theme, setTheme, accentColor, setAccentColor };
}

function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

function applyAccentColor(color: string) {
  document.documentElement.style.setProperty("--accent", color);
}
