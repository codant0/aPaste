# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build / Run

```bash
npm install                   # Install frontend dependencies
npx tauri dev                 # Start dev server + Rust backend (HMR on port 1421)
npx tauri build               # Production build → src-tauri/target/release/bundle/
cargo test                    # Run Rust unit tests (in-memory SQLite, no server needed)
npx tsc --noEmit              # TypeScript type-check (no emit)
```

No Jest/Vitest setup — frontend testing is manual.

## Architecture

**Stack**: Tauri v2 desktop app (Windows only). React 19 + TypeScript + Tailwind CSS v4 frontend, Rust backend, SQLite (rusqlite bundled with FTS5) database.

### Data flow

```
Win32 clipboard event (WM_CLIPBOARDUPDATE)
  → monitor.rs: reads text, SHA-256 dedup hash against last row
  → saves to SQLite via manager::add_item (FTS5 triggers keep index in sync)
  → emits "clipboard-changed" event → frontend re-fetches if no active search query
```

```
User types in search bar
  → useClipboard hook (150ms debounce) → invoke("search_history") or invoke("get_recent")
  → search.rs: FTS5 MATCH with prefix wildcard + special-char escaping
```

```
User presses Enter on an item
  → invoke("paste_item") → writer.rs: Win32 clipboard write + keybd_event Ctrl+V simulation
```

### Rust backend (`src-tauri/src/`)

- **`lib.rs`** — App bootstrap. Creates window with Mica blur effect, installs a Win32 `WM_NCHITTEST` subclass proc that makes the top 36px draggable (title bar). Opens SQLite at `<appdata>/apaste.db`, runs migrations, manages `AppState { db: Arc<Mutex<Connection>> }`. Spawns a cleanup thread that runs every hour. Registers tray icon (show/settings/quit menu, left-click to show). Registers global hotkey handler (Win+V / Win+Shift+V → calls `hotkey::show_popup`).
- **`commands.rs`** — 8 Tauri IPC commands (`#[tauri::command]`): `search_history`, `get_recent`, `delete_item`, `clear_all`, `paste_item`, `get_settings`, `update_settings`, `get_count`. All lock `AppState.db`.
- **`clipboard/monitor.rs`** — Creates a hidden message-only Win32 window, registers as clipboard format listener. On `WM_CLIPBOARDUPDATE`, reads `CF_UNICODETEXT`, computes SHA-256 hash (first 8 bytes as hex) for dedup, saves via `manager::add_item`, and emits `"clipboard-changed"` event. Gets foreground window title as `source_app`.
- **`clipboard/writer.rs`** — `write_text_and_paste(text)`: Win32 clipboard write (`OpenClipboard` → `EmptyClipboard` → `GlobalAlloc` → `SetClipboardData`) + 30ms delay + `keybd_event` Ctrl+V simulation.
- **`db/connection.rs`** — Opens SQLite with WAL mode, NORMAL synchronous, foreign keys on.
- **`db/migrate.rs`** — Creates `clipboard_items` table, `clipboard_fts` FTS5 virtual table (content-sync to `clipboard_items`), three triggers (after insert/delete/update) that keep FTS in sync, and `settings` table with defaults (max_items=1000, max_days=30, hotkey, autostart).
- **`history/manager.rs`** — `add_item` skips insert if last row's `content_hash` matches (consecutive duplicate suppression). `get_recent` returns items ordered by id DESC. `delete_item`, `clear_all` (also clears FTS), `update_last_used`.
- **`history/search.rs`** — FTS5 prefix search. Escapes FTS5 special characters (`*"()-:^`), appends `*` for prefix matching. Empty query returns all recent items. Joins `clipboard_items` with `clipboard_fts` on rowid.
- **`history/cleanup.rs`** — `run_cleanup`: deletes items older than `max_days`, then deletes oldest items if count exceeds `max_items`.
- **`hotkey/mod.rs`** — Registers global hotkey: tries Win+V first, falls back to Win+Shift+V if Win+V is reserved by Windows. `show_popup` positions window at bottom-right of primary monitor (340×520px logical), shows it, and emits `"popup-shown"`.

### Frontend (`src/`)

- **`App.tsx`** — Single-page popup window. Keyboard-driven: ArrowDown/Up to navigate, Enter to paste (hides window first), Delete to remove, Escape to hide. Title bar with `data-tauri-drag-region` (Win32 subclass handles actual drag).
- **`hooks/useClipboard.ts`** — Central data hook. `fetchItems` dispatches to `search_history` or `get_recent` based on query emptiness. 150ms debounced re-fetch on query change. Listens for `"clipboard-changed"` event to auto-refresh when no search query is active.
- **`hooks/useHotkey.ts`** — Hides window on Escape keydown, hides on focus loss (click outside), shows on `"popup-shown"` event from Rust.
- **`components/SearchBar.tsx`** — Text input with auto-focus on mount, clear button, Esc shortcut badge.
- **`components/ResultList.tsx`** — Renders items or empty state ("no results" vs "no history").
- **`components/ResultItem.tsx`** — Single result: highlights query matches via `dangerouslySetInnerHTML` (escapes HTML first), shows relative time, source app, content preview clamped to 120 chars, delete button.
- **`components/StatusBar.tsx`** — Keyboard shortcut hints + match count.
- **`components/Settings.tsx`** — Settings form: max_items, max_days, hotkey (read-only), autostart toggle. Loads/saves via `get_settings`/`update_settings` commands.

### Database schema

Two tables: `clipboard_items` (id, content, content_hash, source_app, image BLOB for future, created_at, last_used_at) and `settings` (key/value). FTS5 virtual table `clipboard_fts` mirrors content for full-text search, kept in sync via SQLite triggers.

### Key behaviors

- **Consecutive duplicate suppression**: If clipboard content has the same SHA-256 hash (first 8 bytes) as the most recent row, the insert is skipped entirely.
- **Window auto-hide on blur**: The popup hides when it loses focus (click outside), not just on Escape.
- **Window positioning**: On each hotkey press, the window is repositioned to bottom-right of the primary monitor (accounting for DPI scale factor).
- **Autostart**: On launch, checks `settings.autostart` and toggles `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\aPaste` registry key accordingly via `set_autostart()`.
- **Periodic cleanup**: Background thread runs every 3600s, reads `max_items`/`max_days` from settings, calls `run_cleanup`.
