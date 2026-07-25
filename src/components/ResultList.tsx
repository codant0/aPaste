import { useRef, useEffect } from "react";
import type { ClipboardItem } from "../hooks/useClipboard";
import { ResultItem } from "./ResultItem";

interface Props {
  items: ClipboardItem[];
  query: string;
  selectedIndex: number;
  loading: boolean;
  onSelect: (id: number) => void;
  onDelete: (id: number) => void;
  onToggleFavorite: (id: number) => void;
  onRenameFavorite: (id: number, name: string | null) => void;
}

export function ResultList({
  items,
  query,
  selectedIndex,
  loading,
  onSelect,
  onDelete,
  onToggleFavorite,
  onRenameFavorite,
}: Props) {
  const listRef = useRef<HTMLDivElement>(null);

  // Scroll selected item into view when selectedIndex changes
  useEffect(() => {
    if (!listRef.current) return;
    const selectedEl = listRef.current.querySelector('[data-selected="true"]');
    if (selectedEl) {
      selectedEl.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  }, [selectedIndex]);
  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex items-center gap-2 text-[var(--text-tertiary)] text-sm">
          <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          搜索中...
        </div>
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-[var(--text-muted)] gap-3">
        <div className="w-16 h-16 rounded-full bg-gradient-to-br from-[var(--bg-hover)] to-transparent flex items-center justify-center">
          <svg className="w-7 h-7 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
          </svg>
        </div>
        <span className="text-sm font-medium">
          {query ? "无匹配结果" : "暂无剪贴板记录"}
        </span>
        {!query && (
          <span className="text-[11px] text-[var(--text-muted2)]">
            复制内容后会自动出现在这里
          </span>
        )}
        {query && (
          <span className="text-[11px] text-[var(--text-muted2)]">
            尝试其他关键词
          </span>
        )}
      </div>
    );
  }

  return (
    <div ref={listRef} className="flex-1 overflow-y-auto space-y-[3px] px-1">
      {items.map((item, index) => (
        <ResultItem
          key={item.id}
          item={item}
          isSelected={index === selectedIndex}
          query={query}
          onSelect={() => onSelect(item.id)}
          onDelete={onDelete}
          onToggleFavorite={onToggleFavorite}
          onRenameFavorite={onRenameFavorite}
        />
      ))}
    </div>
  );
}
