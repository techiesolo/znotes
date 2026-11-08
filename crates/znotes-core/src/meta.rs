use std::fs;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};

use crate::kdf::{KdfParams, SALT_LEN};
use crate::{Error, Result};

pub const DB_FILE: &str = "vault.znotes";
pub const META_FILE: &str = "vault.meta.json";
pub const META_TMP_FILE: &str = "vault.meta.json.tmp";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultMeta {
    pub format_version: u32,
    pub kdf: KdfParams,
    pub salt_b64: String,
    pub created_at: i64,
}

impl VaultMeta {
    pub fn new(kdf: KdfParams, salt: &[u8; SALT_LEN], created_at: i64) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            kdf,
            salt_b64: BASE64.encode(salt),
            created_at,
        }
    }

    pub fn salt(&self) -> Result<[u8; SALT_LEN]> {
        let bytes = BASE64
            .decode(&self.salt_b64)
            .map_err(|_| Error::NotAVault("meta salt is not valid base64".into()))?;
        bytes
            .try_into()
            .map_err(|_| Error::NotAVault("meta salt has the wrong length".into()))
    }

    pub fn with_new_salt(&self, salt: &[u8; SALT_LEN]) -> Self {
        Self {
            salt_b64: BASE64.encode(salt),
            ..self.clone()
        }
    }
}

pub fn db_path(dir: &Path) -> PathBuf {
    dir.join(DB_FILE)
}

pub fn meta_path(dir: &Path) -> PathBuf {
    dir.join(META_FILE)
}

pub fn meta_tmp_path(dir: &Path) -> PathBuf {
    dir.join(META_TMP_FILE)
}

pub fn load(dir: &Path) -> Result<VaultMeta> {
    load_file(&meta_path(dir))
}

pub fn load_tmp(dir: &Path) -> Option<VaultMeta> {
    load_file(&meta_tmp_path(dir)).ok()
}

fn load_file(path: &Path) -> Result<VaultMeta> {
    let text = fs::read_to_string(path)
        .map_err(|_| Error::NotAVault(format!("missing {}", path.display())))?;
    let meta: VaultMeta = serde_json::from_str(&text)
        .map_err(|e| Error::NotAVault(format!("unreadable {}: {e}", path.display())))?;
    if meta.format_version > FORMAT_VERSION {
        return Err(Error::UnsupportedVersion {
            found: meta.format_version,
            supported: FORMAT_VERSION,
        });
    }
    meta.salt()?;
    Ok(meta)
}

pub fn write_tmp(dir: &Path, meta: &VaultMeta) -> Result<()> {
    let json = serde_json::to_string_pretty(meta).map_err(std::io::Error::other)?;
    fs::write(meta_tmp_path(dir), json)?;
    Ok(())
}

pub fn promote_tmp(dir: &Path) -> Result<()> {
    fs::rename(meta_tmp_path(dir), meta_path(dir))?;
    Ok(())
}

pub fn save_atomic(dir: &Path, meta: &VaultMeta) -> Result<()> {
    write_tmp(dir, meta)?;
    promote_tmp(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> VaultMeta {
        VaultMeta::new(KdfParams::default(), &[9u8; SALT_LEN], 1_700_000_000)
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = tempfile::tempdir().unwrap();
        save_atomic(dir.path(), &sample()).unwrap();
        assert!(!meta_tmp_path(dir.path()).exists());
        assert_eq!(load(dir.path()).unwrap(), sample());
    }

    #[test]
    fn missing_file_is_not_a_vault() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(load(dir.path()), Err(Error::NotAVault(_))));
    }

    #[test]
    fn corrupt_json_is_not_a_vault() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(meta_path(dir.path()), "{not json").unwrap();
        assert!(matches!(load(dir.path()), Err(Error::NotAVault(_))));
    }

    #[test]
    fn bad_salt_is_not_a_vault() {
        let dir = tempfile::tempdir().unwrap();
        let mut meta = sample();
        meta.salt_b64 = "####".into();
        save_atomic(dir.path(), &meta).unwrap();
        assert!(matches!(load(dir.path()), Err(Error::NotAVault(_))));
    }

    #[test]
    fn newer_format_is_unsupported() {
        let dir = tempfile::tempdir().unwrap();
        let mut meta = sample();
        meta.format_version = FORMAT_VERSION + 1;
        save_atomic(dir.path(), &meta).unwrap();
        assert!(matches!(
            load(dir.path()),
            Err(Error::UnsupportedVersion { found, supported })
                if found == FORMAT_VERSION + 1 && supported == FORMAT_VERSION
        ));
    }
}
