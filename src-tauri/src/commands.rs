// Tauri commands — stubs, will be implemented in later tasks

use tauri::State;
use crate::AppState;

#[tauri::command]
pub fn search_history(state: State<AppState>, query: String) -> Result<String, String> {
    todo!("search_history")
}

#[tauri::command]
pub fn get_recent(state: State<AppState>, limit: u32) -> Result<String, String> {
    todo!("get_recent")
}

#[tauri::command]
pub fn delete_item(state: State<AppState>, id: i64) -> Result<(), String> {
    todo!("delete_item")
}

#[tauri::command]
pub fn clear_all(state: State<AppState>) -> Result<(), String> {
    todo!("clear_all")
}

#[tauri::command]
pub fn paste_item(state: State<AppState>, id: i64) -> Result<(), String> {
    todo!("paste_item")
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<String, String> {
    todo!("get_settings")
}

#[tauri::command]
pub fn update_settings(state: State<AppState>, settings: String) -> Result<(), String> {
    todo!("update_settings")
}
