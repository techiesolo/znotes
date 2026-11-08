use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags, ffi};
use zeroize::Zeroizing;

use crate::kdf::{self, KEY_LEN, KdfParams};
use crate::meta::{self, VaultMeta};
use crate::schema;
use crate::{Error, Result};

pub struct Vault {
    conn: Connection,
    dir: PathBuf,
    meta: VaultMeta,
}

impl fmt::Debug for Vault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vault").field("dir", &self.dir).finish()
    }
}

impl Vault {
    pub fn create(dir: &Path, password: &str, params: KdfParams) -> Result<Vault> {
        validate_password(password)?;
        let db_path = meta::db_path(dir);
        if db_path.exists() || meta::meta_path(dir).exists() {
            return Err(Error::AlreadyExists(dir.display().to_string()));
        }
        fs::create_dir_all(dir)?;
        let salt = kdf::random_salt()?;
        let vault_meta = VaultMeta::new(params, &salt, now());
        let key = kdf::derive(password.as_bytes(), &salt, &params)?;
        let created = (|| -> Result<Connection> {
            let mut conn = open_connection(&db_path, &key, true)?;
            schema::migrate(&mut conn)?;
            Ok(conn)
        })();
        let conn = match created {
            Ok(conn) => conn,
            Err(e) => {
                let _ = fs::remove_file(&db_path);
                return Err(e);
            }
        };
        meta::save_atomic(dir, &vault_meta)?;
        Ok(Vault {
            conn,
            dir: dir.to_path_buf(),
            meta: vault_meta,
        })
    }

    pub fn open(dir: &Path, password: &str) -> Result<Vault> {
        validate_password(password)?;
        let db_path = meta::db_path(dir);
        if !db_path.exists() {
            return Err(Error::NotAVault(format!("missing {}", db_path.display())));
        }
        let vault_meta = meta::load(dir)?;
        match open_with_meta(&db_path, &vault_meta, password) {
            Ok(conn) => Ok(Vault {
                conn,
                dir: dir.to_path_buf(),
                meta: vault_meta,
            }),
            Err(Error::WrongPassword) => recover_from_interrupted_rekey(dir, &db_path, password),
            Err(e) => Err(e),
        }
    }

    pub fn change_password(&mut self, current: &str, new: &str) -> Result<()> {
        validate_password(new)?;
        let db_path = meta::db_path(&self.dir);
        drop(open_with_meta(&db_path, &self.meta, current)?);
        let salt = kdf::random_salt()?;
        let new_meta = self.meta.with_new_salt(&salt);
        let key = kdf::derive(new.as_bytes(), &salt, &new_meta.kdf)?;
        meta::write_tmp(&self.dir, &new_meta)?;
        let hex = kdf::to_hex_key(&key);
        self.conn
            .execute_batch(&format!("PRAGMA rekey = \"x'{}'\";", *hex))?;
        meta::promote_tmp(&self.dir)?;
        self.meta = new_meta;
        Ok(())
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn kdf_params(&self) -> KdfParams {
        self.meta.kdf
    }
}

fn recover_from_interrupted_rekey(dir: &Path, db_path: &Path, password: &str) -> Result<Vault> {
    let Some(tmp_meta) = meta::load_tmp(dir) else {
        return Err(Error::WrongPassword);
    };
    let conn = open_with_meta(db_path, &tmp_meta, password)?;
    meta::promote_tmp(dir)?;
    Ok(Vault {
        conn,
        dir: dir.to_path_buf(),
        meta: tmp_meta,
    })
}

fn open_with_meta(db_path: &Path, vault_meta: &VaultMeta, password: &str) -> Result<Connection> {
    let key = kdf::derive(password.as_bytes(), &vault_meta.salt()?, &vault_meta.kdf)?;
    let mut conn = open_connection(db_path, &key, false)?;
    schema::migrate(&mut conn)?;
    Ok(conn)
}

fn open_connection(
    db_path: &Path,
    key: &Zeroizing<[u8; KEY_LEN]>,
    create: bool,
) -> Result<Connection> {
    let mut flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    if create {
        flags |= OpenFlags::SQLITE_OPEN_CREATE;
    }
    let conn = Connection::open_with_flags(db_path, flags)?;
    let hex = kdf::to_hex_key(key);
    debug_assert!(hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()));
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", *hex))?;
    verify_key(&conn)?;
    conn.execute_batch(
        "PRAGMA cipher_memory_security = ON;
         PRAGMA temp_store = MEMORY;
         PRAGMA foreign_keys = ON;
         PRAGMA synchronous = FULL;",
    )?;
    Ok(conn)
}

fn verify_key(conn: &Connection) -> Result<()> {
    conn.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
        row.get::<_, i64>(0)
    })
    .map(|_| ())
    .map_err(wrong_key_as_wrong_password)
}

fn wrong_key_as_wrong_password(err: rusqlite::Error) -> Error {
    match err {
        rusqlite::Error::SqliteFailure(inner, _) if inner.code == ffi::ErrorCode::NotADatabase => {
            Error::WrongPassword
        }
        other => Error::Db(other),
    }
}

fn validate_password(password: &str) -> Result<()> {
    if password.is_empty() {
        return Err(Error::invalid("password must not be empty"));
    }
    Ok(())
}

pub(crate) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
