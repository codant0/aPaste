# aPaste

> Windows 剪贴板历史管理器 — 弹窗、搜索、粘贴、收藏。

基于 **Tauri v2** + **React 19** + **TypeScript** + **Tailwind CSS v4**（前端）和 **Rust** + **SQLite**（后端）构建。

## 功能特性

- **剪贴板历史** — 自动捕获 Windows 剪贴板中的文本内容
- **即时搜索** — 基于 SQLite FTS5 的全文搜索，支持前缀匹配
- **快速粘贴** — 按 Enter 将选中项粘贴回任意应用程序
- **收藏功能** — 点击星标收藏关键条目，永久保留，不受清理和「清除所有」影响
- **全局快捷键** — 按 Win+Shift+V 随时随地呼出弹窗
- **自动清理** — 可配置最大条数和保留天数；收藏项永不过期
- **Mica 模糊效果** — Windows 11 亚克力风格半透明背景
- **深色 / 浅色主题** — 设置中切换，重启后保持
- **便携安装** — 单个 NSIS 安装包，无需外部数据库

## 环境要求

- **Windows 10/11**（唯一支持的平台）
- **Node.js** ≥ 18（含 npm）
- **Rust**（通过 [rustup.rs](https://rustup.rs) 安装）

首次安装 Rust 时需要 Windows 构建工具链：
```bash
rustup default stable-msvc
```

## 快速开始

```bash
# 安装前端依赖
npm install

# 启动开发服务器 + Rust 后端（HMR 端口 1420）
npx tauri dev
```

按 **Win+Shift+V** 打开弹窗。

## 项目结构

```
aPaste/
├── src/                          # React 前端
│   ├── App.tsx                   # 主应用：键盘导航、视图路由
│   ├── App.css                   # 全局样式和滚动条
│   ├── hooks/
│   │   ├── useClipboard.ts       # 数据获取、搜索、收藏、增删改
│   │   ├── useHotkey.ts          # 窗口显示/隐藏、失焦处理
│   │   └── useTheme.ts           # 深色/浅色主题持久化
│   └── components/
│       ├── SearchBar.tsx         # 带防抖和清除按钮的搜索框
│       ├── CategoryTabs.tsx      #「所有」/「收藏项」分类标签
│       ├── ResultList.tsx        # 可滚动条目列表
│       ├── ResultItem.tsx        # 单行：时间、来源、预览、★ 星标
│       ├── StatusBar.tsx         # 快捷键提示 + 匹配计数
│       └── Settings.tsx          # 主题、条数、快捷键、开机启动
├── src-tauri/                    # Rust 后端
│   ├── Cargo.toml                # Rust 依赖
│   ├── tauri.conf.json           # Tauri 窗口和打包配置
│   └── src/
│       ├── main.rs               # Windows 入口
│       ├── lib.rs                # 应用启动、托盘、热键、清理线程
│       ├── commands.rs           # 12 个 Tauri IPC 命令
│       ├── db/
│       │   ├── connection.rs     # SQLite 连接（WAL 模式，NORMAL 同步）
│       │   └── migrate.rs        # 数据库迁移 + 初始数据
│       ├── clipboard/
│       │   ├── monitor.rs        # Win32 WM_CLIPBOARDUPDATE 监听器
│       │   └── writer.rs         # Win32 剪贴板写入 + Ctrl+V 模拟
│       ├── history/
│       │   ├── manager.rs        # 增删改查 + 收藏切换
│       │   ├── search.rs         # FTS5 前缀搜索
│       │   └── cleanup.rs        # 按时间和条数定期清理
│       └── hotkey/
│           └── mod.rs            # 全局热键注册（Win+V 回退方案）
└── docs/
    └── plans/                    # 设计和实施计划文档
```

## 常用命令

| 命令 | 说明 |
|---|---|
| `npm install` | 安装前端依赖 |
| `npx tauri dev` | 启动开发服务器 + Rust 后端（热更新） |
| `npx tauri build` | 生产构建 → NSIS 安装包 |
| `npx tsc --noEmit` | TypeScript 类型检查（不输出文件） |
| `cargo test` | 运行 Rust 单元测试（内存 SQLite，无需启动服务） |

> `cargo` 命令需在 `src-tauri/` 目录下运行，或传入 `--manifest-path src-tauri/Cargo.toml`。

## 架构

```
Win32 剪贴板事件 (WM_CLIPBOARDUPDATE)
  → monitor.rs: 读取文本，SHA-256 去重哈希
  → 通过 manager::add_item 存入 SQLite
  → 发送 "clipboard-changed" 事件 → 前端刷新数据

用户在搜索栏输入
  → useClipboard hook（150ms 防抖）→ invoke("search_history") 或按分类
  → search.rs: FTS5 MATCH 前缀通配 + 特殊字符转义

用户按 Enter 选择条目
  → invoke("paste_item") → writer.rs: Win32 剪贴板写入 + keybd_event Ctrl+V
```

### 数据库

单文件 SQLite 数据库，位于 `%APPDATA%/apaste/apaste.db`。共三张表：

| 表 | 用途 |
|---|---|
| `clipboard_items` | id, content, content_hash, source_app, is_favorite, created_at, last_used_at |
| `clipboard_fts` | FTS5 虚拟表（内容同步自 clipboard_items） |
| `settings` | 键值对存储偏好设置 |

通过三个触发器（`clipboard_items` 表的插入/删除/更新后）保持 FTS5 索引同步。

### IPC 命令（共 12 个）

| 命令 | 签名 |
|---|---|
| `search_history` | `(query, limit?) → ClipboardItem[]` |
| `get_recent` | `(limit?, offset?) → ClipboardItem[]` |
| `delete_item` | `(id) → void` |
| `clear_all` | `() → void`（跳过收藏项） |
| `paste_item` | `(id) → void` |
| `get_settings` | `() → HashMap<String, String>` |
| `update_settings` | `(settings) → void` |
| `get_count` | `() → i64` |
| `update_hotkey` | `(hotkey_str, app) → String` |
| `toggle_favorite` | `(id) → bool` |
| `get_favorites` | `(limit?, offset?) → ClipboardItem[]` |
| `search_favorites` | `(query, limit?) → ClipboardItem[]` |

## 构建产物

执行 `npx tauri build` 后：

```
src-tauri/target/release/apaste.exe
src-tauri/target/release/bundle/nsis/aPaste_0.1.0_x64-setup.exe
```

## 许可证

MIT
