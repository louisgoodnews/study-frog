#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0096, STORY-0097: keys and colours in the index, allocating keys and finding items by key.

mod common;

use common::vault::{card, note, stack, write};
use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::item_keys::{allocate_key, find_by_key, items_without_key};
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::migrate::migrate;
use studyfrog_store::overview::{load_overview, search_items};
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn with(text: &str, kind: &str, lines: &str) -> String {
    text.replace(
        &format!("kind: {kind}\n"),
        &format!("kind: {kind}\n{lines}"),
    )
}

fn vault(dir: &std::path::Path) {
    write(
        dir,
        "a/_stack.md",
        &with(&stack(1, "A"), "stack", "key: STACK_1\n"),
    );
    let colored = "key: FLASHCARD_7\ncolor: \"#4a90e2\"\n";
    write(
        dir,
        "a/c.md",
        &with(&card(10, "Cell", "q", "a"), "card", colored),
    );
    write(
        dir,
        "a/d.md",
        &with(
            &card(11, "Dna", "q", "a"),
            "card",
            "key: flashcard_02\ncolor: sage\n",
        ),
    );
    write(dir, "a/n.md", &note(12, "Loose", "x"));
}

#[test]
fn the_index_keeps_each_items_key_and_colour_in_their_stored_form() {
    let dir = common::temp_dir("keys-index");
    vault(&dir.path);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    let meta = load_item_meta(&conn).unwrap();
    let c = &meta[&Uid(10)];
    assert_eq!(c.key.as_deref(), Some("FLASHCARD_7"));
    assert_eq!(c.color.as_deref(), Some("#4A90E2"));
    assert_eq!(meta[&Uid(11)].key.as_deref(), Some("FLASHCARD_2"));
    assert_eq!(meta[&Uid(11)].color.as_deref(), Some("sage"));
    assert_eq!(meta[&Uid(12)].key, None);
    let stacks = load_overview(&conn).unwrap().stacks;
    assert_eq!(stacks[0].key.as_deref(), Some("STACK_1"));
}

#[test]
fn an_item_is_found_by_its_exact_key_in_any_case() {
    let dir = common::temp_dir("keys-find");
    vault(&dir.path);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    let hit = find_by_key(&conn, "flashcard_7").unwrap().unwrap();
    assert_eq!((hit.uid, hit.kind.as_str()), (Uid(10), "card"));
    assert_eq!(find_by_key(&conn, "STACK_1").unwrap().unwrap().uid, Uid(1));
    assert_eq!(find_by_key(&conn, "NOTE_1").unwrap(), None);
    assert_eq!(find_by_key(&conn, "Cell").unwrap(), None);
    let found = search_items(&conn, "FLASHCARD_2", 10).unwrap();
    assert_eq!(
        found.first().map(|r| r.uid),
        Some(Uid(11)),
        "the navbar search finds a key"
    );
}

#[test]
fn keys_are_handed_out_once_per_kind_and_never_again() {
    let dir = common::temp_dir("keys-allocate");
    vault(&dir.path);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    assert_eq!(allocate_key(&conn, "card").unwrap().unwrap(), "FLASHCARD_8");
    assert_eq!(allocate_key(&conn, "card").unwrap().unwrap(), "FLASHCARD_9");
    assert_eq!(allocate_key(&conn, "note").unwrap().unwrap(), "NOTE_1");
    assert_eq!(allocate_key(&conn, "stack").unwrap().unwrap(), "STACK_2");
    assert_eq!(allocate_key(&conn, "diary").unwrap(), None);
    std::fs::remove_file(dir.path.join("a/c.md")).unwrap();
    reindex(&mut conn, &dir.path).unwrap();
    assert_eq!(
        allocate_key(&conn, "card").unwrap().unwrap(),
        "FLASHCARD_10",
        "a deleted item's key is not handed out again"
    );
}

#[test]
fn the_items_without_a_key_are_listed() {
    let dir = common::temp_dir("keys-missing");
    vault(&dir.path);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    let missing = items_without_key(&conn).unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(
        (missing[0].uid, missing[0].kind.as_str()),
        (Uid(12), "note")
    );
}
