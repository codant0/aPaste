use crate::history::{manager, search};
use crate::clipboard::writer;
use crate::hotkey;
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
    if let Err(e) = manager::update_last_used(&conn, id) {
        log::warn!("Failed to update last_used for item {}: {}", id, e);
    }

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
        .filter_map(|r| match r {
            Ok(pair) => Some(pair),
            Err(e) => {
                log::warn!("Failed to parse setting row: {}", e);
                None
            }
        })
        .collect();

    Ok(map)
}

#[tauri::command]
pub fn get_count(state: State<'_, AppState>) -> Result<i64, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.query_row("SELECT COUNT(*) FROM clipboard_items", [], |r| r.get(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    settings: std::collections::HashMap<String, String>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for (key, value) in &settings {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_favorite(state: State<'_, AppState>, id: i64) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::toggle_favorite(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn rename_favorite(state: State<'_, AppState>, id: i64, name: Option<String>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::rename_favorite(&conn, id, name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_favorites(
    state: State<'_, AppState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<manager::ClipboardItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    manager::get_favorites(&conn, limit.unwrap_or(50), offset.unwrap_or(0))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_favorites(
    state: State<'_, AppState>,
    query: String,
    limit: Option<i64>,
) -> Result<Vec<manager::ClipboardItem>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    search::search_favorites(&conn, &query, limit.unwrap_or(50)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_hotkey(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    hotkey_str: String,
) -> Result<String, String> {
    // Save to DB first, then register. If registration fails, rollback.
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Read old value for rollback
    let old_hotkey: String = conn
        .query_row("SELECT value FROM settings WHERE key = 'hotkey'", [], |r| r.get(0))
        .unwrap_or_else(|_| "Win+Shift+V".into());

    // Write new value to DB
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('hotkey', ?1)",
        params![hotkey_str],
    )
    .map_err(|e| e.to_string())?;

    // Release DB lock before registering hotkey
    drop(conn);

    // Register the hotkey
    match hotkey::re_register(&app, &hotkey_str) {
        Ok(actual) => Ok(actual),
        Err(e) => {
            // Rollback DB to old value
            if let Ok(conn) = state.db.lock() {
                let _ = conn.execute(
                    "INSERT OR REPLACE INTO settings (key, value) VALUES ('hotkey', ?1)",
                    params![old_hotkey],
                );
            }
            Err(e)
        }
    }
}
