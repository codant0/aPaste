import { useCallback, useEffect, useState } from "react";
import { useClipboard } from "./hooks/useClipboard";
import { useHotkey } from "./hooks/useHotkey";
import { useTheme } from "./hooks/useTheme";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { StatusBar } from "./components/StatusBar";
import { Settings } from "./components/Settings";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

function App() {
  const [view, setView] = useState<"main" | "settings">("main");
  const { theme, setTheme } = useTheme();

  const {
    items,
    query,
    loading,
    selectedIndex,
    setSelectedIndex,
    search,
    deleteItem,
    clearAll,
    pasteItem,
    fetchItems,
  } = useClipboard();

  useHotkey();

  // Initial load
  useEffect(() => {
    fetchItems();
  }, []);

  // Listen for show-settings event from tray menu
  useEffect(() => {
    const unlisten = listen("show-settings", () => {
      setView("settings");
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (view === "settings") return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < items.length - 1 ? prev + 1 : prev
        );
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (items[selectedIndex]) {
          getCurrentWindow().hide().then(() => {
            pasteItem(items[selectedIndex].id);
          });
        }
      } else if (e.key === "Delete" && items[selectedIndex]) {
        e.preventDefault();
        deleteItem(items[selectedIndex].id);
      } else if (e.key === "Escape") {
        e.preventDefault();
        getCurrentWindow().hide();
      }
    },
    [view, items, selectedIndex, setSelectedIndex, pasteItem, deleteItem]
  );

  if (view === "settings") {
    return <Settings onBack={() => setView("main")} theme={theme} setTheme={setTheme} />;
  }

  return (
    <div className="h-screen flex flex-col select-none backdrop-blur-xl bg-[var(--bg-app)]">
      {/* Title bar — drag handled by Win32 WM_NCHITTEST subclass */}
      <div
        className="flex items-center justify-between px-3 py-2 cursor-grab active:cursor-grabbing bg-[var(--bg-surface)] border-b border-[var(--border)]"
      >
        <span className="text-xs font-semibold text-[var(--text-secondary)]">
          aPaste
        </span>
        <div className="flex items-center gap-0.5">
          <button
            onClick={() => setView("settings")}
            onMouseDown={(e) => e.stopPropagation()}
            className="text-[var(--text-muted)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] transition-all cursor-pointer p-1 rounded"
            title="设置"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
          <button
            onClick={clearAll}
            onMouseDown={(e) => e.stopPropagation()}
            className="text-[10px] text-[var(--text-muted)] hover:text-red-400 hover:bg-[var(--bg-hover)] transition-all cursor-pointer px-1.5 py-0.5 rounded"
            title="清空全部"
          >
            清空全部
          </button>
        </div>
      </div>

      <SearchBar
        query={query}
        onChange={search}
        onKeyDown={handleKeyDown}
      />

      <ResultList
        items={items}
        query={query}
        selectedIndex={selectedIndex}
        loading={loading}
        onSelect={(id) => {
          getCurrentWindow().hide().then(() => pasteItem(id));
        }}
        onDelete={deleteItem}
      />

      <StatusBar
        totalCount={items.length}
        matchCount={items.length}
        query={query}
      />
    </div>
  );
}

export default App;
