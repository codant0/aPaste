# aPaste

> Clipboard history manager for Windows — popup, search, paste, favorite.

Built with **Tauri v2** + **React 19** + **TypeScript** + **Tailwind CSS v4** (frontend) and **Rust** + **SQLite** (backend).

## Features

- **Clipboard history** — automatically captures text copied to the Windows clipboard
- **Instant search** — full-text search via SQLite FTS5 with prefix matching
- **Quick paste** — Enter to paste the selected item back to any application
- **Favorites** — star items to keep them permanently; immune to cleanup and "Clear All"
- **Global hotkey** — Win+Shift+V to summon the popup anywhere
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

Press **Win+Shift+V** to open the popup.

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
