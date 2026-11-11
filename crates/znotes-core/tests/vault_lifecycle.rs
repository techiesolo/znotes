use std::fs;
use std::time::Instant;

use znotes_core::{DB_FILE, Error, KdfParams, META_FILE, Vault};

fn params() -> KdfParams {
    KdfParams::fast_for_tests()
}

#[test]
fn create_writes_db_and_meta() {
    let dir = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", params()).unwrap();
    assert!(dir.path().join(DB_FILE).exists());
    assert!(dir.path().join(META_FILE).exists());
    assert_eq!(vault.dir(), dir.path());
    assert_eq!(vault.kdf_params(), params());
}

#[test]
fn create_twice_is_already_exists() {
    let dir = tempfile::tempdir().unwrap();
    let _first = Vault::create(dir.path(), "pw", params()).unwrap();
    assert!(matches!(
        Vault::create(dir.path(), "pw", params()),
        Err(Error::AlreadyExists(_))
    ));
}

#[test]
fn create_in_missing_nested_dir_works() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("a").join("b");
    Vault::create(&dir, "pw", params()).unwrap();
    assert!(dir.join(DB_FILE).exists());
}

#[test]
fn open_with_correct_password_sees_data() {
    let dir = tempfile::tempdir().unwrap();
    {
        let vault = Vault::create(dir.path(), "pw", params()).unwrap();
        vault
            .conn()
            .execute(
                "INSERT INTO notes (title, body, created_at, updated_at) VALUES ('t', 'b', 0, 0)",
                [],
            )
            .unwrap();
    }
    let vault = Vault::open(dir.path(), "pw").unwrap();
    let count: i64 = vault
        .conn()
        .query_row("SELECT count(*) FROM notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn open_with_wrong_password_fails() {
    let dir = tempfile::tempdir().unwrap();
    drop(Vault::create(dir.path(), "pw", params()).unwrap());
    assert!(matches!(
        Vault::open(dir.path(), "nope"),
        Err(Error::WrongPassword)
    ));
}

#[test]
fn unicode_password_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let pw = "mật khẩu rất dài 🔐";
    drop(Vault::create(dir.path(), pw, params()).unwrap());
    Vault::open(dir.path(), pw).unwrap();
}

#[test]
fn empty_password_is_invalid() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Vault::create(dir.path(), "", params()),
        Err(Error::InvalidInput(_))
    ));
    assert!(!dir.path().join(DB_FILE).exists());
}

#[test]
fn open_without_meta_is_not_a_vault() {
    let dir = tempfile::tempdir().unwrap();
    drop(Vault::create(dir.path(), "pw", params()).unwrap());
    fs::remove_file(dir.path().join(META_FILE)).unwrap();
    assert!(matches!(
        Vault::open(dir.path(), "pw"),
        Err(Error::NotAVault(_))
    ));
}

#[test]
fn open_empty_dir_is_not_a_vault() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Vault::open(dir.path(), "pw"),
        Err(Error::NotAVault(_))
    ));
}

#[test]
fn tampered_salt_is_not_a_vault() {
    let dir = tempfile::tempdir().unwrap();
    drop(Vault::create(dir.path(), "pw", params()).unwrap());
    let meta_path = dir.path().join(META_FILE);
    let text = fs::read_to_string(&meta_path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&text).unwrap();
    json["salt_b64"] = serde_json::Value::String("!!!!".into());
    fs::write(&meta_path, json.to_string()).unwrap();
    assert!(matches!(
        Vault::open(dir.path(), "pw"),
        Err(Error::NotAVault(_))
    ));
}

#[test]
fn change_password_swaps_which_password_opens() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut vault = Vault::create(dir.path(), "old", params()).unwrap();
        vault
            .conn()
            .execute(
                "INSERT INTO notes (title, body, created_at, updated_at) VALUES ('t', 'keep me', 0, 0)",
                [],
            )
            .unwrap();
        vault.change_password("old", "new").unwrap();
    }
    assert!(matches!(
        Vault::open(dir.path(), "old"),
        Err(Error::WrongPassword)
    ));
    let vault = Vault::open(dir.path(), "new").unwrap();
    let body: String = vault
        .conn()
        .query_row("SELECT body FROM notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(body, "keep me");
    assert!(!dir.path().join("vault.meta.json.tmp").exists());
}

#[test]
fn change_password_requires_current_password() {
    let dir = tempfile::tempdir().unwrap();
    let mut vault = Vault::create(dir.path(), "old", params()).unwrap();
    assert!(matches!(
        vault.change_password("wrong", "new"),
        Err(Error::WrongPassword)
    ));
    drop(vault);
    Vault::open(dir.path(), "old").unwrap();
}

#[test]
fn change_password_rejects_empty_new_password() {
    let dir = tempfile::tempdir().unwrap();
    let mut vault = Vault::create(dir.path(), "old", params()).unwrap();
    assert!(matches!(
        vault.change_password("old", ""),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn interrupted_rekey_recovers_from_tmp_meta() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut vault = Vault::create(dir.path(), "old", params()).unwrap();
        let old_meta = fs::read_to_string(dir.path().join(META_FILE)).unwrap();
        vault.change_password("old", "new").unwrap();
        let new_meta = fs::read_to_string(dir.path().join(META_FILE)).unwrap();
        fs::write(dir.path().join("vault.meta.json.tmp"), new_meta).unwrap();
        fs::write(dir.path().join(META_FILE), old_meta).unwrap();
    }
    Vault::open(dir.path(), "new").unwrap();
    assert!(!dir.path().join("vault.meta.json.tmp").exists());
    Vault::open(dir.path(), "new").unwrap();
}

#[test]
fn pragmas_are_applied() {
    let dir = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", params()).unwrap();
    let get = |pragma: &str| -> String {
        vault
            .conn()
            .query_row(&format!("PRAGMA {pragma}"), [], |r| {
                r.get::<_, rusqlite::types::Value>(0)
            })
            .map(|v| match v {
                rusqlite::types::Value::Integer(i) => i.to_string(),
                rusqlite::types::Value::Text(t) => t,
                other => format!("{other:?}"),
            })
            .unwrap()
    };
    assert_eq!(get("cipher_memory_security"), "1");
    assert_eq!(get("foreign_keys"), "1");
    assert_eq!(get("temp_store"), "2");
}

#[test]
fn vault_file_does_not_start_with_sqlite_magic() {
    let dir = tempfile::tempdir().unwrap();
    drop(Vault::create(dir.path(), "pw", params()).unwrap());
    let bytes = fs::read(dir.path().join(DB_FILE)).unwrap();
    assert!(bytes.len() > 16);
    assert_ne!(&bytes[..15], b"SQLite format 3");
}

#[test]
fn default_params_open_time_is_reasonable() {
    let dir = tempfile::tempdir().unwrap();
    drop(Vault::create(dir.path(), "pw", KdfParams::default()).unwrap());
    let started = Instant::now();
    Vault::open(dir.path(), "pw").unwrap();
    let elapsed = started.elapsed();
    println!(
        "default-params open took {elapsed:?} (debug build; see docs/kdf-tuning.md for release numbers)"
    );
    assert!(elapsed.as_secs_f64() < 15.0, "open took {elapsed:?}");
}

#[test]
fn backup_is_an_encrypted_copy_that_opens_with_the_same_password() {
    let dir = tempfile::tempdir().unwrap();
    let dest = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", params()).unwrap();
    vault
        .conn()
        .execute(
            "INSERT INTO notes (title, body, created_at, updated_at) VALUES ('t', 'BACKUP_MARKER_body', 0, 0)",
            [],
        )
        .unwrap();
    let paths = vault.backup_to(dest.path(), "20261108-2200").unwrap();
    assert!(paths.db.ends_with("znotes-backup-20261108-2200.znotes"));
    assert!(
        paths
            .meta
            .ends_with("znotes-backup-20261108-2200.meta.json")
    );
    let bytes = fs::read(&paths.db).unwrap();
    assert!(!bytes.windows(13).any(|w| w == b"BACKUP_MARKER"));

    let restore = tempfile::tempdir().unwrap();
    fs::copy(&paths.db, restore.path().join(DB_FILE)).unwrap();
    fs::copy(&paths.meta, restore.path().join(META_FILE)).unwrap();
    let restored = Vault::open(restore.path(), "pw").unwrap();
    let body: String = restored
        .conn()
        .query_row("SELECT body FROM notes", [], |r| r.get(0))
        .unwrap();
    assert_eq!(body, "BACKUP_MARKER_body");

    assert!(matches!(
        vault.backup_to(dest.path(), "20261108-2200"),
        Err(Error::AlreadyExists(_))
    ));
}
