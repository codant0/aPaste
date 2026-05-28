import type { ClipboardItem } from "../hooks/useClipboard";
import { ResultItem } from "./ResultItem";

interface Props {
  items: ClipboardItem[];
  query: string;
  selectedIndex: number;
  loading: boolean;
  onSelect: (id: number) => void;
  onDelete: (id: number) => void;
}

export function ResultList({
  items,
  query,
  selectedIndex,
  loading,
  onSelect,
  onDelete,
}: Props) {
  if (loading) {
    return (
      <div className="flex-1 flex items-center justify-center text-gray-500 text-sm">
        搜索中...
      </div>
    );
  }

  if (items.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center text-gray-600 gap-1">
        <svg className="w-10 h-10 mb-2 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
        </svg>
        <span className="text-sm">
          {query ? "无匹配结果" : "暂无剪贴板记录"}
        </span>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto">
      {items.map((item, index) => (
        <ResultItem
          key={item.id}
          item={item}
          isSelected={index === selectedIndex}
          query={query}
          onSelect={() => onSelect(item.id)}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
