import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ClipboardItem {
  id: number;
  content: string;
  source_app: string | null;
  created_at: string;
  last_used_at: string | null;
}

export function useClipboard() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);

  const fetchItems = useCallback(async (searchQuery?: string) => {
    setLoading(true);
    try {
      const q = searchQuery ?? query;
      if (q.trim()) {
        const results = await invoke<ClipboardItem[]>("search_history", {
          query: q,
          limit: 50,
        });
        setItems(results);
      } else {
        const results = await invoke<ClipboardItem[]>("get_recent", {
          limit: 50,
          offset: 0,
        });
        setItems(results);
      }
      setSelectedIndex(0);
    } catch (err) {
      console.error("Failed to fetch items:", err);
    } finally {
      setLoading(false);
    }
  }, [query]);

  const search = useCallback((q: string) => {
    setQuery(q);
  }, []);

  const deleteItem = useCallback(async (id: number) => {
    try {
      await invoke("delete_item", { id });
      setItems((prev) => prev.filter((item) => item.id !== id));
    } catch (err) {
      console.error("Failed to delete item:", err);
    }
  }, []);

  const clearAll = useCallback(async () => {
    try {
      await invoke("clear_all");
      setItems([]);
    } catch (err) {
      console.error("Failed to clear all:", err);
    }
  }, []);

  const pasteItem = useCallback(async (id: number) => {
    try {
      await invoke("paste_item", { id });
    } catch (err) {
      console.error("Failed to paste:", err);
    }
  }, []);

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      fetchItems(query);
    }, 150);
    return () => clearTimeout(timer);
  }, [query, fetchItems]);

  // Listen for clipboard changes from Rust backend
  useEffect(() => {
    const unlisten = listen<string>("clipboard-changed", () => {
      if (!query.trim()) {
        fetchItems();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [query, fetchItems]);

  return {
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
  };
}
