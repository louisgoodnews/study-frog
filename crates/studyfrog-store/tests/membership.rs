#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0211: the membership of stacks in the index, the vault problems and the upgrade of an existing index.

mod common;

use common::vault::{card, note, stack, write};
use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::db::open_memory;
use studyfrog_store::migrate::{migrate, schema_version};
use studyfrog_store::overview::load_overview;
use studyfrog_store::refs_index::{members_of, reference_problems, stacks_of};
use studyfrog_store::reindex::reindex;
use studyfrog_store::schema::MIGRATIONS;

fn db() -> Connection {
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    conn
}

fn id(n: u128) -> String {
    format_ulid(Uid(n))
}

/// A card that sits in `a/` and is referenced by the given stacks.
fn referencing(n: u128, by: &[u128]) -> String {
    let list: Vec<String> = by.iter().map(|b| id(*b)).collect();
    card(n, "C", "q", "a").replace(
        "kind: card\n",
        &format!("kind: card\nreferenced_by: [{}]\n", list.join(", ")),
    )
}

fn vault(dir: &std::path::Path, by: &[u128]) {
    write(dir, "a/_stack.md", &stack(1, "A"));
    write(dir, "b/_stack.md", &stack(2, "B"));
    write(dir, "a/c.md", &referencing(10, by));
    write(dir, "b/d.md", &card(11, "D", "q", "a"));
    write(dir, "loose.md", &note(12, "Loose", "x"));
}

#[test]
fn an_item_belongs_to_its_folders_stack_and_to_every_stack_that_references_it() {
    let dir = common::temp_dir("members");
    vault(&dir.path, &[2]);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    let a = members_of(&conn, &id(1)).unwrap();
    let b = members_of(&conn, &id(2)).unwrap();
    assert_eq!(a, [(id(10), true)]);
    assert_eq!(b, [(id(10), false), (id(11), true)]);
    assert_eq!(
        stacks_of(&conn, &id(10)).unwrap(),
        [(id(1), true), (id(2), false)]
    );
    assert!(
        stacks_of(&conn, &id(12)).unwrap().is_empty(),
        "a loose item is in the Inbox, in no stack"
    );
    assert!(reference_problems(&conn).unwrap().is_empty());
}

#[test]
fn editing_the_references_updates_the_membership_incrementally() {
    let dir = common::temp_dir("members-edit");
    vault(&dir.path, &[2]);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    write(&dir.path, "a/c.md", &referencing(10, &[]));
    let later = std::time::SystemTime::now() + std::time::Duration::from_secs(10);
    std::fs::File::options()
        .write(true)
        .open(dir.path.join("a/c.md"))
        .unwrap()
        .set_modified(later)
        .unwrap();
    reindex(&mut conn, &dir.path).unwrap();
    assert_eq!(stacks_of(&conn, &id(10)).unwrap(), [(id(1), true)]);
}

#[test]
fn a_missing_stack_the_own_stack_and_a_repeat_are_vault_problems_with_clear_messages() {
    let dir = common::temp_dir("members-bad");
    vault(&dir.path, &[999, 1, 2, 2]);
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    let problems = reference_problems(&conn).unwrap();
    let text: Vec<&str> = problems.iter().map(|(_, m)| m.as_str()).collect();
    assert!(
        text.iter()
            .any(|m| m.contains("does not exist") && m.contains(&id(999))),
        "{text:?}"
    );
    assert!(
        text.iter()
            .any(|m| m.contains("its own stack") && m.contains('a')),
        "{text:?}"
    );
    assert!(
        text.iter()
            .any(|m| m.contains("more than once") && m.contains(&id(2))),
        "{text:?}"
    );
    assert!(problems.iter().all(|(p, _)| p == "a/c.md"));
    let shown = load_overview(&conn).unwrap().problems;
    assert_eq!(shown.len(), 3, "the vault overview lists them");
    assert_eq!(
        members_of(&conn, &id(1)).unwrap(),
        [(id(10), true)],
        "a wrong reference adds nothing"
    );
}

#[test]
fn a_vault_without_references_has_the_same_membership_as_its_folders() {
    let dir = common::temp_dir("members-plain");
    write(&dir.path, "a/_stack.md", &stack(1, "A"));
    write(&dir.path, "a/sub/x.md", &card(5, "X", "q", "a"));
    let mut conn = db();
    reindex(&mut conn, &dir.path).unwrap();
    assert_eq!(
        members_of(&conn, &id(1)).unwrap(),
        [(id(5), true)],
        "a normal folder belongs to the stack above"
    );
    assert!(reference_problems(&conn).unwrap().is_empty());
}

#[test]
fn an_index_from_before_gets_the_table_and_reads_every_file_again() {
    let dir = common::temp_dir("members-upgrade");
    vault(&dir.path, &[2]);
    let mut conn = open_memory().unwrap();
    migrate(&mut conn, &MIGRATIONS[..19]).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), 19);
    conn.execute(
        "INSERT INTO file (path, size, mtime_ns, hash, uid) VALUES ('a/c.md', 10, 5, 7, ?1)",
        [id(10)],
    )
    .unwrap();
    migrate(&mut conn, MIGRATIONS).unwrap();
    assert_eq!(schema_version(&conn).unwrap(), 20);
    let (mtime, hash): (i64, i64) = conn
        .query_row("SELECT mtime_ns, hash FROM file", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((mtime, hash), (-1, 0), "marked to be read again");
    conn.execute("DELETE FROM file", []).unwrap();
    reindex(&mut conn, &dir.path).unwrap();
    assert_eq!(
        stacks_of(&conn, &id(10)).unwrap(),
        [(id(1), true), (id(2), false)]
    );
}
