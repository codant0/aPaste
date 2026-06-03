import type { ClipboardItem } from "../hooks/useClipboard";

interface Props {
  item: ClipboardItem;
  isSelected: boolean;
  query: string;
  onSelect: () => void;
  onDelete: (id: number) => void;
  onToggleFavorite: (id: number) => void;
}

function highlightMatch(text: string, query: string): string {
  if (!query.trim()) return escapeHtml(text);

  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return escapeHtml(text).replace(
    new RegExp(`(${escapedQuery})`, "gi"),
    "<mark style='background:var(--mark-bg);color:var(--mark-text)' class='rounded-sm px-0.5'>$1</mark>"
  );
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr + "Z");
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMin = Math.floor(diffMs / 60000);
  const diffHr = Math.floor(diffMs / 3600000);

  if (diffMin < 1) return "刚刚";
  if (diffMin < 60) return `${diffMin} 分钟前`;
  if (diffHr < 24) return `${diffHr} 小时前`;
  if (diffHr < 48) return "昨天";

  return date.toLocaleDateString("zh-CN", {
    month: "short",
    day: "numeric",
  });
}

export function ResultItem({ item, isSelected, query, onSelect, onDelete, onToggleFavorite }: Props) {
  const preview = item.content.length > 120
    ? item.content.slice(0, 120) + "..."
    : item.content;

  return (
    <div
      onClick={onSelect}
      className={`group px-3 py-2.5 cursor-pointer border-l-2 transition-all duration-150 animate-fade-in ${
        isSelected
          ? "bg-[var(--bg-selected)] border-l-[var(--accent)]"
          : "border-l-transparent hover:bg-[var(--bg-hover)]"
      }`}
    >
      <div className="flex justify-between items-start gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-[10px] text-[var(--text-tertiary)] font-medium">
              {formatTime(item.created_at)}
            </span>
            {item.source_app && (
              <>
                <span className="text-[var(--text-muted2)] text-[8px]">&bull;</span>
                <span className="text-[10px] text-[var(--text-muted)] truncate max-w-[100px]">
                  {item.source_app}
                </span>
              </>
            )}
          </div>
          <div
            className="text-[13px] text-[var(--text-primary)] leading-relaxed break-all line-clamp-2"
            dangerouslySetInnerHTML={{
              __html: highlightMatch(preview, query),
            }}
          />
        </div>
        <div className="flex items-center gap-0.5 shrink-0 mt-0.5 opacity-0 group-hover:opacity-100 translate-x-1 group-hover:translate-x-0 transition-all duration-150">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggleFavorite(item.id);
            }}
            className={`text-xs cursor-pointer p-0.5 rounded transition-colors ${
              item.is_favorite
                ? "text-yellow-400 hover:text-yellow-300"
                : "text-[var(--text-muted)] hover:text-yellow-400"
            }`}
            title={item.is_favorite ? "取消收藏" : "收藏"}
          >
            <svg className="w-3.5 h-3.5" fill={item.is_favorite ? "currentColor" : "none"} stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={item.is_favorite ? 0 : 2}
                d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"
              />
            </svg>
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onDelete(item.id);
            }}
            className="text-[var(--text-muted)] hover:text-[var(--danger)] text-xs cursor-pointer p-0.5 rounded transition-colors"
            title="删除"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </div>
    </div>
  );
}
