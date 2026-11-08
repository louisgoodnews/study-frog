#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0096: new items get keys, older items get theirs only when asked, keys survive a rename and a
//! restart, and assigning keys can be undone.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::time::UnixMs;
use studyfrog_md::extra::extra_value;
use studyfrog_md::parse::parse_doc;
use studyfrog_service::bulk::bulk_undo;
use studyfrog_service::create::{IdSource, create_stack};
use studyfrog_service::create_more::{NewItem, NewKind, create_item};
use studyfrog_service::item_keys::{assign_missing_keys, keyless_items};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_store::item_keys::find_by_key;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-keys-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let (s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), s)
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    }
}

fn new(kind: NewKind, title: &'static str) -> NewItem<'static> {
    NewItem {
        source: None,
        kind,
        title,
        text: "front",
        answer: "back",
        tags: &[],
        parent: Some("bio"),
        priority: None,
        difficulty: None,
        also_in: None,
    }
}

fn key_in(dir: &std::path::Path, rel: &str) -> Option<String> {
    let doc = parse_doc(&fs::read_to_string(dir.join(rel)).unwrap()).unwrap();
    extra_value(&doc, "key")
}

#[test]
fn every_new_stack_and_item_gets_the_next_key_of_its_kind() {
    let (dir, mut s) = vault("new");
    create_stack(&mut s, "Bio", ids(1)).unwrap();
    assert_eq!(key_in(&dir.0, "bio/_stack.md").as_deref(), Some("STACK_1"));
    let made = [
        (NewKind::Card, "Cell"),
        (NewKind::Card, "Dna"),
        (NewKind::Note, "Notes"),
        (NewKind::Question, "Why"),
    ];
    let mut paths = Vec::new();
    for (n, (kind, title)) in made.into_iter().enumerate() {
        let (_, created) = create_item(&mut s, &new(kind, title), ids(10 + n as u8)).unwrap();
        paths.push(created.path);
    }
    let keys: Vec<_> = paths.iter().map(|p| key_in(&dir.0, p).unwrap()).collect();
    assert_eq!(keys, ["FLASHCARD_1", "FLASHCARD_2", "NOTE_1", "QUESTION_1"]);
    let (_, sub) = create_item(&mut s, &new(NewKind::Stack, "Cells"), ids(30)).unwrap();
    assert_eq!(key_in(&dir.0, &sub.path).as_deref(), Some("STACK_2"));
}

#[test]
fn a_key_survives_a_rename_and_a_restart() {
    let (dir, mut s) = vault("stable");
    create_stack(&mut s, "Bio", ids(1)).unwrap();
    let (_, c) = create_item(&mut s, &new(NewKind::Card, "Cell"), ids(2)).unwrap();
    fs::rename(dir.0.join(&c.path), dir.0.join("bio/renamed.md")).unwrap();
    drop(s);
    let (s, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let hit = find_by_key(&s.conn, "FLASHCARD_1").unwrap().unwrap();
    assert_eq!(hit.path, "bio/renamed.md");
    assert_eq!(Some(hit.uid), c.uid);
}

#[test]
fn older_items_get_keys_only_when_asked_and_it_can_be_undone() {
    let (dir, mut s) = vault("assign");
    fs::create_dir_all(dir.0.join("old")).unwrap();
    let stack = "---\nid: 01JBYQ00W9Y7A19B1X00000001\nkind: stack\ntitle: Old\n---\n";
    let note = "---\nid: 01JBYQ00W9Y7A19B1X00000002\nkind: note\ntitle: N\n---\nbody\n";
    let card = "---\nid: 01JBYQ00W9Y7A19B1X00000003\nkind: card\ntitle: C\nkey: FLASHCARD_4\n---\nq\n???\na\n";
    fs::write(dir.0.join("old/_stack.md"), stack).unwrap();
    fs::write(dir.0.join("old/n.md"), note).unwrap();
    fs::write(dir.0.join("old/c.md"), card).unwrap();
    studyfrog_service::create::refresh(&mut s).unwrap();
    assert_eq!(
        fs::read_to_string(dir.0.join("old/n.md")).unwrap(),
        note,
        "opening never rewrites"
    );
    assert_eq!(keyless_items(&s).unwrap().len(), 2);
    let done = assign_missing_keys(&mut s, UnixMs(1_900_000_000_000)).unwrap();
    assert_eq!((done.changed, done.skipped.len()), (2, 0));
    assert_eq!(key_in(&dir.0, "old/_stack.md").as_deref(), Some("STACK_1"));
    assert_eq!(key_in(&dir.0, "old/n.md").as_deref(), Some("NOTE_1"));
    assert_eq!(key_in(&dir.0, "old/c.md").as_deref(), Some("FLASHCARD_4"));
    let text = fs::read_to_string(dir.0.join("old/n.md")).unwrap();
    assert!(text.contains("body"), "{text}");
    assert!(keyless_items(&s).unwrap().is_empty());
    bulk_undo(&mut s, &done.undo).unwrap();
    assert_eq!(fs::read_to_string(dir.0.join("old/n.md")).unwrap(), note);
}
