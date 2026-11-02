#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Item status in the index: the stored word, the derived one and the migration.

mod common;

use common::vault::{card, note, stack, write};
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::{migrate, schema_version};
use studyfrog_store::overview::list_items;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schedule::insert_review;
use studyfrog_store::schema::MIGRATIONS;

#[test]
fn a_stored_status_wins_and_the_rest_follow_the_history() {
    let dir = common::temp_dir("status");
    let r = &dir.path;
    write(r, "deck/_stack.md", &stack(1, "Deck"));
    write(r, "deck/a.md", &card(10, "Never reviewed", "q", "a"));
    write(r, "deck/b.md", &card(11, "Reviewed", "q", "a"));
    write(
        r,
        "deck/c.md",
        &card(12, "Done by the user", "q", "a").replace(
            "title: Done by the user\n",
            "title: Done by the user\nstatus: done\n",
        ),
    );
    write(
        r,
        "deck/d.md",
        &card(13, "Reviewed but new", "q", "a").replace(
            "title: Reviewed but new\n",
            "title: Reviewed but new\nstatus: new\n",
        ),
    );
    write(
        r,
        "deck/e.md",
        &note(14, "Odd word", "")
            .replace("title: Odd word\n", "title: Odd word\nstatus: finished\n"),
    );
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    reindex(&mut conn, r).unwrap();
    for n in [11, 13] {
        let review = Review {
            item: Uid(n),
            rating: Rating::Good,
            at: UnixMs(1),
            elapsed_days: 0.0,
        };
        insert_review(&conn, &review).unwrap();
    }
    let status = |title: &str| {
        list_items(&conn, Some("deck"))
            .unwrap()
            .into_iter()
            .find(|i| i.title == title)
            .unwrap()
            .status
    };
    assert_eq!(status("Never reviewed"), Status::New);
    assert_eq!(
        status("Reviewed"),
        Status::Learning,
        "derived from the review"
    );
    assert_eq!(status("Done by the user"), Status::Done);
    assert_eq!(
        status("Reviewed but new"),
        Status::New,
        "a stored status wins over the history"
    );
    assert_eq!(
        status("Odd word"),
        Status::New,
        "an unknown word is ignored and the note was never reviewed"
    );
}

#[test]
fn the_migration_adds_the_column_to_an_existing_index() {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert_eq!(
        schema_version(&conn).unwrap(),
        MIGRATIONS.last().unwrap().version
    );
    let found: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('item') WHERE name = 'status'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(found, 1);
    assert!(conn.execute("INSERT INTO item (uid, path, kind, title, status) VALUES ('x', 'x.md', 'note', 'x', 'bogus')", []).is_err(), "only the five words are accepted");
}
