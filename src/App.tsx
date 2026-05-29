import { useCallback, useEffect } from "react";
import { useClipboard } from "./hooks/useClipboard";
import { useHotkey } from "./hooks/useHotkey";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { StatusBar } from "./components/StatusBar";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
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

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
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
    [items, selectedIndex, setSelectedIndex, pasteItem, deleteItem]
  );

  return (
    <div className="h-screen flex flex-col select-none backdrop-blur-xl bg-gray-950/80">
      {/* Title bar — drag handled by Win32 WM_NCHITTEST */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-3 py-2 cursor-grab active:cursor-grabbing bg-gray-950/60 border-b border-gray-800/50"
      >
        <span className="text-xs font-semibold text-gray-400">
          aPaste
        </span>
        <button
          onClick={clearAll}
          onMouseDown={(e) => e.stopPropagation()}
          className="text-[10px] text-gray-600 hover:text-red-400 transition-colors cursor-pointer"
          title="清空全部"
        >
          清空全部
        </button>
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
