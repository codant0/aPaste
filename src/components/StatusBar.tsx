interface Props {
  totalCount: number;
  matchCount: number;
  query: string;
}

export function StatusBar({ totalCount, matchCount, query }: Props) {
  return (
    <div className="px-3 py-1.5 bg-[var(--bg-surface)] backdrop-blur-md border-t border-[var(--border)] flex justify-between items-center text-[10px] text-[var(--text-muted)]">
      <div className="flex items-center gap-2.5">
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] px-1 py-0.5 rounded text-[9px] font-mono">↑</kbd>
          <kbd className="bg-[var(--kbd-bg)] px-1 py-0.5 rounded text-[9px] font-mono">↓</kbd>
          <span className="ml-0.5">导航</span>
        </span>
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] px-1 py-0.5 rounded text-[9px] font-mono">Enter</kbd>
          <span className="ml-0.5">粘贴</span>
        </span>
        <span className="flex items-center gap-1">
          <kbd className="bg-[var(--kbd-bg)] px-1 py-0.5 rounded text-[9px] font-mono">Del</kbd>
          <span className="ml-0.5">删除</span>
        </span>
      </div>
      <span className="tabular-nums">
        {query ? `${matchCount} 条匹配` : `共 ${totalCount} 条`}
      </span>
    </div>
  );
}
