#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The rule that every item except a stack needs a stack, and the other refusals.

use std::fs;
use std::path::PathBuf;

use studyfrog_service::create::IdSource;
use studyfrog_service::create_more::{NewItem, NewKind, create_item};
use studyfrog_service::session::{ServiceError, VaultSession, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!("studyfrog-item-{label}-{}", std::process::id()));
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

fn item(
    kind: NewKind,
    title: &'static str,
    text: &'static str,
    answer: &'static str,
    parent: Option<&'static str>,
) -> NewItem<'static> {
    NewItem {
        source: None,
        kind,
        title,
        text,
        answer,
        tags: &[],
        parent,
        priority: None,
        difficulty: None,
        also_in: None,
    }
}

fn make(s: &mut VaultSession, n: u8, i: NewItem<'_>) -> Result<(), ServiceError> {
    create_item(s, &i, ids(n)).map(drop)
}

#[test]
fn an_item_without_a_stack_or_in_a_folder_that_is_no_stack_is_refused() {
    let (d, mut s) = vault("refuse");
    make(&mut s, 1, item(NewKind::Stack, "French", "", "", None)).unwrap();
    fs::create_dir_all(d.0.join("plain")).unwrap();
    let why = |r: Result<(), ServiceError>| match r {
        Err(ServiceError::Invalid(m)) => m,
        other => panic!("expected a refusal: {other:?}"),
    };
    assert!(
        why(make(&mut s, 2, item(NewKind::Card, "", "Q?", "A", None))).contains("choose the stack")
    );
    // BUG-0019: notes without a parent stack are permitted at the vault root
    assert!(
        make(
            &mut s,
            3,
            item(NewKind::Note, "Loose Note", "content", "", None)
        )
        .is_ok()
    );
    assert!(
        why(make(
            &mut s,
            4,
            item(NewKind::Note, "N", "", "", Some("plain"))
        ))
        .contains("not a stack")
    );
    assert!(
        why(make(
            &mut s,
            5,
            item(NewKind::Stack, "S", "", "", Some("plain"))
        ))
        .contains("not a stack")
    );
    assert!(
        why(make(
            &mut s,
            6,
            item(NewKind::Stack, "S", "", "", Some("nowhere"))
        ))
        .contains("not a stack")
    );
    assert!(
        why(make(
            &mut s,
            7,
            item(NewKind::Card, "", "only front", "", Some("french"))
        ))
        .contains("front and a back")
    );
    assert!(
        why(make(
            &mut s,
            8,
            item(NewKind::Question, "", "prompt", "", Some("french"))
        ))
        .contains("prompt and a reference answer")
    );
    assert!(
        why(make(
            &mut s,
            9,
            item(NewKind::Note, "  ", "body", "", Some("french"))
        ))
        .contains("title")
    );
    assert!(
        why(make(
            &mut s,
            10,
            item(NewKind::Stack, "two\nlines", "", "", None)
        ))
        .contains("single line")
    );
}

#[test]
fn loose_notes_at_vault_root_are_indexed_and_created_at_root() {
    let (d, mut s) = vault("loose-note");
    let (_, created) = create_item(
        &mut s,
        &item(NewKind::Note, "Standalone Note", "My body", "", None),
        ids(1),
    )
    .unwrap();
    assert_eq!(created.path, "standalone-note.md");
    assert!(d.0.join("standalone-note.md").exists());
}
