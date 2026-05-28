# aPaste 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 Windows 剪切板管理工具，支持历史搜索和持久化存储

**Architecture:** Tauri v2 (Rust backend + React frontend) 桌面应用。Rust 端通过 Win32 API 监听剪切板变化，SQLite + FTS5 做持久化和全文搜索，React 端提供弹出搜索窗口和设置界面。系统托盘常驻后台。

**Tech Stack:** Tauri v2, Rust, React 18, TypeScript, Tailwind CSS, SQLite (rusqlite + FTS5), windows-rs crate

**Spec:** `docs/superpowers/specs/2026-05-27-clipboard-tool-design.md`

---

## 文件结构总览

### 新建文件

```
src/                              # React 前端
├── main.tsx                      # React 入口
├── App.tsx                       # 窗口管理 & 全局状态
├── App.css                       # 全局样式
├── components/
│   ├── SearchBar.tsx             # 搜索框
│   ├── ResultList.tsx            # 结果列表
│   ├── ResultItem.tsx            # 单条记录
│   ├── StatusBar.tsx             # 底部状态栏
│   └── Settings.tsx              # 设置页面
├── hooks/
│   ├── useClipboard.ts           # 剪切板数据获取 & 搜索
│   └── useHotkey.ts              # 快捷键事件监听
└── styles/
    └── index.css                 # Tailwind 指令

src-tauri/                        # Rust 后端
├── Cargo.toml
├── tauri.conf.json
├── capabilities/
│   └── default.json
├── build.rs
├── icons/                        # 应用图标
│   └── icon.png
└── src/
    ├── main.rs                   # Tauri 入口
    ├── lib.rs                    # 模块注册 & 插件初始化
    ├── commands.rs               # Tauri 命令导出
    ├── clipboard/
    │   ├── mod.rs
    │   ├── monitor.rs            # Win32 剪切板监听
    │   └── writer.rs             # 剪切板写入 + 粘贴模拟
    ├── history/
    │   ├── mod.rs
    │   ├── manager.rs            # CRUD 操作
    │   ├── search.rs             # FTS5 全文搜索
    │   └── cleanup.rs            # 过期清理
    ├── hotkey/
    │   └── mod.rs                # 热键注册 & 智能降级
    └── db/
        ├── mod.rs
        ├── migrate.rs            # Schema 迁移
        └── connection.rs         # 连接管理

根目录
├── index.html                    # Vite HTML 入口
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.js
└── postcss.config.js
```

---

### Task 1: 项目脚手架

**Files:**
- Create: `package.json`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `tailwind.config.js`, `postcss.config.js`
- Create: `src/main.tsx`, `src/App.tsx`, `src/App.css`, `src/styles/index.css`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/capabilities/default.json`
- Create: `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`

- [ ] **Step 1: 使用 Tauri CLI 创建项目**

```bash
cd F:/projects/Clipboard
npm create tauri-app@latest . -- --template react-ts --manager npm
```

Expected: 生成完整项目骨架

- [ ] **Step 2: 安装前端依赖**

```bash
cd F:/projects/Clipboard
npm install
npm install -D tailwindcss @tailwindcss/vite
```

Expected: 依赖安装成功

- [ ] **Step 3: 配置 Vite + Tailwind**

Write `vite.config.ts`:
```typescript
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
}));
```

- [ ] **Step 4: 引入 Tailwind CSS**

Write `src/styles/index.css`:
```css
@import "tailwindcss";
```

Write `src/main.tsx`:
```typescript
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
```

- [ ] **Step 5: 创建基础 App 组件**

Write `src/App.tsx`:
```typescript
function App() {
  return (
    <div className="min-h-screen bg-gray-950 text-gray-200 flex items-center justify-center">
      <h1 className="text-2xl font-bold">aPaste</h1>
    </div>
  );
}

export default App;
```

- [ ] **Step 6: 配置 Tauri 窗口**

Write `src-tauri/tauri.conf.json`:
```json
{
  "$schema": "https://raw.githubusercontent.com/nicknisi/tauri-config-schema/main/schema.json",
  "productName": "aPaste",
  "version": "0.1.0",
  "identifier": "com.apaste.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "aPaste",
        "width": 680,
        "height": 480,
        "decorations": false,
        "resizable": true,
        "visible": false,
        "center": false,
        "x": 0,
        "y": 0,
        "skipTaskbar": true,
        "alwaysOnTop": true,
        "focus": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 7: 配置 Tauri capabilities**

Write `src-tauri/capabilities/default.json`:
```json
{
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-position",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "core:event:default",
    "global-shortcut:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered"
  ]
}
```

- [ ] **Step 8: 配置 Rust 依赖**

Write `src-tauri/Cargo.toml`:
```toml
[package]
name = "apaste"
version = "0.1.0"
edition = "2021"

[lib]
name = "apaste_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.31", features = ["bundled", "vtab"] }
sha2 = "0.10"
hex = "0.4"
windows = { version = "0.58", features = [
  "Win32_System_DataExchange",
  "Win32_UI_Input_KeyboardAndMouse",
  "Win32_UI_WindowsAndMessaging",
  "Win32_Foundation",
  "Win32_System_Threading",
  "Win32_System_LibraryLoader",
  "Win32_System_Registry",
  "Win32_System_Ole",
] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
log = "0.4"
env_logger = "0.11"
```

- [ ] **Step 9: 编写 Rust 入口文件**

Write `src-tauri/src/main.rs`:
```rust
// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    apaste_lib::run()
}
```

Write `src-tauri/src/lib.rs`:
```rust
mod commands;
mod clipboard;
mod history;
mod hotkey;
mod db;

use tauri::Manager;

#[derive(Clone)]
pub struct AppState {
    pub db: std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");

            let db_path = app_dir.join("apaste.db");
            let conn = db::connection::open(&db_path)
                .expect("failed to open database");
            db::migrate::run(&conn).expect("failed to run migrations");

            let state = AppState {
                db: std::sync::Arc::new(std::sync::Mutex::new(conn)),
            };
            app.manage(state);

            // Start clipboard monitor
            clipboard::monitor::start(app.handle().clone());

            // Register global hotkey
            hotkey::register(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_history,
            commands::get_recent,
            commands::delete_item,
            commands::clear_all,
            commands::paste_item,
            commands::get_settings,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 10: 验证项目能编译运行**

```bash
cd F:/projects/Clipboard
cargo tauri dev
```

Expected: 窗口弹出，显示 "Clipboard Manager"

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri v2 + React + Tailwind project"
```

---

### Task 2: 数据库层 — 连接管理 & Schema 迁移

**Files:**
- Create: `src-tauri/src/db/mod.rs`
- Create: `src-tauri/src/db/connection.rs`
- Create: `src-tauri/src/db/migrate.rs`

- [ ] **Step 1: 编写 db/mod.rs**

Write `src-tauri/src/db/mod.rs`:
```rust
pub mod connection;
pub mod migrate;
```

- [ ] **Step 2: 编写连接管理**

Write `src-tauri/src/db/connection.rs`:
```rust
use rusqlite::{Connection, Result};
use std::path::Path;

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;"
    )?;

    Ok(conn)
}
```

- [ ] **Step 3: 编写 Schema 迁移**

Write `src-tauri/src/db/migrate.rs`:
```rust
use rusqlite::{Connection, Result};

pub fn run(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS clipboard_items (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content      TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            source_app   TEXT,
            image        BLOB,
            created_at   TEXT NOT NULL DEFAULT (datetime('now')),
            last_used_at TEXT
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts
        USING fts5(content, content='clipboard_items', content_rowid='id');

        CREATE TRIGGER IF NOT EXISTS clipboard_ai
        AFTER INSERT ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(rowid, content)
            VALUES (new.id, new.content);
        END;

        CREATE TRIGGER IF NOT EXISTS clipboard_ad
        AFTER DELETE ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
        END;

        CREATE TRIGGER IF NOT EXISTS clipboard_au
        AFTER UPDATE ON clipboard_items BEGIN
            INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
            VALUES ('delete', old.id, old.content);
            INSERT INTO clipboard_fts(rowid, content)
            VALUES (new.id, new.content);
        END;

        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO settings (key, value) VALUES ('max_items', '1000');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('max_days', '30');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('hotkey', 'Win+Shift+V');
        INSERT OR IGNORE INTO settings (key, value) VALUES ('autostart', 'true');"
    )?;

    Ok(())
}
```

- [ ] **Step 4: 编写迁移测试**

Append to `src-tauri/src/db/migrate.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        run(&conn).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 4);

        // Verify FTS table exists
        conn.execute("INSERT INTO clipboard_items (content, content_hash) VALUES ('test', 'abc')", [])
            .unwrap();
        let fts_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM clipboard_fts WHERE clipboard_fts MATCH 'test'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fts_count, 1);
    }
}
```

- [ ] **Step 5: 运行测试**

```bash
cd F:/projects/Clipboard/src-tauri
cargo test db::migrate::tests::test_migration_creates_tables
```

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db/
git commit -m "feat: add database layer with connection and migration"
```

---

### Task 3: 剪切板监听器

**Files:**
- Create: `src-tauri/src/clipboard/mod.rs`
- Create: `src-tauri/src/clipboard/monitor.rs`

- [ ] **Step 1: 编写 clipboard/mod.rs**

Write `src-tauri/src/clipboard/mod.rs`:
```rust
pub mod monitor;
pub mod writer;
```

- [ ] **Step 2: 编写剪切板监听器**

Write `src-tauri/src/clipboard/monitor.rs`:
```rust
use crate::history::manager;
use crate::AppState;
use sha2::{Sha256, Digest};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, OpenClipboard,
};
use windows::Win32::System::Ole::GetForegroundWindow;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    AddClipboardFormatListener, GetWindowTextW,
    RemoveClipboardFormatListener, CF_UNICODETEXT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetMessageW,
    RegisterClassExW, TranslateMessage, DispatchMessageW,
    WNDCLASSEXW, CW_USEDEFAULT, HWND_MESSAGE, MSG,
    WM_CLIPBOARDUPDATE, WM_DESTROY, WS_EX_LEFT,
};

const CLIPBOARD_WINDOW_CLASS: &str = "aPasteMonitor";

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let hinstance = GetModuleHandleW(None).expect("GetModuleHandleW failed");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(message_window_proc),
                hInstance: hinstance.into(),
                lpszClassName: windows::core::w!("aPasteMonitor"),
                ..Default::default()
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_LEFT,
                windows::core::w!("aPasteMonitor"),
                windows::core::w!(""),
                0,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND_MESSAGE,
                None,
                hinstance,
                Some(&app as *const _ as *const std::ffi::c_void),
            );

            if hwnd.0 == 0 {
                log::error!("Failed to create clipboard monitor window");
                return;
            }

            AddClipboardFormatListener(hwnd).expect("AddClipboardFormatListener failed");

            let app_ref = app.clone();

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
                if msg.message == WM_CLIPBOARDUPDATE {
                    handle_clipboard_change(&app_ref);
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    });
}

unsafe extern "system" fn message_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    if msg == WM_DESTROY {
        let _ = RemoveClipboardFormatListener(hwnd);
        windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn handle_clipboard_change(app: &AppHandle) {
    unsafe {
        if let Err(e) = OpenClipboard(None) {
            log::error!("OpenClipboard failed: {:?}", e);
            return;
        }

        let handle = GetClipboardData(CF_UNICODETEXT.0 as u32);
        if handle.is_ok() {
            let handle = handle.unwrap();
            if !handle.0.is_null() {
                let ptr = windows::Win32::System::DataExchange::GlobalLock(handle.0) as *const u16;
                if !ptr.is_null() {
                    let len = (0..).take_while(|&i| *ptr.add(i) != 0).count();
                    let slice = std::slice::from_raw_parts(ptr, len);
                    if let Ok(text) = String::from_utf16(slice) {
                        if !text.trim().is_empty() {
                            let source_app = get_foreground_window_title();
                            let mut hasher = Sha256::new();
                            hasher.update(text.as_bytes());
                            let hash = hex::encode(&hasher.finalize()[..8]);

                            let state = app.state::<AppState>();
                            let conn = state.db.lock().unwrap();
                            manager::add_item(&conn, &text, &hash, source_app.as_deref())
                                .unwrap_or_else(|e| log::error!("Failed to save clipboard: {}", e));

                            // Notify frontend
                            let _ = app.emit("clipboard-changed", text);
                        }
                    }
                }
                windows::Win32::System::DataExchange::GlobalUnlock(handle.0);
            }
        }

        let _ = CloseClipboard();
    }
}

fn get_foreground_window_title() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len > 0 {
            Some(String::from_utf16_lossy(&buf[..len as usize]))
        } else {
            None
        }
    }
}
```

- [ ] **Step 3: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功或仅 warning

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/clipboard/monitor.rs src-tauri/src/clipboard/mod.rs
git commit -m "feat: add Win32 clipboard monitor with hash dedup"
```

---

### Task 4: 历史管理 — CRUD + 搜索 + 清理

**Files:**
- Create: `src-tauri/src/history/mod.rs`
- Create: `src-tauri/src/history/manager.rs`
- Create: `src-tauri/src/history/search.rs`
- Create: `src-tauri/src/history/cleanup.rs`

- [ ] **Step 1: 编写 history/mod.rs**

Write `src-tauri/src/history/mod.rs`:
```rust
pub mod manager;
pub mod search;
pub mod cleanup;
```

- [ ] **Step 2: 编写 history/manager.rs**

Write `src-tauri/src/history/manager.rs`:
```rust
use rusqlite::{Connection, Result, params};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub source_app: Option<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

pub fn add_item(
    conn: &Connection,
    content: &str,
    content_hash: &str,
    source_app: Option<&str>,
) -> Result<()> {
    // Check for duplicate — if last item has same hash, skip
    let last_hash: Option<String> = conn
        .query_row(
            "SELECT content_hash FROM clipboard_items ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .ok();

    if last_hash.as_deref() == Some(content_hash) {
        return Ok(());
    }

    conn.execute(
        "INSERT INTO clipboard_items (content, content_hash, source_app) VALUES (?1, ?2, ?3)",
        params![content, content_hash, source_app],
    )?;

    Ok(())
}

pub fn get_recent(conn: &Connection, limit: i64, offset: i64) -> Result<Vec<ClipboardItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, source_app, created_at, last_used_at
         FROM clipboard_items
         ORDER BY id DESC
         LIMIT ?1 OFFSET ?2"
    )?;

    let items = stmt.query_map(params![limit, offset], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            source_app: row.get(2)?,
            created_at: row.get(3)?,
            last_used_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

pub fn delete_item(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM clipboard_items", [])?;
    // FTS index is cleared via triggers
    conn.execute("DELETE FROM clipboard_fts", [])?;
    Ok(())
}

pub fn update_last_used(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clipboard_items SET last_used_at = datetime('now') WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate::run(&conn).unwrap();
        conn
    }

    #[test]
    fn test_add_and_get_recent() {
        let conn = setup_db();
        add_item(&conn, "hello world", "hash1", Some("Notepad")).unwrap();
        add_item(&conn, "foo bar", "hash2", None).unwrap();

        let items = get_recent(&conn, 10, 0).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].content, "foo bar"); // Most recent first
    }

    #[test]
    fn test_dedup_skips_consecutive_same_hash() {
        let conn = setup_db();
        add_item(&conn, "hello", "hash1", None).unwrap();
        add_item(&conn, "hello again", "hash1", None).unwrap(); // Same hash as last

        let items = get_recent(&conn, 10, 0).unwrap();
        assert_eq!(items.len(), 1); // Second insert skipped
    }

    #[test]
    fn test_delete_and_clear() {
        let conn = setup_db();
        add_item(&conn, "a", "h1", None).unwrap();
        add_item(&conn, "b", "h2", None).unwrap();

        delete_item(&conn, 2).unwrap();
        assert_eq!(get_recent(&conn, 10, 0).unwrap().len(), 1);

        clear_all(&conn).unwrap();
        assert_eq!(get_recent(&conn, 10, 0).unwrap().len(), 0);
    }
}
```

- [ ] **Step 3: 编写 history/search.rs**

Write `src-tauri/src/history/search.rs`:
```rust
use rusqlite::{Connection, Result, params};
use super::manager::ClipboardItem;

pub fn search(conn: &Connection, query: &str, limit: i64) -> Result<Vec<ClipboardItem>> {
    // Use FTS5 for full-text search
    // Escape special FTS5 characters and build prefix query for fuzzy matching
    let escaped = escape_fts5(query);
    let fts_query = if escaped.is_empty() {
        return get_all(conn, limit);
    } else {
        format!("{}*", escaped)
    };

    let mut stmt = conn.prepare(
        "SELECT ci.id, ci.content, ci.source_app, ci.created_at, ci.last_used_at
         FROM clipboard_items ci
         INNER JOIN clipboard_fts fts ON ci.id = fts.rowid
         WHERE clipboard_fts MATCH ?1
         ORDER BY rank
         LIMIT ?2"
    )?;

    let items = stmt.query_map(params![fts_query, limit], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            source_app: row.get(2)?,
            created_at: row.get(3)?,
            last_used_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(items)
}

fn get_all(conn: &Connection, limit: i64) -> Result<Vec<ClipboardItem>> {
    super::manager::get_recent(conn, limit, 0)
}

fn escape_fts5(query: &str) -> String {
    // Escape FTS5 special characters: * " ( ) - : ^
    let special = ['*', '"', '(', ')', '-', ':', '^'];
    let trimmed = query.trim();
    let mut result = String::with_capacity(trimmed.len());
    for ch in trimmed.chars() {
        if special.contains(&ch) {
            // Replace with space to break the special meaning
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrate;
    use crate::history::manager::add_item;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        migrate::run(&conn).unwrap();
        add_item(&conn, "import React from 'react'", "h1", None).unwrap();
        add_item(&conn, "npm install tauri", "h2", None).unwrap();
        add_item(&conn, "const x = 42", "h3", None).unwrap();
        add_item(&conn, "React hooks are useful", "h4", None).unwrap();
        conn
    }

    #[test]
    fn test_fuzzy_search() {
        let conn = setup();
        let results = search(&conn, "react", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_no_results() {
        let conn = setup();
        let results = search(&conn, "zzzznotfound", 10).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_empty_query_returns_all() {
        let conn = setup();
        let results = search(&conn, "", 10).unwrap();
        assert_eq!(results.len(), 4);
    }
}
```

- [ ] **Step 4: 编写 history/cleanup.rs**

Write `src-tauri/src/history/cleanup.rs`:
```rust
use rusqlite::{Connection, Result, params};

pub fn run_cleanup(conn: &Connection, max_items: i64, max_days: i64) -> Result<usize> {
    let mut deleted = 0;

    // Delete by age
    deleted += conn.execute(
        "DELETE FROM clipboard_items
         WHERE created_at < datetime('now', ?1)",
        params![format!("-{} days", max_days)],
    )?;

    // Delete by count — keep only max_items most recent
    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM clipboard_items", [], |r| r.get(0)
    )?;

    if total > max_items {
        let to_delete = total - max_items;
        conn.execute(
            "DELETE FROM clipboard_items WHERE id IN (
                SELECT id FROM clipboard_items ORDER BY id ASC LIMIT ?1
            )",
            params![to_delete],
        )?;
        deleted += to_delete as usize;
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_cleanup_by_count() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE clipboard_items (id INTEGER PRIMARY KEY, content TEXT, created_at TEXT DEFAULT (datetime('now')));
             INSERT INTO clipboard_items (content) VALUES ('a'), ('b'), ('c'), ('d'), ('e');"
        ).unwrap();

        let deleted = run_cleanup(&conn, 3, 365).unwrap();
        assert!(deleted >= 2);
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
    }
}
```

- [ ] **Step 5: 运行所有 history 测试**

```bash
cd F:/projects/Clipboard/src-tauri
cargo test history::
```

Expected: 所有测试 PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/history/
git commit -m "feat: add history CRUD, FTS5 search, and cleanup"
```

---

### Task 5: 剪切板写入 & 粘贴模拟

**Files:**
- Create: `src-tauri/src/clipboard/writer.rs`

- [ ] **Step 1: 编写剪切板写入 + 粘贴模拟**

Write `src-tauri/src/clipboard/writer.rs`:
```rust
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Ole::GetForegroundWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    keybd_event, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
};

pub fn write_text_and_paste(text: &str) -> Result<(), String> {
    write_to_clipboard(text)?;
    simulate_ctrl_v();
    Ok(())
}

fn write_to_clipboard(text: &str) -> Result<(), String> {
    unsafe {
        // Encode as UTF-16 null-terminated
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();

        if !OpenClipboard(None).is_ok() {
            return Err("OpenClipboard failed".into());
        }

        let _ = EmptyClipboard();

        // Allocate global memory
        let size = (wide.len() * 2) as usize;
        let hglobal = windows::Win32::System::DataExchange::GlobalAlloc(
            windows::Win32::System::DataExchange::GMEM_MOVEABLE,
            size,
        )
        .map_err(|e| format!("GlobalAlloc failed: {:?}", e))?;

        let ptr = windows::Win32::System::DataExchange::GlobalLock(hglobal.0);
        if ptr.is_null() {
            let _ = CloseClipboard();
            return Err("GlobalLock failed".into());
        }

        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());

        windows::Win32::System::DataExchange::GlobalUnlock(hglobal.0);

        let result = SetClipboardData(
            13, // CF_UNICODETEXT = 13
            Some(windows::Win32::Foundation::HANDLE(hglobal.0)),
        );

        let _ = CloseClipboard();

        if result.is_err() {
            return Err(format!("SetClipboardData failed: {:?}", result));
        }
    }

    Ok(())
}

fn simulate_ctrl_v() {
    // Brief delay for clipboard to settle
    std::thread::sleep(std::time::Duration::from_millis(30));

    unsafe {
        keybd_event(VK_CONTROL.0 as u8, 0, 0, 0);
        keybd_event(VK_V.0 as u8, 0, 0, 0);
        keybd_event(VK_V.0 as u8, 0, KEYEVENTF_KEYUP.0 as u32, 0);
        keybd_event(VK_CONTROL.0 as u8, 0, KEYEVENTF_KEYUP.0 as u32, 0);
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/clipboard/writer.rs
git commit -m "feat: add clipboard write and Ctrl+V paste simulation"
```

---

### Task 6: 全局热键管理

**Files:**
- Create: `src-tauri/src/hotkey/mod.rs`

- [ ] **Step 1: 编写热键注册（智能检测 Win+V）**

Write `src-tauri/src/hotkey/mod.rs`:
```rust
use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub fn register(app: AppHandle) {
    // Try Win+V first, fall back to Win+Shift+V
    let primary = Shortcut::try_from("Super+KeyV").unwrap();
    let fallback = Shortcut::try_from("Super+Shift+KeyV").unwrap();

    let app_clone = app.clone();
    let registered = app.global_shortcut().on_shortcut(primary, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            show_popup(&app_clone);
        }
    });

    if registered.is_err() {
        log::warn!("Win+V unavailable, falling back to Win+Shift+V");
        let app_clone2 = app.clone();
        app.global_shortcut()
            .on_shortcut(fallback, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    show_popup(&app_clone2);
                }
            })
            .expect("Failed to register fallback hotkey");
    }
}

fn show_popup(app: &AppHandle) {
    use tauri::Manager;

    if let Some(window) = app.get_webview_window("main") {
        // Position at bottom-right of the primary monitor
        if let Ok(Some(monitor)) = window.primary_monitor() {
            let size = monitor.size();
            let scale = monitor.scale_factor();
            let w_width = 680.0;
            let w_height = 480.0;
            let x = (size.width as f64 / scale) - w_width - 20.0;
            let y = (size.height as f64 / scale) - w_height - 60.0;

            let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
        }

        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.emit("popup-shown", ());
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/hotkey/
git commit -m "feat: add global hotkey with smart Win+V/Win+Shift+V fallback"
```

---

### Task 7: Tauri 命令层

**Files:**
- Create: `src-tauri/src/commands.rs`

- [ ] **Step 1: 编写所有 Tauri 命令**

Write `src-tauri/src/commands.rs`:
```rust
use crate::history::{manager, search, cleanup};
use crate::clipboard::writer;
use crate::AppState;
use rusqlite::params;
use tauri::State;

#[tauri::command]
pub fn search_history(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<manager::ClipboardItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    search::search(&conn, &query, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_recent(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<manager::ClipboardItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::get_recent(&conn, limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::delete_item(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_all(state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::clear_all(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn paste_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get content
    let content: String = conn
        .query_row(
            "SELECT content FROM clipboard_items WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .map_err(|e| format!("Item not found: {}", e))?;

    // Update last_used_at
    let _ = manager::update_last_used(&conn, id);

    // Hide window first so it doesn't receive the paste
    // (handled by frontend before calling this)

    // Write to clipboard and paste
    writer::write_text_and_paste(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let map = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(map)
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    for (key, value) in &settings {
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: add Tauri command layer for search, CRUD, paste, settings"
```

---

### Task 8: 前端 — 类型定义 & useClipboard Hook

**Files:**
- Create: `src/hooks/useClipboard.ts`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建 useClipboard hook**

Write `src/hooks/useClipboard.ts`:
```typescript
import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface ClipboardItem {
  id: number;
  content: string;
  source_app: string | null;
  created_at: string;
  last_used_at: string | null;
}

export function useClipboard() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);

  const fetchItems = useCallback(async (searchQuery?: string) => {
    setLoading(true);
    try {
      const q = searchQuery ?? query;
      if (q.trim()) {
        const results = await invoke<ClipboardItem[]>("search_history", {
          query: q,
          limit: 50,
        });
        setItems(results);
      } else {
        const results = await invoke<ClipboardItem[]>("get_recent", {
          limit: 50,
          offset: 0,
        });
        setItems(results);
      }
      setSelectedIndex(0);
    } catch (err) {
      console.error("Failed to fetch items:", err);
    } finally {
      setLoading(false);
    }
  }, [query]);

  const search = useCallback((q: string) => {
    setQuery(q);
  }, []);

  const deleteItem = useCallback(async (id: number) => {
    try {
      await invoke("delete_item", { id });
      setItems((prev) => prev.filter((item) => item.id !== id));
    } catch (err) {
      console.error("Failed to delete item:", err);
    }
  }, []);

  const clearAll = useCallback(async () => {
    try {
      await invoke("clear_all");
      setItems([]);
    } catch (err) {
      console.error("Failed to clear all:", err);
    }
  }, []);

  const pasteItem = useCallback(async (id: number) => {
    try {
      await invoke("paste_item", { id });
    } catch (err) {
      console.error("Failed to paste:", err);
    }
  }, []);

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(() => {
      fetchItems(query);
    }, 150);
    return () => clearTimeout(timer);
  }, [query, fetchItems]);

  // Listen for clipboard changes
  useEffect(() => {
    const unlisten = listen<string>("clipboard-changed", () => {
      // Re-fetch if showing recent items (not searching)
      if (!query.trim()) {
        fetchItems();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [query, fetchItems]);

  return {
    items,
    query,
    loading,
    selectedIndex,
    setSelectedIndex,
    search,
    deleteItem,
    clearAll,
    pasteItem,
    fetchItems,
  };
}
```

- [ ] **Step 2: 更新 App 组件使用 hook**

Write `src/App.tsx`:
```typescript
import { useClipboard } from "./hooks/useClipboard";

function App() {
  const clipboard = useClipboard();

  return (
    <div className="h-screen bg-gray-950 text-gray-200 flex flex-col select-none">
      <div className="text-center text-gray-500 mt-20">
        弹出窗口 — 按下 Win+Shift+V 打开搜索
      </div>
    </div>
  );
}

export default App;
```

- [ ] **Step 3: 编译验证**

```bash
cd F:/projects/Clipboard
npx tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add src/hooks/useClipboard.ts src/App.tsx
git commit -m "feat: add useClipboard hook with search, CRUD, and event listening"
```

---

### Task 9: 前端 — SearchBar & ResultItem 组件

**Files:**
- Create: `src/components/SearchBar.tsx`
- Create: `src/components/ResultItem.tsx`

- [ ] **Step 1: 编写 SearchBar 组件**

Write `src/components/SearchBar.tsx`:
```typescript
import { useRef, useEffect } from "react";

interface Props {
  query: string;
  onChange: (query: string) => void;
  onEscape: () => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
}

export function SearchBar({ query, onChange, onEscape, onKeyDown }: Props) {
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
```

- [ ] **Step 2: 编写 ResultItem 组件**

Write `src/components/ResultItem.tsx`:
```typescript
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
  const regex = new RegExp(`(${escapedQuery})`, "gi");

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
```

- [ ] **Step 3: 编译验证**

```bash
cd F:/projects/Clipboard
npx tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add src/components/SearchBar.tsx src/components/ResultItem.tsx
git commit -m "feat: add SearchBar and ResultItem components"
```

---

### Task 10: 前端 — ResultList & StatusBar 组件

**Files:**
- Create: `src/components/ResultList.tsx`
- Create: `src/components/StatusBar.tsx`

- [ ] **Step 1: 编写 ResultList 组件**

Write `src/components/ResultList.tsx`:
```typescript
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
```

- [ ] **Step 2: 编写 StatusBar 组件**

Write `src/components/StatusBar.tsx`:
```typescript
interface Props {
  totalCount: number;
  matchCount: number;
  query: string;
}

export function StatusBar({ totalCount, matchCount, query }: Props) {
  return (
    <div className="px-3 py-1.5 bg-gray-900 border-t border-gray-800 flex justify-between items-center text-[11px] text-gray-600">
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
```

- [ ] **Step 3: 编译验证**

```bash
cd F:/projects/Clipboard
npx tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add src/components/ResultList.tsx src/components/StatusBar.tsx
git commit -m "feat: add ResultList and StatusBar components"
```

---

### Task 11: 前端 — App 组件集成 & 窗口控制

**Files:**
- Create: `src/hooks/useHotkey.ts`
- Modify: `src/App.tsx`
- Modify: `src/App.css`

- [ ] **Step 1: 编写 useHotkey hook**

Write `src/hooks/useHotkey.ts`:
```typescript
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function useHotkey() {
  useEffect(() => {
    const appWindow = getCurrentWindow();

    const unlistenShow = listen("popup-shown", () => {
      appWindow.show();
      appWindow.setFocus();
    });

    // Listen for Escape key to hide window
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        appWindow.hide();
      }
    };
    window.addEventListener("keydown", handleKeyDown);

    // Hide window on blur (click outside)
    const unlistenBlur = appWindow.onFocusChanged(({ payload: focused }) => {
      if (!focused) {
        appWindow.hide();
      }
    });

    return () => {
      unlistenShow.then((fn) => fn());
      unlistenBlur.then((fn) => fn());
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);
}
```

- [ ] **Step 2: 编写完整 App 组件**

Write `src/App.tsx`:
```typescript
import { useCallback, useEffect } from "react";
import { useClipboard } from "./hooks/useClipboard";
import { useHotkey } from "./hooks/useHotkey";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { StatusBar } from "./components/StatusBar";
import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
  const {
    items,
    query,
    loading,
    selectedIndex,
    setSelectedIndex,
    search,
    deleteItem,
    clearAll,
    pasteItem,
    fetchItems,
  } = useClipboard();

  useHotkey();

  // Initial load
  useEffect(() => {
    fetchItems();
  }, []);

  // Keyboard navigation
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < items.length - 1 ? prev + 1 : prev
        );
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        if (items[selectedIndex]) {
          const appWindow = getCurrentWindow();
          appWindow.hide().then(() => {
            pasteItem(items[selectedIndex].id);
          });
        }
      } else if (e.key === "Delete" && items[selectedIndex]) {
        e.preventDefault();
        deleteItem(items[selectedIndex].id);
      } else if (e.key === "Escape") {
        e.preventDefault();
        getCurrentWindow().hide();
      }
    },
    [items, selectedIndex, setSelectedIndex, pasteItem, deleteItem]
  );

  return (
    <div className="h-screen bg-gray-950 text-gray-200 flex flex-col select-none">
      {/* Title bar */}
      <div
        data-tauri-drag-region
        className="flex items-center justify-between px-3 py-1.5 bg-gray-900 border-b border-gray-800"
      >
        <span className="text-xs font-semibold text-gray-400">
          aPaste
        </span>
        <button
          onClick={clearAll}
          className="text-[10px] text-gray-600 hover:text-red-400 transition-colors"
          title="清空全部"
        >
          清空全部
        </button>
      </div>

      <SearchBar
        query={query}
        onChange={search}
        onEscape={() => getCurrentWindow().hide()}
        onKeyDown={handleKeyDown}
      />

      <ResultList
        items={items}
        query={query}
        selectedIndex={selectedIndex}
        loading={loading}
        onSelect={(id) => {
          const appWindow = getCurrentWindow();
          appWindow.hide().then(() => pasteItem(id));
        }}
        onDelete={deleteItem}
      />

      <StatusBar
        totalCount={items.length}
        matchCount={items.length}
        query={query}
      />
    </div>
  );
}

export default App;
```

- [ ] **Step 3: 编写 App.css 基础样式**

Write `src/App.css`:
```css
/* Custom scrollbar */
::-webkit-scrollbar {
  width: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #374151;
  border-radius: 3px;
}
::-webkit-scrollbar-thumb:hover {
  background: #4b5563;
}

/* Line clamp utility */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
```

- [ ] **Step 4: 编译验证**

```bash
cd F:/projects/Clipboard
npx tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 5: Commit**

```bash
git add src/hooks/useHotkey.ts src/App.tsx src/App.css
git commit -m "feat: integrate App shell with keyboard nav and window control"
```

---

### Task 12: 系统托盘

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 在 lib.rs 中添加系统托盘**

Modify `src-tauri/src/lib.rs` — 在 `setup` 闭包中添加托盘初始化（在 `Ok(())` 之前）:

```rust
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

// ... inside setup, before Ok(()):

// Build tray menu
let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
let settings_item = MenuItemBuilder::with_id("settings", "设置").build(app)?;
let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

let menu = MenuBuilder::new(app)
    .item(&show_item)
    .item(&settings_item)
    .separator()
    .item(&quit_item)
    .build()?;

let _tray = TrayIconBuilder::new()
    .icon(app.default_window_icon().unwrap().clone())
    .tooltip("aPaste")
    .menu(&menu)
    .on_menu_event(|app, event| {
        match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "settings" => {
                // Will be handled in settings task
                log::info!("Settings menu clicked");
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        }
    })
    .on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })
    .build(app)?;
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add system tray with menu and left-click to show"
```

---

### Task 13: 设置页面 & 配置持久化

**Files:**
- Create: `src/components/Settings.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 编写 Settings 组件**

Write `src/components/Settings.tsx`:
```typescript
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SettingsData {
  max_items: string;
  max_days: string;
  hotkey: string;
  autostart: string;
}

export function Settings() {
  const [settings, setSettings] = useState<SettingsData>({
    max_items: "1000",
    max_days: "30",
    hotkey: "Win+Shift+V",
    autostart: "true",
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<Record<string, string>>("get_settings").then((data) => {
      setSettings({
        max_items: data.max_items || "1000",
        max_days: data.max_days || "30",
        hotkey: data.hotkey || "Win+Shift+V",
        autostart: data.autostart || "true",
      });
    });
  }, []);

  const update = (key: keyof SettingsData, value: string) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setSaved(false);
  };

  const save = async () => {
    const map: Record<string, string> = {};
    for (const [k, v] of Object.entries(settings)) {
      map[k] = v;
    }
    await invoke("update_settings", { settings: map });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="h-screen bg-gray-950 text-gray-200 flex flex-col">
      <div
        data-tauri-drag-region
        className="px-4 py-3 bg-gray-900 border-b border-gray-800"
      >
        <h2 className="text-sm font-semibold text-gray-300">设置</h2>
      </div>

      <div className="flex-1 p-4 space-y-5 overflow-y-auto">
        <div>
          <label className="block text-xs text-gray-500 mb-1.5">最大保留条数</label>
          <input
            type="number"
            value={settings.max_items}
            onChange={(e) => update("max_items", e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            min="100"
            max="10000"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-500 mb-1.5">最大保留天数</label>
          <input
            type="number"
            value={settings.max_days}
            onChange={(e) => update("max_days", e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            min="1"
            max="365"
          />
        </div>

        <div>
          <label className="block text-xs text-gray-500 mb-1.5">快捷键</label>
          <input
            type="text"
            value={settings.hotkey}
            onChange={(e) => update("hotkey", e.target.value)}
            className="w-full bg-gray-900 border border-gray-700 rounded px-3 py-1.5 text-sm text-gray-200 focus:border-rose-500 outline-none"
            readOnly
          />
          <p className="text-[10px] text-gray-600 mt-1">
            修改快捷键请在 Windows 系统设置中更改
          </p>
        </div>

        <div className="flex items-center justify-between">
          <label className="text-xs text-gray-500">开机自动启动</label>
          <button
            onClick={() =>
              update("autostart", settings.autostart === "true" ? "false" : "true")
            }
            className={`w-10 h-5 rounded-full transition-colors ${
              settings.autostart === "true" ? "bg-rose-500" : "bg-gray-700"
            }`}
          >
            <div
              className={`w-4 h-4 bg-white rounded-full transition-transform mx-0.5 ${
                settings.autostart === "true" ? "translate-x-4" : ""
              }`}
            />
          </button>
        </div>
      </div>

      <div className="px-4 py-3 bg-gray-900 border-t border-gray-800">
        <button
          onClick={save}
          className="w-full py-1.5 bg-rose-500 hover:bg-rose-600 text-white text-sm rounded transition-colors"
        >
          {saved ? "已保存" : "保存设置"}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard
npx tsc --noEmit
```

Expected: 无类型错误

- [ ] **Step 3: Commit**

```bash
git add src/components/Settings.tsx
git commit -m "feat: add Settings page with persistent config"
```

---

### Task 14: 开机自启 & 定时清理

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create: `src-tauri/src/history/cleanup.rs` (update with the scheduled version)

- [ ] **Step 1: 在 lib.rs setup 中添加自启注册和定时清理**

在 `src-tauri/src/lib.rs` 的 `setup` 闭包末尾（`Ok(())` 之前）添加:

```rust
// Check autostart setting and register
let autostart: String = conn
    .query_row("SELECT value FROM settings WHERE key = 'autostart'", [], |r| r.get(0))
    .unwrap_or_else(|_| "true".into());

if autostart == "true" {
    set_autostart(true);
}

// Spawn periodic cleanup task (runs every hour)
let state_cleanup = state.clone();
tokio::spawn(async move {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;

        let conn = state_cleanup.db.lock().unwrap();
        let max_items: i64 = conn
            .query_row("SELECT value FROM settings WHERE key = 'max_items'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1000);

        let max_days: i64 = conn
            .query_row("SELECT value FROM settings WHERE key = 'max_days'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        if let Err(e) = crate::history::cleanup::run_cleanup(&conn, max_items, max_days) {
            log::error!("Cleanup failed: {}", e);
        }
    }
});
```

在文件末尾添加 `set_autostart` 辅助函数:

```rust
fn set_autostart(enable: bool) {
    use windows::Win32::System::Registry::{
        RegSetValueExW, RegCreateKeyExW, HKEY_CURRENT_USER,
        REG_SZ, KEY_WRITE, REG_OPTION_NON_VOLATILE,
    };
    use windows::Win32::System::Ole::GetForegroundWindow;

    unsafe {
        let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let value_name = "aPaste";

        let mut hkey = windows::Win32::System::Registry::HKEY::default();
        let result = RegCreateKeyExW(
            HKEY_CURRENT_USER,
            windows::core::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run"),
            0,
            None,
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut hkey,
            None,
        );

        if result.is_ok() {
            if enable {
                if let Ok(exe_path) = std::env::current_exe() {
                    let path = exe_path.to_string_lossy();
                    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
                    let _ = RegSetValueExW(
                        hkey,
                        windows::core::w!("aPaste"),
                        0,
                        REG_SZ,
                        Some(std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2)),
                    );
                }
            } else {
                // Delete the registry key
                let _ = windows::Win32::System::Registry::RegDeleteValueW(
                    hkey,
                    windows::core::w!("aPaste"),
                );
            }
        }
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cd F:/projects/Clipboard/src-tauri
cargo check
```

Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add autostart registry and periodic cleanup task"
```

---

### Task 15: 图标资源 & 最终集成验证

**Files:**
- Create: `src-tauri/icons/32x32.png`
- Create: `src-tauri/icons/128x128.png`
- Create: `src-tauri/icons/128x128@2x.png`
- Create: `src-tauri/icons/icon.ico`

- [ ] **Step 1: 生成占位图标**

```bash
cd F:/projects/Clipboard
cargo tauri icon --help 2>/dev/null || echo "Using placeholder icon generation"
# If tauri CLI icon generator is available:
cargo tauri icon src-tauri/icons/icon.png 2>/dev/null || true
```

Note: 如果本地没有原图，使用 `cargo tauri icon` 需要先提供一个 1024x1024 的 PNG。可以先从在线工具生成一个简单图标，或暂时使用 Tauri 默认图标。

- [ ] **Step 2: 全文编译验证**

```bash
cd F:/projects/Clipboard
cargo tauri build --debug 2>&1 | tail -20
```

Expected: 构建成功，生成 `src-tauri/target/debug/clipboard-manager.exe`

- [ ] **Step 3: 功能验证清单**

- [ ] 启动应用 → 系统托盘图标出现
- [ ] 复制文本 → 数据库自动记录
- [ ] 按下快捷键 → 右下角弹出窗口
- [ ] 输入搜索词 → 模糊匹配结果显示
- [ ] ↑↓ 浏览 → Enter 粘贴 → 内容写入目标
- [ ] 失去焦点 / Esc → 窗口隐藏
- [ ] 右键托盘图标 → 显示菜单
- [ ] 点击托盘图标 → 窗口弹出
- [ ] 修改设置 → 保存 → 重启后生效
- [ ] 重启电脑 → 数据保留

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: add icons and final integration verification"
```

---

## 执行顺序

```
Task 1  → 项目脚手架
Task 2  → 数据库层
Task 3  → 剪切板监听 (依赖 Task 2)
Task 4  → 历史管理 (依赖 Task 2)
Task 5  → 剪切板写入 (独立)
Task 6  → 热键管理 (独立)
Task 7  → Tauri 命令层 (依赖 Task 3,4,5)
Task 8  → useClipboard Hook (依赖 Task 7)
Task 9  → SearchBar & ResultItem (独立组件)
Task 10 → ResultList & StatusBar (依赖 Task 9)
Task 11 → App 集成 (依赖 Task 8,9,10)
Task 12 → 系统托盘 (依赖 Task 1)
Task 13 → 设置页面 (依赖 Task 7)
Task 14 → 开机自启 & 定时清理 (依赖 Task 2,3)
Task 15 → 图标 & 最终验证 (依赖全部)
```
