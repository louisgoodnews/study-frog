#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Priority and difficulty from the files, with inheritance through nested stacks.

use std::fs;

use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::session::open_vault;
use studyfrog_store::overview::load_indexed_items;

fn write(dir: &std::path::Path, rel: &str, n: u128, kind: &str, extra: &str) {
    let body = if kind == "card" { "q\n???\na\n" } else { "" };
    let text = format!(
        "---\nid: {}\nkind: {kind}\ntitle: T{n}\n{extra}---\n{body}",
        format_ulid(Uid(n))
    );
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

#[test]
fn items_take_their_own_values_else_the_nearest_stack_that_states_one_else_the_defaults() {
    let dir = std::env::temp_dir().join(format!("studyfrog-priority-{}", std::process::id()));
    write(
        &dir,
        "a/_stack.md",
        1,
        "stack",
        "priority: high\ndifficulty: difficult\n",
    );
    write(&dir, "a/b/_stack.md", 2, "stack", "priority: low\n");
    write(
        &dir,
        "a/own.md",
        10,
        "card",
        "priority: highest\ndifficulty: easy\n",
    );
    write(&dir, "a/plain.md", 11, "card", "");
    write(&dir, "a/b/deep.md", 12, "card", "");
    write(&dir, "c/_stack.md", 3, "stack", "");
    write(&dir, "c/none.md", 13, "card", "priority: shouting\n");
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let items = load_indexed_items(&session.conn).unwrap();
    let of = |n: u128| {
        items
            .iter()
            .find(|i| i.uid == Uid(n))
            .map(|i| (i.priority, i.difficulty))
            .unwrap()
    };
    assert_eq!(
        of(10),
        (Priority::Highest, Some(Difficulty::Easy)),
        "its own words win"
    );
    assert_eq!(
        of(11),
        (Priority::High, Some(Difficulty::Difficult)),
        "the stack's"
    );
    assert_eq!(
        of(12),
        (Priority::Low, Some(Difficulty::Difficult)),
        "priority from b, difficulty from a"
    );
    assert_eq!(
        of(13),
        (Priority::Medium, None),
        "an unknown word is ignored"
    );
    let _ = fs::remove_dir_all(&dir);
}
