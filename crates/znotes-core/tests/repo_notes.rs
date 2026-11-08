use znotes_core::repo::{folders, notes};
use znotes_core::{Error, KdfParams, Vault};

fn vault() -> (tempfile::TempDir, Vault) {
    let dir = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", KdfParams::fast_for_tests()).unwrap();
    (dir, vault)
}

#[test]
fn create_get_update_roundtrip() {
    let (_d, v) = vault();
    let c = v.conn();
    let note = notes::create(c, None, "First").unwrap();
    assert_eq!(note.body, "");
    let updated = notes::update(c, note.id, "First!", "# body\n\ntext").unwrap();
    assert_eq!(updated.title, "First!");
    assert_eq!(updated.body, "# body\n\ntext");
    assert!(updated.updated_at >= note.updated_at);
    assert_eq!(notes::get(c, note.id).unwrap(), updated);
}

#[test]
fn empty_title_becomes_untitled_and_is_trimmed() {
    let (_d, v) = vault();
    let c = v.conn();
    assert_eq!(notes::create(c, None, "   ").unwrap().title, "Untitled");
    let n = notes::create(c, None, "  Hi  ").unwrap();
    assert_eq!(n.title, "Hi");
    assert_eq!(notes::update(c, n.id, "", "x").unwrap().title, "Untitled");
}

#[test]
fn overlong_title_is_invalid() {
    let (_d, v) = vault();
    let long = "t".repeat(501);
    assert!(matches!(
        notes::create(v.conn(), None, &long),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn large_body_round_trips() {
    let (_d, v) = vault();
    let c = v.conn();
    let big = "tiếng việt 🔐 ".repeat(80_000);
    assert!(big.len() > 1_000_000);
    let n = notes::create(c, None, "big").unwrap();
    notes::update(c, n.id, "big", &big).unwrap();
    assert_eq!(notes::get(c, n.id).unwrap().body, big);
}

#[test]
fn list_filters_by_folder_and_orders_by_updated_desc() {
    let (_d, v) = vault();
    let c = v.conn();
    let f = folders::create(c, None, "F").unwrap();
    let a = notes::create(c, Some(f.id), "a").unwrap();
    let b = notes::create(c, Some(f.id), "b").unwrap();
    let _root = notes::create(c, None, "root").unwrap();
    c.execute("UPDATE notes SET updated_at = 10 WHERE id = ?1", [a.id])
        .unwrap();
    c.execute("UPDATE notes SET updated_at = 20 WHERE id = ?1", [b.id])
        .unwrap();
    let in_f = notes::list(c, Some(f.id)).unwrap();
    assert_eq!(
        in_f.iter().map(|n| n.id).collect::<Vec<_>>(),
        vec![b.id, a.id]
    );
    let at_root = notes::list(c, None).unwrap();
    assert_eq!(at_root.len(), 1);
    assert_eq!(at_root[0].title, "root");
}

#[test]
fn move_between_folders_and_root() {
    let (_d, v) = vault();
    let c = v.conn();
    let f = folders::create(c, None, "F").unwrap();
    let n = notes::create(c, None, "n").unwrap();
    assert_eq!(
        notes::move_to(c, n.id, Some(f.id)).unwrap().folder_id,
        Some(f.id)
    );
    assert_eq!(notes::move_to(c, n.id, None).unwrap().folder_id, None);
    assert!(matches!(
        notes::move_to(c, n.id, Some(999)),
        Err(Error::NotFound {
            entity: "folder",
            ..
        })
    ));
}

#[test]
fn missing_note_is_not_found_everywhere() {
    let (_d, v) = vault();
    let c = v.conn();
    assert!(matches!(
        notes::get(c, 7),
        Err(Error::NotFound {
            entity: "note",
            id: 7
        })
    ));
    assert!(matches!(
        notes::update(c, 7, "t", "b"),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        notes::move_to(c, 7, None),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(notes::delete(c, 7), Err(Error::NotFound { .. })));
    assert!(matches!(
        notes::create(c, Some(7), "t"),
        Err(Error::NotFound {
            entity: "folder",
            ..
        })
    ));
}

#[test]
fn delete_removes_note() {
    let (_d, v) = vault();
    let c = v.conn();
    let n = notes::create(c, None, "bye").unwrap();
    notes::delete(c, n.id).unwrap();
    assert!(matches!(notes::get(c, n.id), Err(Error::NotFound { .. })));
}
