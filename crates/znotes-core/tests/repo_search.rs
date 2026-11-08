use znotes_core::repo::{folders, notes, search};
use znotes_core::{KdfParams, Vault};

fn vault() -> (tempfile::TempDir, Vault) {
    let dir = tempfile::tempdir().unwrap();
    let vault = Vault::create(dir.path(), "pw", KdfParams::fast_for_tests()).unwrap();
    (dir, vault)
}

fn seed(v: &Vault) -> (i64, i64, i64) {
    let c = v.conn();
    let f = folders::create(c, None, "Việt").unwrap();
    let a = notes::create(c, Some(f.id), "Ghi chú tiếng Việt").unwrap();
    notes::update(
        c,
        a.id,
        "Ghi chú tiếng Việt",
        "Hôm nay học về mã hóa dữ liệu.",
    )
    .unwrap();
    let b = notes::create(c, None, "API keys").unwrap();
    notes::update(
        c,
        b.id,
        "API keys",
        "API_KEY=sk-live-12345 belongs to the staging account.",
    )
    .unwrap();
    let c2 = notes::create(c, None, "Groceries").unwrap();
    notes::update(c, c2.id, "Groceries", "milk eggs bread").unwrap();
    (a.id, b.id, c2.id)
}

#[test]
fn diacritics_insensitive_and_prefix() {
    let (_d, v) = vault();
    let (a, _, _) = seed(&v);
    let hits = search::search(v.conn(), "tieng", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, a);
    let hits = search::search(v.conn(), "ma hoa", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert!(
        hits[0].snippet.contains('['),
        "snippet should mark the match: {}",
        hits[0].snippet
    );
    let hits = search::search(v.conn(), "Hô", 10).unwrap();
    assert_eq!(hits.len(), 1);
}

#[test]
fn multiple_tokens_are_anded() {
    let (_d, v) = vault();
    let (_, b, _) = seed(&v);
    let hits = search::search(v.conn(), "staging api", 10).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, b);
    assert!(
        search::search(v.conn(), "staging milk", 10)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn underscore_separated_identifiers_are_found_by_part() {
    let (_d, v) = vault();
    let (_, b, _) = seed(&v);
    let hits = search::search(v.conn(), "API_KEY", 10).unwrap();
    assert_eq!(hits[0].id, b);
    let hits = search::search(v.conn(), "sk-live", 10).unwrap();
    assert_eq!(hits[0].id, b);
}

#[test]
fn odd_input_never_errors() {
    let (_d, v) = vault();
    seed(&v);
    for q in [
        "",
        "  ",
        "\"",
        "a\" OR \"b",
        "*",
        "-",
        "(",
        ":",
        "NOT",
        "AND OR",
        "milk*",
        "^milk",
        "{",
        "}",
    ] {
        let result = search::search(v.conn(), q, 10);
        assert!(result.is_ok(), "query {q:?} errored: {result:?}");
    }
}

#[test]
fn limit_is_respected_and_clamped() {
    let (_d, v) = vault();
    let c = v.conn();
    for i in 0..5 {
        let n = notes::create(c, None, &format!("note {i}")).unwrap();
        notes::update(c, n.id, &format!("note {i}"), "common word").unwrap();
    }
    assert_eq!(search::search(c, "common", 2).unwrap().len(), 2);
    assert_eq!(search::search(c, "common", 0).unwrap().len(), 1);
    assert_eq!(search::search(c, "common", 10_000).unwrap().len(), 5);
}

#[test]
fn title_matches_rank_above_body_matches() {
    let (_d, v) = vault();
    let c = v.conn();
    let body_only = notes::create(c, None, "something").unwrap();
    notes::update(c, body_only.id, "something", "rust rust rust").unwrap();
    let in_title = notes::create(c, None, "rust").unwrap();
    notes::update(c, in_title.id, "rust", "unrelated text").unwrap();
    let hits = search::search(c, "rust", 10).unwrap();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].id, in_title.id);
}

#[test]
fn rebuild_index_keeps_results() {
    let (_d, v) = vault();
    let (a, _, _) = seed(&v);
    search::rebuild_index(v.conn()).unwrap();
    assert_eq!(search::search(v.conn(), "tieng", 10).unwrap()[0].id, a);
}
