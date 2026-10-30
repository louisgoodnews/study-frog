#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Finding items by title across the vault.

mod common;

use common::vault::{card, note, stack, write};
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::migrate;
use studyfrog_store::overview::search_items;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

fn vault() -> (common::TempDir, rusqlite::Connection) {
    let dir = common::temp_dir("search");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(r, "deck/a.md", &card(10, "Alphabet soup", "q", "a"));
    write(r, "deck/b.md", &card(11, "The alpha card", "q", "a"));
    write(r, "deck/c.md", &note(12, "100% sure", ""));
    write(r, "deck/d.md", &note(13, "a_b", ""));
    write(r, "loose.md", &note(14, "ALPHA loose note", ""));
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    reindex(&mut conn, r).unwrap();
    (dir, conn)
}

fn titles(conn: &rusqlite::Connection, q: &str, limit: usize) -> Vec<String> {
    search_items(conn, q, limit)
        .unwrap()
        .into_iter()
        .map(|i| i.title)
        .collect()
}

#[test]
fn matches_anywhere_in_the_title_ignoring_case_with_prefix_matches_first() {
    let (_d, conn) = vault();
    assert_eq!(
        titles(&conn, "alpha", 10),
        ["ALPHA loose note", "Alphabet soup", "The alpha card"]
    );
    assert_eq!(
        titles(&conn, "  ALPHA ", 10).len(),
        3,
        "surrounding spaces and case do not matter"
    );
    assert!(titles(&conn, "zzz", 10).is_empty());
}

#[test]
fn nothing_is_found_for_an_empty_query_and_stacks_are_not_items() {
    let (_d, conn) = vault();
    assert!(titles(&conn, "", 10).is_empty() && titles(&conn, "   ", 10).is_empty());
    assert!(
        titles(&conn, "Deck", 10).is_empty(),
        "the stack itself is not an item"
    );
}

#[test]
fn like_wildcards_in_the_query_are_taken_literally() {
    let (_d, conn) = vault();
    assert_eq!(titles(&conn, "%", 10), ["100% sure"]);
    assert_eq!(
        titles(&conn, "a_b", 10),
        ["a_b"],
        "an underscore is not 'any character'"
    );
    assert!(titles(&conn, "a\\b", 10).is_empty());
}

#[test]
fn the_limit_caps_the_results() {
    let (_d, conn) = vault();
    assert_eq!(titles(&conn, "a", 2).len(), 2);
    let found = search_items(&conn, "soup", 5).unwrap();
    assert_eq!((found.len(), found[0].path.as_str()), (1, "deck/a.md"));
}
