import { useRef, useEffect } from "react";

interface Props {
  query: string;
  onChange: (query: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
}

export function SearchBar({ query, onChange, onKeyDown }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="px-3 pt-3 pb-2">
      <div className="flex items-center gap-2 bg-gray-900 border border-gray-700 rounded-lg px-3 py-2 focus-within:border-rose-500 transition-colors">
        <svg className="w-4 h-4 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
          placeholder="搜索剪贴板历史..."
          className="flex-1 bg-transparent border-none outline-none text-sm text-gray-200 placeholder-gray-500"
        />
        {query && (
          <button
            onClick={() => onChange("")}
            className="text-gray-500 hover:text-gray-300 text-xs px-1"
          >
            ✕
          </button>
        )}
        <kbd className="text-[10px] text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded">Esc</kbd>
      </div>
    </div>
  );
}
