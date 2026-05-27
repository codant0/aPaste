# Windows Clipboard Manager — 设计方案

**日期:** 2026-05-27
**技术栈:** Tauri (Rust + React)
**目标:** 替代 Windows 默认剪切板，提供历史搜索和持久化存储

## 一、核心需求

- 剪切板历史记录搜索（模糊匹配）
- 内容在电脑重启后不会清空（SQLite 持久化）
- 纯文本支持（架构上预留图片扩展能力）
- 数量 + 时间组合的自动清理策略，参数可配置

## 二、交互模式

- **热键弹出窗口：** 按下快捷键 → 屏幕右下角弹出搜索窗口 → 搜索/选择 → Enter 粘贴
- **系统托盘常驻：** 右键菜单 → 最近记录预览 / 设置 / 退出
- **智能热键：** 启动时检测 Win+V 是否可用
  - 系统剪切板历史已禁用 → 注册 Win+V
  - 系统剪切板历史已启用 → 降级注册 Win+Shift+V

### 弹出窗口交互细节

- 搜索框自动聚焦，输入即搜索
- 结果列表时间倒序，当天显示相对时间（"刚刚"/"5 分钟前"）
- 选中项 Enter → 写入剪切板 + 模拟 Ctrl+V 粘贴到上一活跃窗口
- 窗口失去焦点自动隐藏，Esc 或点击外部关闭
- 再次按快捷键时恢复上次搜索状态
- 窗口支持标题栏拖动，可调整位置

## 三、架构设计

```
UI 层 (React + Tailwind CSS)
  ↕ IPC (invoke / event)
Tauri 命令层 (#[tauri::command])
  ↕
核心逻辑层 (Rust 模块: clipboard / history / hotkey / db)
  ↕
存储层 (SQLite + FTS5)
```

### 前端组件树

```
src/
├── main.tsx
├── App.tsx              # 窗口管理(弹出/设置)
├── components/
│   ├── SearchBar.tsx    # 搜索框(自动聚焦, 实时搜索)
│   ├── ResultList.tsx   # 结果列表(虚拟滚动, 键盘导航)
│   ├── ResultItem.tsx   # 单条记录(高亮匹配)
│   ├── StatusBar.tsx    # 底部状态栏(快捷操作提示)
│   └── Settings.tsx     # 设置页面
├── hooks/
│   ├── useClipboard.ts  # 剪切板数据 hook
│   └── useHotkey.ts     # 快捷键监听 hook
└── styles/
    └── index.css        # Tailwind + 自定义样式
```

### Rust 模块结构

```
src-tauri/src/
├── main.rs              # 入口, Tauri 启动
├── lib.rs               # 模块注册
├── clipboard/
│   ├── mod.rs
│   ├── monitor.rs       # Win32 剪切板监听 (AddClipboardFormatListener)
│   └── writer.rs        # 剪切板写入 + Ctrl+V 模拟
├── history/
│   ├── mod.rs
│   ├── manager.rs       # CRUD 操作
│   ├── search.rs        # FTS5 模糊搜索
│   └── cleanup.rs       # 过期清理任务
├── hotkey/
│   └── mod.rs           # 热键注册/智能降级
├── db/
│   ├── mod.rs
│   ├── migrate.rs       # Schema 版本迁移
│   └── connection.rs    # 连接管理
└── commands.rs          # Tauri #[command] 导出
```

## 四、数据库设计 (SQLite)

```sql
-- 主表
CREATE TABLE clipboard_items (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  content      TEXT NOT NULL,
  content_hash TEXT NOT NULL,      -- SHA-256 前16位, 用于去重
  source_app   TEXT,               -- 来源应用窗口标题
  image        BLOB,               -- 图片数据 (v2 启用)
  created_at   TEXT NOT NULL DEFAULT (datetime('now')),
  last_used_at TEXT                -- 上次粘贴时间
);

-- FTS5 全文搜索索引
CREATE VIRTUAL TABLE clipboard_fts
USING fts5(content, content='clipboard_items', content_rowid='id');

-- 自动同步触发器
CREATE TRIGGER clipboard_ai AFTER INSERT ON clipboard_items BEGIN
  INSERT INTO clipboard_fts(rowid, content) VALUES (new.id, new.content);
END;

CREATE TRIGGER clipboard_ad AFTER DELETE ON clipboard_items BEGIN
  INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
  VALUES ('delete', old.id, old.content);
END;

-- 配置表
CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
```

### 默认配置

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| max_items | 1000 | 最大保留条数 |
| max_days | 30 | 最大保留天数 |
| hotkey | Win+Shift+V | 快捷键 |
| autostart | true | 开机自启 |

## 五、数据流

1. Windows 剪切板变化 → Win32 `AddClipboardFormatListener` 事件
2. Rust 后台线程捕获 → 去重（content_hash 比对）→ 过滤空内容
3. 写入 SQLite → FTS5 索引自动更新
4. 用户按快捷键 → Tauri 事件通知前端 → 显示窗口
5. 用户搜索 → Rust FTS5 查询（`SELECT ... WHERE clipboard_fts MATCH ?`）→ 模糊匹配结果
6. 用户选择 → 写入剪切板 → 模拟 Ctrl+V → 恢复剪切板原内容

## 六、错误处理

| 场景 | 处理策略 |
|------|----------|
| 剪切板监听失败 | 系统托盘图标变灰 + 通知 + 30s 自动重试 |
| 数据库写入失败 | 内存缓存队列 + 定期重试 |
| 数据库损坏 | 自动备份旧文件 + 重新初始化 |
| 磁盘空间不足 | 紧急清理: 仅保留最近 100 条 |
| 热键注册失败 | 提示冲突, 引导用户更换 |
| 粘贴失败 | 仅写剪切板不模拟按键; 3s 超时放弃 |
| 窗口位置异常 | 限定当前活跃屏幕右下角 |
| 配置损坏 | 使用默认值覆盖写入 |

## 七、测试策略

| 层级 | 范围 | 工具 |
|------|------|------|
| Rust 单元测试 | 搜索、去重、清理策略、DB 迁移 | cargo test |
| Rust 集成测试 | Tauri commands 端到端调用链 | cargo test --test integration |
| 前端组件测试 | SearchBar、ResultList 交互 | Vitest + React Testing Library |
| E2E | 完整剪切板流程 | Windows 实机手动验证 |

## 八、非功能需求

- **性能：** 全链路（搜索 → 显示）< 50ms
- **内存：** 运行时 < 50MB
- **存储：** 1000 条文本记录 < 2MB
- **启动：** 开机自启，托盘图标 < 1 秒内出现
- **兼容：** Windows 10 / 11
