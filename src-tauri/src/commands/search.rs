use tauri::State;
use znotes_core::Result;
use znotes_core::repo::search::{self, SearchHit};

use crate::state::AppState;

#[tauri::command]
pub fn search(state: State<'_, AppState>, query: String, limit: u32) -> Result<Vec<SearchHit>> {
    state.with_vault(|v| search::search(v.conn(), &query, limit))
}
