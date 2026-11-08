use tauri::State;
use znotes_core::Result;
use znotes_core::repo::notes::{self, Note, NoteSummary};

use crate::state::AppState;

#[tauri::command]
pub fn note_list(state: State<'_, AppState>, folder_id: Option<i64>) -> Result<Vec<NoteSummary>> {
    state.with_vault(|v| notes::list(v.conn(), folder_id))
}

#[tauri::command]
pub fn note_get(state: State<'_, AppState>, id: i64) -> Result<Note> {
    state.with_vault(|v| notes::get(v.conn(), id))
}

#[tauri::command]
pub fn note_create(
    state: State<'_, AppState>,
    folder_id: Option<i64>,
    title: String,
) -> Result<Note> {
    state.with_vault(|v| notes::create(v.conn(), folder_id, &title))
}

#[tauri::command]
pub fn note_update(
    state: State<'_, AppState>,
    id: i64,
    title: String,
    body: String,
) -> Result<Note> {
    state.with_vault(|v| notes::update(v.conn(), id, &title, &body))
}

#[tauri::command]
pub fn note_move(state: State<'_, AppState>, id: i64, folder_id: Option<i64>) -> Result<Note> {
    state.with_vault(|v| notes::move_to(v.conn(), id, folder_id))
}

#[tauri::command]
pub fn note_delete(state: State<'_, AppState>, id: i64) -> Result<()> {
    state.with_vault(|v| notes::delete(v.conn(), id))
}
