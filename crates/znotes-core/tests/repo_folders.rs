use znotes_core::repo::{folders, notes};
use znotes_core::{Error, KdfParams, Vault};

fn vault() -> (tempfile::TempDir, Vault) {
    let dir = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", KdfParams::fast_for_tests()).unwrap();
    (dir, vault)
}

#[test]
fn create_list_nested() {
    let (_d, v) = vault();
    let c = v.conn();
    let work = folders::create(c, None, "Work").unwrap();
    let sub = folders::create(c, Some(work.id), "Projects").unwrap();
    assert_eq!(sub.parent_id, Some(work.id));
    let all = folders::list(c).unwrap();
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].name, "Work");
    assert_eq!(all[1].name, "Projects");
}

#[test]
fn names_are_trimmed_and_validated() {
    let (_d, v) = vault();
    let c = v.conn();
    assert_eq!(folders::create(c, None, "  Inbox ").unwrap().name, "Inbox");
    assert!(matches!(
        folders::create(c, None, "   "),
        Err(Error::InvalidInput(_))
    ));
    let long = "x".repeat(201);
    assert!(matches!(
        folders::create(c, None, &long),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn duplicate_names_in_same_parent_are_rejected_including_root() {
    let (_d, v) = vault();
    let c = v.conn();
    folders::create(c, None, "Dup").unwrap();
    assert!(matches!(
        folders::create(c, None, "Dup"),
        Err(Error::InvalidInput(_))
    ));
    let other = folders::create(c, None, "Other").unwrap();
    folders::create(c, Some(other.id), "Dup").unwrap();
}

#[test]
fn rename_checks_collision_and_existence() {
    let (_d, v) = vault();
    let c = v.conn();
    let a = folders::create(c, None, "A").unwrap();
    let b = folders::create(c, None, "B").unwrap();
    assert!(matches!(
        folders::rename(c, b.id, "A"),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(folders::rename(c, b.id, "B").unwrap().name, "B");
    assert_eq!(folders::rename(c, a.id, "C").unwrap().name, "C");
    assert!(matches!(
        folders::rename(c, 999, "Z"),
        Err(Error::NotFound {
            entity: "folder",
            id: 999
        })
    ));
}

#[test]
fn create_under_missing_parent_is_not_found() {
    let (_d, v) = vault();
    assert!(matches!(
        folders::create(v.conn(), Some(42), "X"),
        Err(Error::NotFound {
            entity: "folder",
            id: 42
        })
    ));
}

#[test]
fn delete_cascades_to_children_and_notes() {
    let (_d, v) = vault();
    let c = v.conn();
    let root = folders::create(c, None, "Root").unwrap();
    let child = folders::create(c, Some(root.id), "Child").unwrap();
    let note = notes::create(c, Some(child.id), "In child").unwrap();
    let loose = notes::create(c, None, "Loose").unwrap();
    folders::delete(c, root.id).unwrap();
    assert!(folders::list(c).unwrap().is_empty());
    assert!(matches!(
        notes::get(c, note.id),
        Err(Error::NotFound { .. })
    ));
    assert_eq!(notes::get(c, loose.id).unwrap().title, "Loose");
    assert!(matches!(
        folders::delete(c, root.id),
        Err(Error::NotFound { .. })
    ));
}
