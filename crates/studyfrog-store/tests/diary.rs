#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Diary index, scanner, and query tests.

mod common;

use common::vault::write;
use studyfrog_core::civil::Date;
use studyfrog_core::diary::RatingValue;
use studyfrog_core::uid::Uid;
use studyfrog_store::db::open_memory;
use studyfrog_store::diary::{
    diary_entries_for_item, diary_entries_for_run, diary_entries_for_stack,
    list_diary_entries_by_day, load_diary_entry, search_diary_entries,
};
use studyfrog_store::migrate::migrate;
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

const DIARY_ENTRY_1: &str = r#"---
id: 01K2V9B7M1N3P5R7T9W0XY1234
kind: diary
date: 2026-11-04
title: Evening Biochemistry Rehearsal
study_minutes: 35
tags: [biochem, enzymes]
performance: 4
focus: 5
caffeine: 2
runs: [01K2V8X3Y4Z5A6B7C8D9E01234]
stacks: [biochem/metabolism]
items: [01K2P4Q5R6S7T8A9V0W1X2Y3Z4]
---
Derived the Lineweaver-Burk equation tonight. It finally clicked!
"#;

const DIARY_ENTRY_2: &str = r#"---
id: 01K2V9C8N2M4Q6S8A0W1YZ2345
kind: diary
date: 2026-11-05
title: Morning Anatomy Session
study_minutes: 50
tags: [anatomy, bones]
performance: 3
focus: 3
runs: [01K2V8X3Y4Z5A6B7C8D9E01234]
stacks: [medicine/anatomy]
items: [01K2P4Q5R6S7T8A9V0W1X2Y3Z4, 01K2P4Q5R6S7T8A9V0W1X2Y3Z9]
---
Reviewed skull foramina and cranial nerves before breakfast.
"#;

fn setup_vault() -> (common::TempDir, rusqlite::Connection) {
    let dir = common::temp_dir("diary");
    let r = &dir.path;

    write(r, "diary/2026-11-04.md", DIARY_ENTRY_1);
    write(r, "diary/2026-11-05.md", DIARY_ENTRY_2);

    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    let report = reindex(&mut conn, r).unwrap();
    assert_eq!(report.indexed, 2);
    (dir, conn)
}

#[test]
fn list_diary_entries_by_day_returns_exact_matches() {
    let (_dir, conn) = setup_vault();

    let nov_4 = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    let entries = list_diary_entries_by_day(&conn, nov_4).unwrap();
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.title, "Evening Biochemistry Rehearsal");
    assert_eq!(entry.study_minutes, Some(35));
    assert_eq!(entry.date, nov_4);
    assert_eq!(entry.ratings.get("performance"), Some(&RatingValue::Int(4)));
    assert_eq!(entry.ratings.get("focus"), Some(&RatingValue::Int(5)));
    assert_eq!(entry.ratings.get("caffeine"), Some(&RatingValue::Int(2)));
    assert_eq!(entry.tags, ["biochem", "enzymes"]);
    assert_eq!(entry.stacks, ["biochem/metabolism"]);
    assert_eq!(
        entry.runs,
        [studyfrog_md::ulid::parse_ulid("01K2V8X3Y4Z5A6B7C8D9E01234").unwrap()]
    );
    assert_eq!(
        entry.items,
        [studyfrog_md::ulid::parse_ulid("01K2P4Q5R6S7T8A9V0W1X2Y3Z4").unwrap()]
    );

    let nov_6 = Date {
        year: 2026,
        month: 11,
        day: 6,
    };
    let empty = list_diary_entries_by_day(&conn, nov_6).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn search_diary_entries_matches_title_and_content() {
    let (_dir, conn) = setup_vault();

    // Match in title
    let res1 = search_diary_entries(&conn, "Biochemistry").unwrap();
    assert_eq!(res1.len(), 1);
    assert_eq!(res1[0].title, "Evening Biochemistry Rehearsal");

    // Match in content
    let res2 = search_diary_entries(&conn, "Lineweaver-Burk").unwrap();
    assert_eq!(res2.len(), 1);
    assert_eq!(res2[0].title, "Evening Biochemistry Rehearsal");

    // Match in content of second entry
    let res3 = search_diary_entries(&conn, "foramina").unwrap();
    assert_eq!(res3.len(), 1);
    assert_eq!(res3[0].title, "Morning Anatomy Session");

    // Empty search query
    assert!(search_diary_entries(&conn, "   ").unwrap().is_empty());

    // Non-matching query
    assert!(search_diary_entries(&conn, "quantum").unwrap().is_empty());
}

#[test]
fn diary_link_queries_find_referenced_entities() {
    let (_dir, conn) = setup_vault();
    let shared_item = studyfrog_md::ulid::parse_ulid("01K2P4Q5R6S7T8A9V0W1X2Y3Z4").unwrap();
    let second_item = studyfrog_md::ulid::parse_ulid("01K2P4Q5R6S7T8A9V0W1X2Y3Z9").unwrap();
    let unlinked_item = Uid(999_999);

    // Both entries link shared_item
    let for_shared = diary_entries_for_item(&conn, shared_item).unwrap();
    assert_eq!(for_shared.len(), 2);

    // Only second entry links second_item
    let for_second = diary_entries_for_item(&conn, second_item).unwrap();
    assert_eq!(for_second.len(), 1);
    assert_eq!(for_second[0].title, "Morning Anatomy Session");

    // Unlinked item returns empty
    let for_unlinked = diary_entries_for_item(&conn, unlinked_item).unwrap();
    assert!(for_unlinked.is_empty());

    // Query by stack
    let by_stack = diary_entries_for_stack(&conn, "biochem/metabolism").unwrap();
    assert_eq!(by_stack.len(), 1);
    assert_eq!(by_stack[0].title, "Evening Biochemistry Rehearsal");

    // Query by run
    let run_uid = studyfrog_md::ulid::parse_ulid("01K2V8X3Y4Z5A6B7C8D9E01234").unwrap();
    let by_run = diary_entries_for_run(&conn, run_uid).unwrap();
    assert_eq!(by_run.len(), 2);
}

#[test]
fn reindex_updates_and_deletes_diary_entries() {
    let (dir, mut conn) = setup_vault();
    let r = &dir.path;

    let uid1 = studyfrog_md::ulid::parse_ulid("01K2V9B7M1N3P5R7T9W0XY1234").unwrap();
    let loaded = load_diary_entry(&conn, uid1)
        .unwrap()
        .expect("loaded entry 1");
    assert_eq!(loaded.study_minutes, Some(35));

    // Update entry 1 file
    let updated_entry = DIARY_ENTRY_1.replace("study_minutes: 35", "study_minutes: 45");
    write(r, "diary/2026-11-04.md", &updated_entry);

    let report = reindex(&mut conn, r).unwrap();
    assert_eq!(report.indexed, 1);

    let reloaded = load_diary_entry(&conn, uid1)
        .unwrap()
        .expect("reloaded entry 1");
    assert_eq!(reloaded.study_minutes, Some(45));

    // Delete entry 2 file
    std::fs::remove_file(r.join("diary/2026-11-05.md")).unwrap();
    let del_report = reindex(&mut conn, r).unwrap();
    assert_eq!(del_report.removed, 1);

    let uid2 = studyfrog_md::ulid::parse_ulid("01K2V9C8N2M4Q6S8A0W1YZ2345").unwrap();
    assert!(load_diary_entry(&conn, uid2).unwrap().is_none());
}
