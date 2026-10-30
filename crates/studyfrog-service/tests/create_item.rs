#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Creating every kind of item, stacks inside stacks, and the rule that items need a stack.

use std::fs;
use std::path::PathBuf;

use studyfrog_md::doc::{Doc, doc_meta};
use studyfrog_md::parse::parse_doc;
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
        kind,
        title,
        text,
        answer,
        tags: &[],
        parent,
    }
}

fn make(s: &mut VaultSession, n: u8, i: NewItem<'_>) -> Result<(), ServiceError> {
    create_item(s, &i, ids(n)).map(drop)
}

#[test]
fn stacks_nest_and_the_overview_reports_each_ones_parent_in_tree_order() {
    let (d, mut s) = vault("nest");
    make(
        &mut s,
        1,
        item(NewKind::Stack, "French", "All about French", "", None),
    )
    .unwrap();
    make(
        &mut s,
        2,
        item(NewKind::Stack, "Verbs", "", "", Some("french")),
    )
    .unwrap();
    make(
        &mut s,
        3,
        item(NewKind::Stack, "Irregular", "", "", Some("french/verbs")),
    )
    .unwrap();
    let (info, created) =
        create_item(&mut s, &item(NewKind::Stack, "Maths", "", "", None), ids(4)).unwrap();
    assert_eq!(created.path, "maths/_stack.md");
    assert!(d.0.join("french/verbs/irregular/_stack.md").is_file());

    let rows: Vec<(&str, Option<&str>)> = info
        .overview
        .stacks
        .iter()
        .map(|r| (r.folder.as_str(), r.parent.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("french", None),
            ("french/verbs", Some("french")),
            ("french/verbs/irregular", Some("french/verbs")),
            ("maths", None)
        ]
    );
    let french = fs::read_to_string(d.0.join("french/_stack.md")).unwrap();
    assert!(
        french.contains("All about French"),
        "the description is kept: {french}"
    );
}

#[test]
fn items_go_into_a_stack_with_tags_and_count_for_the_nearest_stack() {
    let (d, mut s) = vault("items");
    make(&mut s, 1, item(NewKind::Stack, "French", "", "", None)).unwrap();
    make(
        &mut s,
        2,
        item(NewKind::Stack, "Verbs", "", "", Some("french")),
    )
    .unwrap();
    let tags = vec!["geo".to_owned(), "easy".to_owned()];
    let card = NewItem {
        tags: &tags,
        ..item(NewKind::Card, "", "Bonjour?", "Hello", Some("french/verbs"))
    };
    let (info, c) = create_item(&mut s, &card, ids(3)).unwrap();
    assert_eq!(
        (c.title.as_str(), c.path.as_str()),
        ("Bonjour?", "french/verbs/bonjour.md")
    );
    let (info2, _) = create_item(
        &mut s,
        &item(NewKind::Note, "Pronouns", "je, tu, il", "", Some("french")),
        ids(4),
    )
    .unwrap();
    let (info3, _) = create_item(
        &mut s,
        &item(
            NewKind::Question,
            "",
            "Capital of France?",
            "Paris",
            Some("french"),
        ),
        ids(5),
    )
    .unwrap();
    let _ = (info, info2);
    let by = |f: &str| {
        info3
            .overview
            .stacks
            .iter()
            .find(|r| r.folder == f)
            .map(|r| (r.notes, r.cards, r.questions))
            .unwrap()
    };
    assert_eq!(
        by("french"),
        (1, 0, 1),
        "the note and question belong to the outer stack"
    );
    assert_eq!(
        by("french/verbs"),
        (0, 1, 0),
        "the card belongs to the inner stack"
    );

    let Doc::Card(doc) =
        parse_doc(&fs::read_to_string(d.0.join("french/verbs/bonjour.md")).unwrap()).unwrap()
    else {
        panic!("not a card")
    };
    assert_eq!(
        (doc.front.as_str(), doc.back.as_str()),
        ("Bonjour?", "Hello")
    );
    assert_eq!(doc.meta.tags, ["geo", "easy"]);
    let q =
        parse_doc(&fs::read_to_string(d.0.join("french/capital-of-france.md")).unwrap()).unwrap();
    assert_eq!(
        doc_meta(&q).title,
        "Capital of France?",
        "a question's title comes from its prompt"
    );
}

#[test]
fn the_same_title_twice_gets_a_distinct_file_name_in_the_same_stack() {
    let (d, mut s) = vault("same");
    make(&mut s, 1, item(NewKind::Stack, "French", "", "", None)).unwrap();
    make(
        &mut s,
        2,
        item(NewKind::Note, "Plan", "a", "", Some("french")),
    )
    .unwrap();
    make(
        &mut s,
        3,
        item(NewKind::Note, "Plan", "b", "", Some("french")),
    )
    .unwrap();
    make(&mut s, 4, item(NewKind::Stack, "French", "", "", None)).unwrap();
    assert!(d.0.join("french/plan.md").is_file() && d.0.join("french/plan-2.md").is_file());
    assert!(
        d.0.join("french-2/_stack.md").is_file(),
        "a second top-level stack of that name"
    );
}
