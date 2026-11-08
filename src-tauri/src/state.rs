use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use znotes_core::{Error, Result, Vault};

pub const DEFAULT_AUTOLOCK: Duration = Duration::from_secs(10 * 60);

#[derive(Debug, Clone, Serialize)]
pub struct VaultStatus {
    pub locked: bool,
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VaultInfo {
    pub dir: String,
}

impl VaultInfo {
    pub fn from_dir(dir: &Path) -> Self {
        Self {
            dir: dir.display().to_string(),
        }
    }
}

pub struct AppState {
    vault: Mutex<Option<Vault>>,
    last_activity: Mutex<Instant>,
    autolock_millis: AtomicU64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            vault: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
            autolock_millis: AtomicU64::new(DEFAULT_AUTOLOCK.as_millis() as u64),
        }
    }
}

impl AppState {
    pub fn with_vault<T>(&self, f: impl FnOnce(&Vault) -> Result<T>) -> Result<T> {
        self.touch();
        let guard = self.vault.lock().unwrap_or_else(|e| e.into_inner());
        let vault = guard.as_ref().ok_or(Error::Locked)?;
        f(vault)
    }

    pub fn with_vault_mut<T>(&self, f: impl FnOnce(&mut Vault) -> Result<T>) -> Result<T> {
        self.touch();
        let mut guard = self.vault.lock().unwrap_or_else(|e| e.into_inner());
        let vault = guard.as_mut().ok_or(Error::Locked)?;
        f(vault)
    }

    pub fn set_vault(&self, vault: Vault) -> VaultInfo {
        self.touch();
        let info = VaultInfo::from_dir(vault.dir());
        *self.vault.lock().unwrap_or_else(|e| e.into_inner()) = Some(vault);
        info
    }

    pub fn take_vault(&self) -> Option<Vault> {
        self.vault.lock().unwrap_or_else(|e| e.into_inner()).take()
    }

    pub fn status(&self) -> VaultStatus {
        let guard = self.vault.lock().unwrap_or_else(|e| e.into_inner());
        VaultStatus {
            locked: guard.is_none(),
            dir: guard.as_ref().map(|v| v.dir().display().to_string()),
        }
    }

    pub fn touch(&self) {
        *self.last_activity.lock().unwrap_or_else(|e| e.into_inner()) = Instant::now();
    }

    pub fn idle_for(&self) -> Duration {
        self.last_activity
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .elapsed()
    }

    pub fn autolock(&self) -> Duration {
        Duration::from_millis(self.autolock_millis.load(Ordering::Relaxed))
    }

    pub fn set_autolock(&self, duration: Duration) {
        self.autolock_millis
            .store(duration.as_millis() as u64, Ordering::Relaxed);
    }

    pub fn lock_if_idle(&self) -> bool {
        if self.idle_for() < self.autolock() {
            return false;
        }
        self.take_vault().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use znotes_core::KdfParams;

    fn open_vault() -> (tempfile::TempDir, Vault) {
        let dir = tempfile::tempdir().unwrap();
        let vault = Vault::create(dir.path(), "pw", KdfParams::fast_for_tests()).unwrap();
        (dir, vault)
    }

    #[test]
    fn locked_state_returns_locked_error() {
        let state = AppState::default();
        assert!(state.status().locked);
        let result = state.with_vault(|_| Ok(()));
        assert!(matches!(result, Err(Error::Locked)));
    }

    #[test]
    fn set_and_take_round_trip() {
        let (dir, vault) = open_vault();
        let state = AppState::default();
        let info = state.set_vault(vault);
        assert_eq!(info.dir, dir.path().display().to_string());
        assert!(!state.status().locked);
        assert_eq!(state.status().dir.as_deref(), Some(info.dir.as_str()));
        state
            .with_vault(|v| {
                assert_eq!(v.dir(), dir.path());
                Ok(())
            })
            .unwrap();
        assert!(state.take_vault().is_some());
        assert!(state.status().locked);
    }

    #[test]
    fn lock_if_idle_respects_threshold_and_activity() {
        let (_dir, vault) = open_vault();
        let state = AppState::default();
        state.set_vault(vault);
        state.set_autolock(Duration::from_millis(50));
        assert!(!state.lock_if_idle());
        std::thread::sleep(Duration::from_millis(80));
        state.with_vault(|_| Ok(())).unwrap();
        assert!(!state.lock_if_idle());
        std::thread::sleep(Duration::from_millis(80));
        assert!(state.lock_if_idle());
        assert!(state.status().locked);
        assert!(!state.lock_if_idle());
    }

    #[test]
    fn autolock_defaults_to_ten_minutes_and_is_settable() {
        let state = AppState::default();
        assert_eq!(state.autolock(), DEFAULT_AUTOLOCK);
        state.set_autolock(Duration::from_secs(90));
        assert_eq!(state.autolock(), Duration::from_secs(90));
    }
}
