# aPaste

> Clipboard history manager for Windows — popup, search, paste, favorite.

Built with **Tauri v2** + **React 19** + **TypeScript** + **Tailwind CSS v4** (frontend) and **Rust** + **SQLite** (backend).

## Features

- **Clipboard history** — automatically captures text copied to the Windows clipboard
- **Instant search** — full-text search via SQLite FTS5 with prefix matching
- **Quick paste** — Enter to paste the selected item back to any application
- **Favorites** — star items to keep them permanently; immune to cleanup and "Clear All"
- **Global hotkey** — Win+V to summon the popup anywhere (falls back to Win+Shift+V if Win+V is occupied by Windows)
- **Auto-cleanup** — configurable max items / max days; favorites are never deleted
- **Mica blur** — Windows 11 acrylic-style translucent background
- **Dark / Light theme** — switch in Settings, persisted across restarts
- **Portable** — single NSIS installer, no external database server required

## Prerequisites

- **Windows 10/11** (the only supported platform)
- **Node.js** ≥ 18 (with npm)
- **Rust** (install via [rustup.rs](https://rustup.rs))

On first Rust install, the Windows build chain is required:
```bash
rustup default stable-msvc
```

## Quick Start

```bash
# Install frontend dependencies
npm install

# Launch dev server + Rust backend (HMR on port 1420)
npx tauri dev
```

Press **Win+V** (or **Win+Shift+V** as fallback) to open the popup.

## Usage Guide

### Getting Started

1. **Install** — Download the latest `aPaste_0.1.1_x64-setup.exe` from [Releases](https://github.com/codant0/aPaste/releases) and run it.
2. **Launch** — aPaste starts automatically after installation. It runs in the system tray (look for the clipboard icon).
3. **Copy as usual** — Continue using Ctrl+C to copy text. aPaste silently records everything in the background.

### Opening the Popup

Press the global hotkey (**Win+V** by default) to summon the popup window. If Win+V is reserved by Windows (e.g. system shortcut), it automatically falls back to **Win+Shift+V**. The popup appears at the bottom-right corner of your primary monitor, always on top of other windows.

| Action | How |
|---|---|
| Show popup | `Win+V` (or `Win+Shift+V` if Win+V is occupied) |
| Hide popup | `Esc` or click outside the window |
| Show from tray | Left-click the tray icon or right-click → "显示窗口" |

### Browsing and Searching

The popup shows your most recent clipboard entries, newest first. Each item displays:

- **Timestamp** — relative time (刚刚, 5 分钟前, 昨天, etc.)
- **Source app** — the application the text was copied from (e.g. Notepad, Chrome)
- **Content preview** — first 120 characters of the copied text

**Search:** Type in the search bar to filter by keyword. aPaste uses FTS5 full-text search with prefix matching — type `npm` to find "npm install tauri", type `React` to find all React-related entries.

### Pasting an Item

1. Press `Win+V` (or `Win+Shift+V`) to open the popup.
2. Use `↑` / `↓` arrow keys to highlight the item you want.
3. Press `Enter` — the popup hides and the selected text is pasted into the focused application.

> **Tip:** You can also click an item with the mouse to paste it.

### Deleting Items

| Action | How |
|---|---|
| Delete a single item | Hover over the item and click the ✕ button, or select it and press `Delete` |
| Clear all non-favorites | Click "清空全部" in the title bar (two-click confirm to prevent accidents) |

> **Favorites are protected:** "Clear All" skips favorited items. They won't be deleted.

### Using Favorites

Favorites let you pin important clipboard entries so they survive cleanup.

**Star an item:**
1. Hover over any item in the list.
2. Click the ☆ (star) icon that appears in the upper-right corner.
3. The star turns solid gold ★ — the item is now favorited.

**View favorites:**
Click the "收藏项" tab below the search bar to see only your favorited items.

**Un-favorite:**
Hover over a favorited item and click the ★ again. In the favorites tab, the item disappears from the view immediately.

**Search within favorites:**
Switch to the "收藏项" tab, then type in the search bar — only favorited items are searched.

### Settings

Click the gear ⚙ icon in the title bar to open Settings.

| Section | Options |
|---|---|
| **外观 (Appearance)** | Dark / Light theme |
| **历史记录 (History)** | Max items (100–10000, default 1000), Max days (1–365, default 30) |
| **快捷键 (Shortcuts)** | Click to record a new global hotkey combo |
| **系统 (System)** | Auto-start with Windows (toggle on/off) |

Click "保存设置" to apply changes.

### System Tray

Right-click the tray icon for quick actions:

| Action | Description |
|---|---|
| 显示窗口 | Show the popup |
| 设置 | Open Settings |
| 退出 | Quit aPaste completely (clipboard monitoring stops) |

Left-click the tray icon to show the popup.

### Keyboard Shortcuts (Summary)

| Shortcut | Action |
|---|---|
| `Win+V` / `Win+Shift+V` | Show popup (customizable in Settings) |
| `↑` / `↓` | Navigate items |
| `Enter` | Paste selected item |
| `Delete` | Delete selected item |
| `Esc` | Hide popup / cancel |
| `←` (in Settings) | Back to main view |

## Project Structure

```
aPaste/
├── src/                          # React frontend
│   ├── App.tsx                   # Main app: keyboard nav, view routing
│   ├── App.css                   # Global styles & scrollbar
│   ├── hooks/
│   │   ├── useClipboard.ts       # Data fetching, search, favorite, CRUD
│   │   ├── useHotkey.ts          # Window show/hide, blur handling
│   │   └── useTheme.ts           # Dark/light theme persistence
│   └── components/
│       ├── SearchBar.tsx         # Text input with debounce & clear
│       ├── CategoryTabs.tsx      # "All" / "Favorites" tab switcher
│       ├── ResultList.tsx        # Scrollable item list
│       ├── ResultItem.tsx        # Single row: time, app, preview, ★ star
│       ├── StatusBar.tsx         # Shortcut hints + match count
│       └── Settings.tsx          # Theme, limits, hotkey, autostart
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri window & bundle config
│   └── src/
│       ├── main.rs               # Windows entry point
│       ├── lib.rs                # App bootstrap, tray, hotkey, cleanup thread
│       ├── commands.rs           # 12 Tauri IPC commands
│       ├── db/
│       │   ├── connection.rs     # SQLite connection (WAL, NORMAL sync)
│       │   └── migrate.rs        # Schema migrations + seed data
│       ├── clipboard/
│       │   ├── monitor.rs        # Win32 WM_CLIPBOARDUPDATE listener
│       │   └── writer.rs         # Win32 clipboard write + Ctrl+V simulation
│       ├── history/
│       │   ├── manager.rs        # CRUD: add, get, delete, toggle_favorite
│       │   ├── search.rs         # FTS5 prefix search
│       │   └── cleanup.rs        # Periodic cleanup by age & count
│       └── hotkey/
│           └── mod.rs            # Global hotkey register (Win+V fallback)
└── docs/
    └── plans/                    # Design & implementation plans
```

## Commands

| Command | Description |
|---|---|
| `npm install` | Install frontend dependencies |
| `npx tauri dev` | Start dev server + Rust backend with HMR |
| `npx tauri build` | Production build → NSIS installer |
| `npx tsc --noEmit` | TypeScript type-check (no emit) |
| `cargo test` | Run Rust unit tests (in-memory SQLite, no server needed) |

> Run `cargo` commands from `src-tauri/` or pass `--manifest-path src-tauri/Cargo.toml`.

## Architecture

```
Win32 clipboard event (WM_CLIPBOARDUPDATE)
  → monitor.rs: reads text, SHA-256 dedup hash
  → saves to SQLite via manager::add_item
  → emits "clipboard-changed" event → frontend re-fetches

User types in search bar
  → useClipboard hook (150ms debounce) → invoke("search_history") or category-aware
  → search.rs: FTS5 MATCH with prefix wildcard + special-char escaping

User presses Enter on an item
  → invoke("paste_item") → writer.rs: Win32 clipboard write + keybd_event Ctrl+V
```

### Database

Single-file SQLite at `%APPDATA%/apaste/apaste.db`. Three tables:

| Table | Purpose |
|---|---|
| `clipboard_items` | id, content, content_hash, source_app, is_favorite, created_at, last_used_at |
| `clipboard_fts` | FTS5 virtual table (content-sync to clipboard_items) |
| `settings` | key/value store for preferences |

FTS5 is kept in sync via three triggers (after insert/delete/update on `clipboard_items`).

### IPC Commands (12 total)

| Command | Signature |
|---|---|
| `search_history` | `(query, limit?) → ClipboardItem[]` |
| `get_recent` | `(limit?, offset?) → ClipboardItem[]` |
| `delete_item` | `(id) → void` |
| `clear_all` | `() → void` (skips favorites) |
| `paste_item` | `(id) → void` |
| `get_settings` | `() → HashMap<String, String>` |
| `update_settings` | `(settings) → void` |
| `get_count` | `() → i64` |
| `update_hotkey` | `(hotkey_str, app) → String` |
| `toggle_favorite` | `(id) → bool` |
| `get_favorites` | `(limit?, offset?) → ClipboardItem[]` |
| `search_favorites` | `(query, limit?) → ClipboardItem[]` |

## Build Output

After `npx tauri build`:

```
src-tauri/target/release/apaste.exe
src-tauri/target/release/bundle/nsis/aPaste_0.1.0_x64-setup.exe
```

## License

MIT
