interface Props {
  totalCount: number;
  matchCount: number;
  query: string;
}

export function StatusBar({ totalCount, matchCount, query }: Props) {
  return (
    <div className="px-3 py-1.5 bg-gray-950/60 backdrop-blur-md border-t border-gray-800/50 flex justify-between items-center text-[11px] text-gray-600">
      <div className="flex gap-3">
        <span>↑↓ 导航</span>
        <span>Enter 粘贴</span>
        <span>Delete 删除</span>
        <span>Esc 关闭</span>
      </div>
      <span>
        {query ? `${matchCount} 条匹配` : `共 ${totalCount} 条`}
      </span>
    </div>
  );
}
