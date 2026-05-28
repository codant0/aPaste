import type { ClipboardItem } from "../hooks/useClipboard";

interface Props {
  item: ClipboardItem;
  isSelected: boolean;
  query: string;
  onSelect: () => void;
  onDelete: (id: number) => void;
}

function highlightMatch(text: string, query: string): string {
  if (!query.trim()) return escapeHtml(text);

  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return escapeHtml(text).replace(
    new RegExp(`(${escapedQuery})`, "gi"),
    "<mark class='bg-rose-500/40 text-rose-200 rounded px-0.5'>$1</mark>"
  );
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
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

export function ResultItem({ item, isSelected, query, onSelect, onDelete }: Props) {
  const preview = item.content.length > 120
    ? item.content.slice(0, 120) + "..."
    : item.content;

  return (
    <div
      onClick={onSelect}
      className={`px-3 py-2 cursor-pointer border-l-3 transition-colors ${
        isSelected
          ? "bg-gray-800 border-l-rose-500"
          : "border-l-transparent hover:bg-gray-800/50"
      }`}
    >
      <div className="flex justify-between items-start gap-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-0.5">
            <span className="text-[10px] text-gray-500">
              {formatTime(item.created_at)}
            </span>
            {item.source_app && (
              <span className="text-[10px] text-gray-600 truncate">
                {item.source_app}
              </span>
            )}
          </div>
          <div
            className="text-sm text-gray-300 leading-relaxed break-all line-clamp-2"
            dangerouslySetInnerHTML={{
              __html: highlightMatch(preview, query),
            }}
          />
        </div>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onDelete(item.id);
          }}
          className="text-gray-600 hover:text-red-400 text-xs shrink-0 mt-1 opacity-0 group-hover:opacity-100 transition-opacity"
          title="删除"
        >
          ✕
        </button>
      </div>
    </div>
  );
}
