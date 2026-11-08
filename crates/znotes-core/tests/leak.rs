use std::fs;
use std::path::Path;

use znotes_core::{DB_FILE, KdfParams, Vault};

fn files_containing(root: &Path, needle: &[u8]) -> Vec<String> {
    let mut hits = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if fs::read(&path)
                .unwrap()
                .windows(needle.len())
                .any(|w| w == needle)
            {
                hits.push(path.display().to_string());
            }
        }
    }
    hits
}

#[test]
fn no_plaintext_marker_survives_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let marker = format!("ZNOTES_LEAK_{:x}", rand_u64());
    let title = format!("Title {marker}");
    let body = format!("API_KEY=sk-{marker}-secret\n\nMore text about {marker}.");
    {
        let vault = Vault::create(dir.path(), "pw", KdfParams::fast_for_tests()).unwrap();
        vault
            .conn()
            .execute(
                "INSERT INTO notes (title, body, created_at, updated_at) VALUES (?1, ?2, 0, 0)",
                [&title, &body],
            )
            .unwrap();
        let hits: i64 = vault
            .conn()
            .query_row(
                "SELECT count(*) FROM notes_fts WHERE notes_fts MATCH ?1",
                [format!("\"{marker}\"")],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1, "sanity: marker is searchable while unlocked");
    }

    let db_len = fs::metadata(dir.path().join(DB_FILE)).unwrap().len();
    assert!(db_len > 0);
    assert!(!dir.path().join(format!("{DB_FILE}-journal")).exists());
    assert!(!dir.path().join(format!("{DB_FILE}-wal")).exists());

    for needle in [marker.as_str(), "API_KEY", "secret", "More text"] {
        let hits = files_containing(dir.path(), needle.as_bytes());
        assert!(hits.is_empty(), "plaintext {needle:?} found in {hits:?}");
    }
}

fn rand_u64() -> u64 {
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).unwrap();
    u64::from_le_bytes(buf)
}
