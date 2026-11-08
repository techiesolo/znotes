use std::path::Path;

use tauri::{AppHandle, Runtime, State};
use znotes_core::{KdfParams, Result, Vault};

use crate::state::{AppState, VaultInfo, VaultStatus};
use crate::watchdog;

#[tauri::command]
pub fn vault_create(
    state: State<'_, AppState>,
    dir: String,
    password: String,
) -> Result<VaultInfo> {
    let vault = Vault::create(Path::new(&dir), &password, KdfParams::default())?;
    Ok(state.set_vault(vault))
}

#[tauri::command]
pub fn vault_open(state: State<'_, AppState>, dir: String, password: String) -> Result<VaultInfo> {
    let vault = Vault::open(Path::new(&dir), &password)?;
    Ok(state.set_vault(vault))
}

#[tauri::command]
pub fn vault_lock<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) -> Result<()> {
    if state.take_vault().is_some() {
        watchdog::emit_locked(&app, "manual");
    }
    Ok(())
}

#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    state.status()
}
