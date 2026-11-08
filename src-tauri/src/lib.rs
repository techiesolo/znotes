mod commands;
pub mod state;
mod watchdog;

use state::AppState;

#[cfg(target_os = "linux")]
fn set_webkitgtk_env_before_threads_start() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
    }
}

#[cfg(not(target_os = "linux"))]
fn set_webkitgtk_env_before_threads_start() {}

pub fn configure<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::vault::vault_create,
            commands::vault::vault_open,
            commands::vault::vault_lock,
            commands::vault::vault_status,
            commands::folders::folder_list,
            commands::folders::folder_create,
            commands::folders::folder_rename,
            commands::folders::folder_delete,
            commands::notes::note_list,
            commands::notes::note_get,
            commands::notes::note_create,
            commands::notes::note_update,
            commands::notes::note_move,
            commands::notes::note_delete,
            commands::search::search,
        ])
}

pub fn run() {
    set_webkitgtk_env_before_threads_start();
    configure(tauri::Builder::default())
        .setup(|app| {
            watchdog::spawn(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running znotes");
}
