use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use znotes_core::{Error, Result};

pub const FILE_NAME: &str = "settings.json";
pub const MIN_AUTOLOCK_MINUTES: u32 = 1;
pub const MAX_AUTOLOCK_MINUTES: u32 = 120;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub autolock_minutes: u32,
    pub last_vault_dir: Option<String>,
    pub preview: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            autolock_minutes: 10,
            last_vault_dir: None,
            preview: false,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        if !(MIN_AUTOLOCK_MINUTES..=MAX_AUTOLOCK_MINUTES).contains(&self.autolock_minutes) {
            return Err(Error::invalid(format!(
                "autolock must be between {MIN_AUTOLOCK_MINUTES} and {MAX_AUTOLOCK_MINUTES} minutes"
            )));
        }
        Ok(())
    }
}

pub fn path_in(config_dir: &Path) -> PathBuf {
    config_dir.join(FILE_NAME)
}

pub fn load(config_dir: &Path) -> Settings {
    fs::read_to_string(path_in(config_dir))
        .ok()
        .and_then(|text| serde_json::from_str::<Settings>(&text).ok())
        .filter(|s| s.validate().is_ok())
        .unwrap_or_default()
}

pub fn save(config_dir: &Path, settings: &Settings) -> Result<()> {
    settings.validate()?;
    fs::create_dir_all(config_dir)?;
    let json = serde_json::to_string_pretty(settings).map_err(std::io::Error::other)?;
    let tmp = config_dir.join(format!("{FILE_NAME}.tmp"));
    fs::write(&tmp, json)?;
    fs::rename(tmp, path_in(config_dir))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn corrupt_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(path_in(dir.path()), "{nope").unwrap();
        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn out_of_range_file_gives_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(path_in(dir.path()), r#"{"autolock_minutes": 0}"#).unwrap();
        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn partial_file_fills_in_defaults() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(path_in(dir.path()), r#"{"preview": true}"#).unwrap();
        let s = load(dir.path());
        assert!(s.preview);
        assert_eq!(s.autolock_minutes, 10);
    }

    #[test]
    fn save_round_trips_and_leaves_no_tmp() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings {
            autolock_minutes: 42,
            last_vault_dir: Some("/v".into()),
            preview: true,
        };
        save(dir.path(), &s).unwrap();
        assert_eq!(load(dir.path()), s);
        assert!(!dir.path().join(format!("{FILE_NAME}.tmp")).exists());
    }

    #[test]
    fn save_rejects_invalid_autolock() {
        let dir = tempfile::tempdir().unwrap();
        let s = Settings {
            autolock_minutes: 121,
            ..Settings::default()
        };
        assert!(matches!(save(dir.path(), &s), Err(Error::InvalidInput(_))));
        assert!(!path_in(dir.path()).exists());
    }
}
