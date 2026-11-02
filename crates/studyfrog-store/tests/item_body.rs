#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The text of each item is indexed, and marking every file brings it back for items indexed before it existed.

mod common;

use common::vault::{card, note, stack, write};
use studyfrog_store::db::open_memory;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

fn bodies(conn: &rusqlite::Connection) -> Vec<String> {
    let mut all: Vec<String> = load_item_meta(conn)
        .unwrap()
        .into_values()
        .map(|m| m.body)
        .collect();
    all.sort();
    all
}

#[test]
fn the_text_of_cards_and_notes_is_indexed_and_a_marked_file_is_read_again() {
    let dir = common::temp_dir("item-body");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(r, "deck/a.md", &card(10, "Alpha", "the front", "the back"));
    write(r, "deck/n.md", &note(11, "Note", "a body of text"));
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    reindex(&mut conn, r).unwrap();
    assert_eq!(bodies(&conn), ["a body of text", "the front\nthe back"]);

    // what the migration does to an index made before the text was kept
    conn.execute("UPDATE item SET body = NULL", []).unwrap();
    assert_eq!(bodies(&conn), ["", ""]);
    let unchanged = reindex(&mut conn, r).unwrap();
    assert_eq!(unchanged.indexed, 0, "unchanged files are not read again");
    conn.execute("UPDATE file SET mtime_ns = -1, hash = 0", [])
        .unwrap();
    let again = reindex(&mut conn, r).unwrap();
    assert_eq!(again.indexed, 3, "every file is read and indexed again");
    assert_eq!(bodies(&conn), ["a body of text", "the front\nthe back"]);
}
