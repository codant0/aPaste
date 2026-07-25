import type { Category } from "../hooks/useClipboard";

interface Props {
  totalCount: number;
  matchCount: number;
  query: string;
  category: Category;
}

export function StatusBar({ totalCount, matchCount, query, category }: Props) {
  return (
    <div className="px-3 py-1.5 bg-[var(--bg-surface)] backdrop-blur-md border-t border-[var(--border)] flex justify-between items-center text-[10px] text-[var(--text-muted)]">
      <div className="flex items-center gap-2.5">
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] border border-[var(--kbd-border)] px-1 py-0.5 rounded text-[10px] font-mono">↑</kbd>
          <kbd className="bg-[var(--kbd-bg)] border border-[var(--kbd-border)] px-1 py-0.5 rounded text-[10px] font-mono">↓</kbd>
          <span className="ml-0.5">导航</span>
        </span>
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] border border-[var(--kbd-border)] px-1 py-0.5 rounded text-[10px] font-mono">Enter</kbd>
          <span className="ml-0.5">粘贴</span>
        </span>
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] border border-[var(--kbd-border)] px-1 py-0.5 rounded text-[10px] font-mono">Del</kbd>
          <span className="ml-0.5">删除</span>
        </span>
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] border border-[var(--kbd-border)] px-1 py-0.5 rounded text-[10px] font-mono">Tab</kbd>
          <span className="ml-0.5">切换</span>
        </span>
      </div>
      <span className="tabular-nums">
        {category === "favorites"
          ? (query ? `${matchCount} 条匹配` : `${matchCount} 条收藏`)
          : (query ? `${matchCount} 条匹配` : `共 ${totalCount} 条`)
        }
      </span>
    </div>
  );
}
