use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::state::AppState;

pub const EVENT_LOCKED: &str = "vault-locked";
const TICK: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Serialize)]
pub struct LockedPayload {
    pub reason: &'static str,
}

pub fn spawn(app: AppHandle) {
    thread::Builder::new()
        .name("znotes-autolock".into())
        .spawn(move || {
            loop {
                thread::sleep(TICK);
                if app.state::<AppState>().lock_if_idle() {
                    emit_locked(&app, "idle");
                }
            }
        })
        .expect("spawn autolock thread");
}

pub fn emit_locked<R: Runtime>(app: &AppHandle<R>, reason: &'static str) {
    let _ = app.emit(EVENT_LOCKED, LockedPayload { reason });
}
