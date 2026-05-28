import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SettingsData {
  max_items: string;
  max_days: string;
  hotkey: string;
  autostart: string;
}

export function Settings() {
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
    <div className="h-screen bg-gray-950 text-gray-200 flex flex-col">
      <div
        data-tauri-drag-region
        className="px-4 py-3 bg-gray-900 border-b border-gray-800"
      >
        <h2 className="text-sm font-semibold text-gray-300">设置</h2>
      </div>

      <div className="flex-1 p-4 space-y-5 overflow-y-auto">
        <div>
          <label className="block text-xs text-gray-500 mb-1.5">最大保留条数</label>
          <input
            type="number"
            value={settings.max_items}
            onChange={(e) => update("max_items", e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            min="100"
            max="10000"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-500 mb-1.5">最大保留天数</label>
          <input
            type="number"
            value={settings.max_days}
            onChange={(e) => update("max_days", e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            min="1"
            max="365"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-500 mb-1.5">快捷键</label>
          <input
            type="text"
            value={settings.hotkey}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            readOnly
          />
          <p className="text-[10px] text-gray-600 mt-1">
            修改快捷键请通过系统设置更改
          </p>
        </div>

        <div className="flex items-center justify-between">
          <label className="text-xs text-gray-500">开机自动启动</label>
          <button
            onClick={() =>
              update("autostart", settings.autostart === "true" ? "false" : "true")
            }
            className={`w-10 h-5 rounded-full transition-colors ${
              settings.autostart === "true" ? "bg-rose-500" : "bg-gray-700"
            }`}
          >
            <div
              className={`w-4 h-4 bg-white rounded-full transition-transform mx-0.5 ${
                settings.autostart === "true" ? "translate-x-4" : ""
              }`}
            />
          </button>
        </div>
      </div>

      <div className="px-4 py-3 bg-gray-900 border-t border-gray-800">
        <button
          onClick={save}
          className="w-full py-1.5 bg-rose-500 hover:bg-rose-600 text-white text-sm rounded transition-colors"
        >
          {saved ? "已保存" : "保存设置"}
        </button>
      </div>
    </div>
  );
}
