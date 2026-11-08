use rusqlite::Connection;

pub fn cipher_version() -> rusqlite::Result<String> {
    let conn = Connection::open_in_memory()?;
    conn.query_row("PRAGMA cipher_version", [], |row| row.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlcipher_is_linked() {
        let version = cipher_version().expect("pragma cipher_version");
        assert!(!version.is_empty(), "cipher_version must be non-empty");
        assert!(
            version.starts_with('4'),
            "expected SQLCipher 4.x, got {version}"
        );
    }
}
