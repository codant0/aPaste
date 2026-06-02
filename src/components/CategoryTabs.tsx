import type { Category } from "../hooks/useClipboard";

interface Props {
  active: Category;
  onChange: (cat: Category) => void;
  favoriteCount?: number;
}

export function CategoryTabs({ active, onChange, favoriteCount }: Props) {
  return (
    <div className="px-3 pb-2">
      <div className="flex gap-1 bg-[var(--bg-input)] backdrop-blur-md rounded-lg p-0.5 border border-[var(--border)]">
        <button
          onClick={() => onChange("all")}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-md transition-all duration-150 cursor-pointer ${
            active === "all"
              ? "bg-[var(--bg-selected)] text-[var(--text-primary)] shadow-sm"
              : "text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          }`}
        >
          <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          所有
        </button>
        <button
          onClick={() => onChange("favorites")}
          className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-md transition-all duration-150 cursor-pointer ${
            active === "favorites"
              ? "bg-[var(--bg-selected)] text-[var(--text-primary)] shadow-sm"
              : "text-[var(--text-muted)] hover:text-[var(--text-secondary)]"
          }`}
        >
          <svg
            className={`w-3.5 h-3.5 ${active === "favorites" ? "text-yellow-400" : ""}`}
            fill={active === "favorites" ? "currentColor" : "none"}
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={active === "favorites" ? 0 : 2}
              d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
            />
          </svg>
          收藏项
          {favoriteCount !== undefined && favoriteCount > 0 && (
            <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-mono ${
              active === "favorites"
                ? "bg-[var(--accent)] text-white"
                : "bg-[var(--bg-hover)] text-[var(--text-muted)]"
            }`}>
              {favoriteCount}
            </span>
          )}
        </button>
      </div>
    </div>
  );
}
