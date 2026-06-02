import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ClipboardItem {
  id: number;
  content: string;
  source_app: string | null;
  created_at: string;
  last_used_at: string | null;
  is_favorite: boolean;
}

export type Category = "all" | "favorites";

export function useClipboard() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [query, setQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState<Category>("all");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const [totalCount, setTotalCount] = useState(0);

  const fetchTotalCount = useCallback(async () => {
    try {
      const count = await invoke<number>("get_count");
      setTotalCount(count);
    } catch (err) {
      console.error("Failed to fetch total count:", err);
    }
  }, []);

  const fetchItems = useCallback(async (searchQuery?: string, category?: Category) => {
    setLoading(true);
    const cat = category ?? activeCategory;
    try {
      const q = (searchQuery ?? query).trim();
      if (cat === "favorites") {
        if (q) {
          const results = await invoke<ClipboardItem[]>("search_favorites", {
            query: q,
            limit: 50,
          });
          setItems(results);
        } else {
          const results = await invoke<ClipboardItem[]>("get_favorites", {
            limit: 50,
            offset: 0,
          });
          setItems(results);
        }
      } else {
        if (q) {
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
      }
      setSelectedIndex(0);
    } catch (err) {
      console.error("Failed to fetch items:", err);
    } finally {
      setLoading(false);
    }
  }, [query, activeCategory]);

  const search = useCallback((q: string) => {
    setQuery(q);
  }, []);

  const switchCategory = useCallback((cat: Category) => {
    setActiveCategory(cat);
  }, []);

  const deleteItem = useCallback(async (id: number) => {
    try {
      await invoke("delete_item", { id });
      setItems((prev) => prev.filter((item) => item.id !== id));
      fetchTotalCount();
    } catch (err) {
      console.error("Failed to delete item:", err);
    }
  }, [fetchTotalCount]);

  const clearAll = useCallback(async () => {
    try {
      await invoke("clear_all");
      // Refetch — some items (favorites) may remain
      fetchItems();
      fetchTotalCount();
    } catch (err) {
      console.error("Failed to clear all:", err);
    }
  }, [fetchItems, fetchTotalCount]);

  const toggleFavorite = useCallback(async (id: number) => {
    try {
      const newState = await invoke<boolean>("toggle_favorite", { id });
      setItems((prev) =>
        prev.map((item) =>
          item.id === id ? { ...item, is_favorite: newState } : item
        )
      );
      // If in favorites view and item was unfavorited, remove it from the list
      if (!newState && activeCategory === "favorites") {
        setItems((prev) => {
          const next = prev.filter((item) => item.id !== id);
          setSelectedIndex((si) => Math.min(si, next.length - 1));
          return next;
        });
      }
    } catch (err) {
      console.error("Failed to toggle favorite:", err);
    }
  }, [activeCategory]);

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
      fetchItems(query, activeCategory);
    }, 150);
    return () => clearTimeout(timer);
  }, [query, activeCategory, fetchItems]);

  // Fetch total count on mount
  useEffect(() => {
    fetchTotalCount();
  }, [fetchTotalCount]);

  // Listen for clipboard changes from Rust backend
  useEffect(() => {
    const unlisten = listen<string>("clipboard-changed", () => {
      fetchTotalCount();
      // Only auto-refresh in "all" view when no active search
      if (activeCategory === "all" && !query.trim()) {
        fetchItems();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [query, activeCategory, fetchItems, fetchTotalCount]);

  return {
    items,
    query,
    loading,
    totalCount,
    selectedIndex,
    activeCategory,
    setSelectedIndex,
    setActiveCategory: switchCategory,
    search,
    deleteItem,
    clearAll,
    toggleFavorite,
    pasteItem,
    fetchItems,
  };
}
