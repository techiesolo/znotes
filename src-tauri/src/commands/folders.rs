use tauri::State;
use znotes_core::Result;
use znotes_core::repo::folders::{self, Folder};

use crate::state::AppState;

#[tauri::command]
pub fn folder_list(state: State<'_, AppState>) -> Result<Vec<Folder>> {
    state.with_vault(|v| folders::list(v.conn()))
}

#[tauri::command]
pub fn folder_create(
    state: State<'_, AppState>,
    parent_id: Option<i64>,
    name: String,
) -> Result<Folder> {
    state.with_vault(|v| folders::create(v.conn(), parent_id, &name))
}

#[tauri::command]
pub fn folder_rename(state: State<'_, AppState>, id: i64, name: String) -> Result<Folder> {
    state.with_vault(|v| folders::rename(v.conn(), id, &name))
}

#[tauri::command]
pub fn folder_delete(state: State<'_, AppState>, id: i64) -> Result<()> {
    state.with_vault(|v| folders::delete(v.conn(), id))
}
