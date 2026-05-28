use crate::history::{manager, search};
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
