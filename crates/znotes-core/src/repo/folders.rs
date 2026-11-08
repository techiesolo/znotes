use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::now;
use crate::{Error, Result};

pub const MAX_NAME_LEN: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

const COLUMNS: &str = "id, parent_id, name, created_at, updated_at";

fn row_to_folder(row: &rusqlite::Row<'_>) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: row.get(0)?,
        parent_id: row.get(1)?,
        name: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

pub fn list(conn: &Connection) -> Result<Vec<Folder>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM folders ORDER BY parent_id IS NOT NULL, parent_id, name COLLATE NOCASE"
    ))?;
    let rows = stmt.query_map([], row_to_folder)?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, id: i64) -> Result<Folder> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM folders WHERE id = ?1"),
        [id],
        row_to_folder,
    )
    .optional()?
    .ok_or(Error::NotFound {
        entity: "folder",
        id,
    })
}

pub fn create(conn: &Connection, parent_id: Option<i64>, name: &str) -> Result<Folder> {
    let name = normalize_name(name)?;
    if let Some(parent) = parent_id {
        get(conn, parent)?;
    }
    ensure_name_free(conn, parent_id, &name, None)?;
    let ts = now();
    conn.execute(
        "INSERT INTO folders (parent_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        params![parent_id, name, ts],
    )
    .map_err(unique_violation_as_invalid)?;
    get(conn, conn.last_insert_rowid())
}

pub fn rename(conn: &Connection, id: i64, name: &str) -> Result<Folder> {
    let name = normalize_name(name)?;
    let current = get(conn, id)?;
    ensure_name_free(conn, current.parent_id, &name, Some(id))?;
    conn.execute(
        "UPDATE folders SET name = ?1, updated_at = ?2 WHERE id = ?3",
        params![name, now(), id],
    )
    .map_err(unique_violation_as_invalid)?;
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM folders WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(Error::NotFound {
            entity: "folder",
            id,
        });
    }
    Ok(())
}

fn normalize_name(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::invalid("folder name must not be empty"));
    }
    if trimmed.chars().count() > MAX_NAME_LEN {
        return Err(Error::invalid(format!(
            "folder name must be at most {MAX_NAME_LEN} characters"
        )));
    }
    Ok(trimmed.to_string())
}

fn ensure_name_free(
    conn: &Connection,
    parent_id: Option<i64>,
    name: &str,
    except_id: Option<i64>,
) -> Result<()> {
    let taken: Option<i64> = conn
        .query_row(
            "SELECT id FROM folders WHERE parent_id IS ?1 AND name = ?2 AND id IS NOT ?3",
            params![parent_id, name, except_id],
            |row| row.get(0),
        )
        .optional()?;
    if taken.is_some() {
        return Err(Error::invalid(format!(
            "a folder named {name:?} already exists here"
        )));
    }
    Ok(())
}

fn unique_violation_as_invalid(err: rusqlite::Error) -> Error {
    match err {
        rusqlite::Error::SqliteFailure(inner, _)
            if inner.code == rusqlite::ffi::ErrorCode::ConstraintViolation =>
        {
            Error::invalid("a folder with that name already exists here")
        }
        other => Error::Db(other),
    }
}
