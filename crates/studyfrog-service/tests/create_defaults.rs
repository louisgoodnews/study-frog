#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! New items start with medium priority and the highest difficulty; stacks start with neither.

use std::fs;

use studyfrog_service::create::{IdSource, NewCard, write_card};
use studyfrog_service::create_more::{NewItem, NewKind, create_item};
use studyfrog_service::session::open_vault;

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    }
}

fn item(kind: NewKind, title: &'static str, parent: Option<&'static str>) -> NewItem<'static> {
    NewItem {
        source: None,
        kind,
        title,
        text: "q",
        answer: "a",
        tags: &[],
        parent,
        priority: None,
        difficulty: None,
        also_in: None,
    }
}

#[test]
fn new_items_start_with_medium_priority_and_the_highest_difficulty_but_stacks_do_not() {
    let dir = std::env::temp_dir().join(format!("studyfrog-defaults-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let (mut s, _) = open_vault(dir.to_str().unwrap()).unwrap();
    let kinds = [
        (NewKind::Stack, "French", None),
        (NewKind::Note, "Note", Some("french")),
        (NewKind::Card, "Card", Some("french")),
        (NewKind::Question, "Question", Some("french")),
    ];
    for (n, (kind, title, parent)) in kinds.into_iter().enumerate() {
        create_item(&mut s, &item(kind, title, parent), ids(n as u8)).unwrap();
    }
    for file in ["note.md", "card.md", "question.md"] {
        let text = fs::read_to_string(dir.join("french").join(file)).unwrap();
        assert!(text.contains("priority: medium"), "{file}: {text}");
        assert!(text.contains("difficulty: difficult"), "{file}: {text}");
    }
    let stack = fs::read_to_string(dir.join("french/_stack.md")).unwrap();
    assert!(
        !stack.contains("difficulty") && !stack.contains("priority"),
        "{stack}"
    );

    let card = NewCard {
        title: "Plain",
        front: "q",
        back: "a",
        tags: &[],
    };
    write_card(&s, "french", &card, ids(9)).unwrap();
    let plain = fs::read_to_string(dir.join("french/plain.md")).unwrap();
    assert!(plain.contains("difficulty: difficult"));
    let _ = fs::remove_dir_all(&dir);
}
