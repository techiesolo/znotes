use std::time::Duration;

use tauri::{AppHandle, Manager, Runtime, State};
use znotes_core::Result;

use crate::settings::{self, Settings};
use crate::state::AppState;

fn config_dir<R: Runtime>(app: &AppHandle<R>) -> Result<std::path::PathBuf> {
    app.path()
        .app_config_dir()
        .map_err(|e| std::io::Error::other(e.to_string()).into())
}

pub fn load_and_apply<R: Runtime>(app: &AppHandle<R>, state: &AppState) -> Settings {
    let loaded = config_dir(app)
        .map(|dir| settings::load(&dir))
        .unwrap_or_default();
    state.set_autolock(Duration::from_secs(u64::from(loaded.autolock_minutes) * 60));
    loaded
}

#[tauri::command]
pub fn settings_get<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) -> Settings {
    load_and_apply(&app, &state)
}

#[tauri::command]
pub fn settings_set<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings> {
    settings.validate()?;
    settings::save(&config_dir(&app)?, &settings)?;
    state.set_autolock(Duration::from_secs(
        u64::from(settings.autolock_minutes) * 60,
    ));
    Ok(settings)
}
