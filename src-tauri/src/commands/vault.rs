use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Runtime, State};
use znotes_core::repo::notes;
use znotes_core::{BackupPaths, Error, KdfParams, Result, Vault};

use crate::state::{AppState, VaultInfo, VaultStatus};
use crate::watchdog;

#[tauri::command(async)]
pub fn vault_create(
    state: State<'_, AppState>,
    dir: String,
    password: String,
) -> Result<VaultInfo> {
    let vault = Vault::create(Path::new(&dir), &password, KdfParams::default())?;
    Ok(state.set_vault(vault))
}

#[tauri::command(async)]
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

#[tauri::command(async)]
pub fn vault_backup(state: State<'_, AppState>, dest_dir: String) -> Result<BackupPaths> {
    state.with_vault(|v| v.backup_to(Path::new(&dest_dir), &timestamp()))
}

#[tauri::command(async)]
pub fn vault_change_password(
    state: State<'_, AppState>,
    current: String,
    new: String,
) -> Result<()> {
    state.with_vault_mut(|v| v.change_password(&current, &new))
}

#[tauri::command(async)]
pub fn note_export_md(state: State<'_, AppState>, id: i64, dest_path: String) -> Result<()> {
    let note = state.with_vault(|v| notes::get(v.conn(), id))?;
    let dest = Path::new(&dest_path);
    if dest.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(Error::invalid("export path must end with .md"));
    }
    std::fs::write(dest, note.body)?;
    Ok(())
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}{m:02}{d:02}-{:02}{:02}",
        rem / 3600,
        (rem % 3600) / 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_are_correct() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_669), (2023, 11, 8));
        assert_eq!(civil_from_days(20_765), (2026, 11, 8));
    }

    #[test]
    fn timestamp_has_expected_shape() {
        let ts = timestamp();
        assert_eq!(ts.len(), 13);
        assert_eq!(&ts[8..9], "-");
    }
}
