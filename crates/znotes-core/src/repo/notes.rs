use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use super::{folders, now};
use crate::{Error, Result};

pub const DEFAULT_TITLE: &str = "Untitled";
pub const MAX_TITLE_LEN: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        folder_id: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

pub fn list(conn: &Connection, folder_id: Option<i64>) -> Result<Vec<NoteSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, folder_id, title, updated_at FROM notes
         WHERE folder_id IS ?1 ORDER BY updated_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([folder_id], |row| {
        Ok(NoteSummary {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            title: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn list_all(conn: &Connection) -> Result<Vec<NoteSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, folder_id, title, updated_at FROM notes ORDER BY updated_at DESC, id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(NoteSummary {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            title: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn get(conn: &Connection, id: i64) -> Result<Note> {
    conn.query_row(
        "SELECT id, folder_id, title, body, created_at, updated_at FROM notes WHERE id = ?1",
        [id],
        row_to_note,
    )
    .optional()?
    .ok_or(Error::NotFound { entity: "note", id })
}

pub fn create(conn: &Connection, folder_id: Option<i64>, title: &str) -> Result<Note> {
    ensure_folder_exists(conn, folder_id)?;
    let title = normalize_title(title)?;
    let ts = now();
    conn.execute(
        "INSERT INTO notes (folder_id, title, body, created_at, updated_at) VALUES (?1, ?2, '', ?3, ?3)",
        params![folder_id, title, ts],
    )?;
    get(conn, conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, title: &str, body: &str) -> Result<Note> {
    let title = normalize_title(title)?;
    let affected = conn.execute(
        "UPDATE notes SET title = ?1, body = ?2, updated_at = ?3 WHERE id = ?4",
        params![title, body, now(), id],
    )?;
    if affected == 0 {
        return Err(Error::NotFound { entity: "note", id });
    }
    get(conn, id)
}

pub fn move_to(conn: &Connection, id: i64, folder_id: Option<i64>) -> Result<Note> {
    ensure_folder_exists(conn, folder_id)?;
    let affected = conn.execute(
        "UPDATE notes SET folder_id = ?1, updated_at = ?2 WHERE id = ?3",
        params![folder_id, now(), id],
    )?;
    if affected == 0 {
        return Err(Error::NotFound { entity: "note", id });
    }
    get(conn, id)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(Error::NotFound { entity: "note", id });
    }
    Ok(())
}

fn normalize_title(title: &str) -> Result<String> {
    let trimmed = title.trim();
    if trimmed.chars().count() > MAX_TITLE_LEN {
        return Err(Error::invalid(format!(
            "title must be at most {MAX_TITLE_LEN} characters"
        )));
    }
    Ok(if trimmed.is_empty() {
        DEFAULT_TITLE.to_string()
    } else {
        trimmed.to_string()
    })
}

fn ensure_folder_exists(conn: &Connection, folder_id: Option<i64>) -> Result<()> {
    if let Some(id) = folder_id {
        folders::get(conn, id)?;
    }
    Ok(())
}
