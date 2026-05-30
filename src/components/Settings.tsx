import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Theme } from "../hooks/useTheme";

interface SettingsData {
  max_items: string;
  max_days: string;
  hotkey: string;
  autostart: string;
}

interface Props {
  onBack: () => void;
  theme: Theme;
  setTheme: (t: Theme) => void;
}

export function Settings({ onBack, theme, setTheme }: Props) {
  const [settings, setSettings] = useState<SettingsData>({
    max_items: "1000",
    max_days: "30",
    hotkey: "Win+Shift+V",
    autostart: "true",
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings").then((data) => {
      setSettings({
        max_items: data.max_items || "1000",
        max_days: data.max_days || "30",
        hotkey: data.hotkey || "Win+Shift+V",
        autostart: data.autostart || "true",
      });
    });
  }, []);

  const update = (key: keyof SettingsData, value: string) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  };

  const save = async () => {
    const map: Record<string, string> = {};
    for (const [k, v] of Object.entries(settings)) {
      map[k] = v;
    }
    await invoke("update_settings", { settings: map });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="h-screen flex flex-col select-none backdrop-blur-xl bg-[var(--bg-app)]">
      {/* Title bar */}
      <div className="flex items-center gap-2 px-3 py-2 cursor-grab active:cursor-grabbing bg-[var(--bg-surface)] border-b border-[var(--border)]">
        <button
          onClick={onBack}
          onMouseDown={(e) => e.stopPropagation()}
          className="text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors cursor-pointer p-0.5 rounded hover:bg-[var(--bg-hover)]"
          title="返回"
        >
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
          </svg>
        </button>
        <span className="text-xs font-semibold text-[var(--text-secondary)]">设置</span>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Appearance section */}
        <div className="px-4 pt-4 pb-3">
          <h3 className="text-[11px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider mb-3">外观</h3>
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={() => setTheme("dark")}
              className={`relative flex flex-col items-center gap-1.5 p-3 rounded-lg border-2 transition-all cursor-pointer ${
                theme === "dark"
                  ? "border-[var(--accent)] bg-[var(--bg-selected)]"
                  : "border-[var(--border)] bg-[var(--bg-input)] hover:border-[var(--text-muted)]"
              }`}
            >
              <svg className="w-5 h-5 text-[var(--text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
              <span className="text-xs text-[var(--text-primary)] font-medium">深色</span>
              {theme === "dark" && (
                <div className="absolute top-1.5 right-1.5 w-4 h-4 bg-[var(--accent)] rounded-full flex items-center justify-center">
                  <svg className="w-2.5 h-2.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
              )}
            </button>
            <button
              onClick={() => setTheme("light")}
              className={`relative flex flex-col items-center gap-1.5 p-3 rounded-lg border-2 transition-all cursor-pointer ${
                theme === "light"
                  ? "border-[var(--accent)] bg-[var(--bg-selected)]"
                  : "border-[var(--border)] bg-[var(--bg-input)] hover:border-[var(--text-muted)]"
              }`}
            >
              <svg className="w-5 h-5 text-[var(--text-secondary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
              <span className="text-xs text-[var(--text-primary)] font-medium">浅色</span>
              {theme === "light" && (
                <div className="absolute top-1.5 right-1.5 w-4 h-4 bg-[var(--accent)] rounded-full flex items-center justify-center">
                  <svg className="w-2.5 h-2.5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                  </svg>
                </div>
              )}
            </button>
          </div>
        </div>

        <div className="border-t border-[var(--border-subtle)] mx-4" />

        {/* History section */}
        <div className="px-4 pt-4 pb-3">
          <h3 className="text-[11px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider mb-3">历史记录</h3>
          <div className="space-y-3">
            <div>
              <label className="block text-xs text-[var(--text-secondary)] mb-1.5">最大保留条数</label>
              <input
                type="number"
                value={settings.max_items}
                onChange={(e) => update("max_items", e.target.value)}
                className="w-full bg-[var(--bg-input)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:border-[var(--border-focus)] focus:bg-[var(--bg-input-focus)] outline-none transition-all"
                min="100"
                max="10000"
              />
              <p className="text-[10px] text-[var(--text-muted)] mt-1">超出时自动删除最早的记录</p>
            </div>

            <div>
              <label className="block text-xs text-[var(--text-secondary)] mb-1.5">最大保留天数</label>
              <input
                type="number"
                value={settings.max_days}
                onChange={(e) => update("max_days", e.target.value)}
                className="w-full bg-[var(--bg-input)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:border-[var(--border-focus)] focus:bg-[var(--bg-input-focus)] outline-none transition-all"
                min="1"
                max="365"
              />
              <p className="text-[10px] text-[var(--text-muted)] mt-1">超过此天数的记录将被清理</p>
            </div>
          </div>
        </div>

        <div className="border-t border-[var(--border-subtle)] mx-4" />

        {/* Shortcuts section */}
        <div className="px-4 py-3">
          <h3 className="text-[11px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider mb-3">快捷键</h3>
          <div>
            <label className="block text-xs text-[var(--text-secondary)] mb-1.5">全局快捷键</label>
            <div className="flex items-center gap-2">
              <input
                type="text"
                value={settings.hotkey}
                className="flex-1 bg-[var(--bg-input)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] outline-none"
                readOnly
              />
              <span className="text-[10px] text-[var(--text-muted)]">只读</span>
            </div>
            <p className="text-[10px] text-[var(--text-muted)] mt-1.5">
              修改快捷键请通过系统设置更改
            </p>
          </div>
        </div>

        <div className="border-t border-[var(--border-subtle)] mx-4" />

        {/* System section */}
        <div className="px-4 py-3">
          <h3 className="text-[11px] font-medium text-[var(--text-tertiary)] uppercase tracking-wider mb-3">系统</h3>
          <div className="flex items-center justify-between py-1">
            <div>
              <label className="text-xs text-[var(--text-secondary)]">开机自动启动</label>
              <p className="text-[10px] text-[var(--text-muted)] mt-0.5">登录 Windows 时自动启动</p>
            </div>
            <button
              onClick={() =>
                update("autostart", settings.autostart === "true" ? "false" : "true")
              }
              className={`relative w-10 h-5 rounded-full transition-colors duration-200 ${
                settings.autostart === "true" ? "bg-[var(--accent)]" : "bg-[var(--toggle-off)]"
              }`}
            >
              <div
                className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow-sm transition-transform duration-200 ${
                  settings.autostart === "true" ? "translate-x-5" : "translate-x-0.5"
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Save button */}
      <div className="px-4 py-3 bg-[var(--bg-surface)] border-t border-[var(--border)]">
        <button
          onClick={save}
          className={`w-full py-2 text-sm rounded-lg transition-all duration-200 font-medium ${
            saved
              ? "bg-[var(--success-bg)] text-[var(--success-text)] border border-[var(--success-border)]"
              : "bg-[var(--accent)] hover:bg-[var(--accent-hover)] text-white"
          }`}
        >
          {saved ? "已保存" : "保存设置"}
        </button>
      </div>
    </div>
  );
}
