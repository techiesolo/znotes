use rusqlite::{Connection, OptionalExtension};

use crate::{Error, Result};

pub const SCHEMA_VERSION: u32 = 1;

const V1: &str = r#"
CREATE TABLE meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE folders (
    id         INTEGER PRIMARY KEY,
    parent_id  INTEGER REFERENCES folders(id) ON DELETE CASCADE,
    name       TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    UNIQUE (parent_id, name)
);

CREATE TABLE notes (
    id         INTEGER PRIMARY KEY,
    folder_id  INTEGER REFERENCES folders(id) ON DELETE CASCADE,
    title      TEXT NOT NULL DEFAULT 'Untitled',
    body       TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE INDEX notes_folder_updated ON notes (folder_id, updated_at DESC);

CREATE VIRTUAL TABLE notes_fts USING fts5 (
    title,
    body,
    content = 'notes',
    content_rowid = 'id',
    tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TRIGGER notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts (rowid, title, body) VALUES (new.id, new.title, new.body);
END;

CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, body)
        VALUES ('delete', old.id, old.title, old.body);
END;

CREATE TRIGGER notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts (notes_fts, rowid, title, body)
        VALUES ('delete', old.id, old.title, old.body);
    INSERT INTO notes_fts (rowid, title, body) VALUES (new.id, new.title, new.body);
END;
"#;

pub fn migrate(conn: &mut Connection) -> Result<()> {
    let current = stored_version(conn)?;
    if current > SCHEMA_VERSION {
        return Err(Error::UnsupportedVersion {
            found: current,
            supported: SCHEMA_VERSION,
        });
    }
    if current == SCHEMA_VERSION {
        return Ok(());
    }
    let tx = conn.transaction()?;
    if current < 1 {
        tx.execute_batch(V1)?;
    }
    tx.execute(
        "INSERT INTO meta (key, value) VALUES ('schema_version', ?1)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        [SCHEMA_VERSION.to_string()],
    )?;
    tx.commit()?;
    Ok(())
}

pub fn stored_version(conn: &Connection) -> Result<u32> {
    let has_meta: Option<String> = conn
        .query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'meta'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if has_meta.is_none() {
        return Ok(0);
    }
    let version: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    version
        .map(|v| {
            v.parse::<u32>()
                .map_err(|_| Error::NotAVault(format!("bad schema_version {v}")))
        })
        .unwrap_or(Ok(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_fresh_db_to_current_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        assert_eq!(stored_version(&conn).unwrap(), 0);
        migrate(&mut conn).unwrap();
        assert_eq!(stored_version(&conn).unwrap(), SCHEMA_VERSION);
        migrate(&mut conn).unwrap();
        assert_eq!(stored_version(&conn).unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn rejects_newer_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn.execute(
            "UPDATE meta SET value = ?1 WHERE key = 'schema_version'",
            [(SCHEMA_VERSION + 1).to_string()],
        )
        .unwrap();
        assert!(matches!(
            migrate(&mut conn),
            Err(Error::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn fts_follows_notes_through_triggers() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        conn.execute(
            "INSERT INTO notes (id, title, body, created_at, updated_at) VALUES (1, 'Hello', 'tiếng việt', 0, 0)",
            [],
        )
        .unwrap();
        let hits = |q: &str| -> i64 {
            conn.query_row(
                "SELECT count(*) FROM notes_fts WHERE notes_fts MATCH ?1",
                [q],
                |r| r.get(0),
            )
            .unwrap()
        };
        assert_eq!(hits("tieng"), 1);
        conn.execute("UPDATE notes SET body = 'changed' WHERE id = 1", [])
            .unwrap();
        assert_eq!(hits("tieng"), 0);
        assert_eq!(hits("changed"), 1);
        conn.execute("DELETE FROM notes WHERE id = 1", []).unwrap();
        assert_eq!(hits("changed"), 0);
    }
}
