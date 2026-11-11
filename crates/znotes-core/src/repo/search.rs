use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::Result;

pub const MAX_LIMIT: u32 = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub title: String,
    pub snippet: String,
}

pub fn build_fts_query(raw: &str) -> Option<String> {
    let terms: Vec<String> = raw
        .split_whitespace()
        .map(|token| token.replace('"', ""))
        .filter(|token| !token.is_empty())
        .map(|token| format!("\"{token}\"*"))
        .collect();
    if terms.is_empty() {
        None
    } else {
        Some(terms.join(" "))
    }
}

pub fn search(conn: &Connection, raw: &str, limit: u32) -> Result<Vec<SearchHit>> {
    let Some(query) = build_fts_query(raw) else {
        return Ok(Vec::new());
    };
    let limit = limit.clamp(1, MAX_LIMIT);
    let mut stmt = conn.prepare(
        "SELECT n.id, n.folder_id, n.title, snippet(notes_fts, 1, char(2), char(3), '…', 12)
         FROM notes_fts
         JOIN notes n ON n.id = notes_fts.rowid
         WHERE notes_fts MATCH ?1
         ORDER BY bm25(notes_fts, 10.0, 1.0)
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![query, limit], |row| {
        Ok(SearchHit {
            id: row.get(0)?,
            folder_id: row.get(1)?,
            title: row.get(2)?,
            snippet: row.get(3)?,
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

pub fn rebuild_index(conn: &Connection) -> Result<()> {
    conn.execute("INSERT INTO notes_fts(notes_fts) VALUES ('rebuild')", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_and_whitespace_queries_are_none() {
        assert_eq!(build_fts_query(""), None);
        assert_eq!(build_fts_query("   \t"), None);
        assert_eq!(build_fts_query("\"\""), None);
    }

    #[test]
    fn tokens_are_quoted_prefix_terms() {
        assert_eq!(
            build_fts_query("hello world").as_deref(),
            Some("\"hello\"* \"world\"*")
        );
    }

    #[test]
    fn quotes_are_stripped_so_user_input_cannot_change_syntax() {
        assert_eq!(
            build_fts_query("a\" OR \"b").as_deref(),
            Some("\"a\"* \"OR\"* \"b\"*")
        );
    }
}
