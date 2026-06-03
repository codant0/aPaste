import { useRef, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

interface Props {
  query: string;
  onChange: (query: string) => void;
}

export function SearchBar({ query, onChange }: Props) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  useEffect(() => {
    const unlisten = listen("popup-shown", () => {
      requestAnimationFrame(() => inputRef.current?.focus());
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return (
    <div className="px-3 pt-3 pb-2">
      <div className="flex items-center gap-2 bg-[var(--bg-input)] backdrop-blur-md border border-[var(--border)] rounded-lg px-3 py-2 focus-within:border-[var(--border-focus)] focus-within:bg-[var(--bg-input-focus)] transition-all duration-200">
        <svg className="w-4 h-4 text-[var(--text-tertiary)] shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
        <input
          ref={inputRef}
          type="text"
          value={query}
          onChange={(e) => onChange(e.target.value)}
          placeholder="搜索剪贴板历史..."
          className="flex-1 bg-transparent border-none outline-none text-sm text-[var(--text-primary)] placeholder-[var(--text-muted)]"
        />
        {query && (
          <button
            onClick={() => onChange("")}
            className="text-[var(--text-tertiary)] hover:text-[var(--text-primary)] transition-colors p-0.5"
          >
            <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        )}
        <kbd className="text-[10px] text-[var(--text-muted)] bg-[var(--kbd-bg)] px-1.5 py-0.5 rounded font-mono">Esc</kbd>
      </div>
    </div>
  );
}
