#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Creating stacks and cards through the service.

use std::fs;
use std::path::PathBuf;

use studyfrog_service::create::{
    IdSource, create_card, create_stack, default_card_title, fresh_id_source, slug,
};
use studyfrog_service::session::{ServiceError, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn empty_vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-create-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    Dir(dir)
}

fn ids(n: u8) -> IdSource {
    IdSource {
        now_ms: 1_800_000_000_000 + u64::from(n),
        random: [n; 10],
    }
}

#[test]
fn slugs_are_file_name_friendly() {
    assert_eq!(slug("French: Verbs & Tenses!"), "french-verbs-tenses");
    assert_eq!(slug("  Ünï cödé  "), "ünï-cödé");
    assert_eq!(slug("???"), "");
    assert_eq!(slug(&"a".repeat(100)).len(), 60);
    assert_eq!(
        default_card_title("\n\n  What is 2+2?\nmore"),
        "What is 2+2?"
    );
}

#[test]
fn a_new_stack_and_card_appear_in_the_vault_and_are_valid_markdown() {
    let d = empty_vault("flow");
    let (mut session, info) = open_vault(d.0.to_str().unwrap()).unwrap();
    assert!(info.overview.stacks.is_empty());

    let info = create_stack(&mut session, "French Verbs", ids(1)).unwrap();
    assert_eq!(info.overview.stacks.len(), 1);
    assert_eq!(
        (
            info.overview.stacks[0].title.as_str(),
            info.overview.stacks[0].folder.as_str()
        ),
        ("French Verbs", "french-verbs")
    );
    assert!(d.0.join("french-verbs/_stack.md").is_file());

    let info = create_card(
        &mut session,
        "french-verbs",
        "",
        "Parler?\n\nTo speak",
        "to speak",
        ids(2),
    )
    .unwrap();
    assert_eq!(info.overview.stacks[0].cards, 1);
    let text = fs::read_to_string(d.0.join("french-verbs/parler.md")).unwrap();
    assert!(
        text.starts_with("---\nid: ")
            && text.contains("kind: card")
            && text.contains("\n???\nto speak"),
        "{text}"
    );
    assert!(
        info.overview.problems.is_empty(),
        "everything created parses back"
    );
    let again = create_card(&mut session, "french-verbs", "Parler?", "Q2", "A2", ids(3)).unwrap();
    assert_eq!(again.overview.stacks[0].cards, 2);
    assert!(
        d.0.join("french-verbs/parler-2.md").is_file(),
        "a clash gets a numbered name, nothing is overwritten"
    );
}

#[test]
fn names_never_overwrite_and_bad_input_is_refused_with_a_reason() {
    let d = empty_vault("refuse");
    let (mut session, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    create_stack(&mut session, "Maths", ids(1)).unwrap();
    create_stack(&mut session, "Maths", ids(2)).unwrap();
    assert!(d.0.join("maths/_stack.md").is_file() && d.0.join("maths-2/_stack.md").is_file());
    let uid_of = |p: &str| {
        fs::read_to_string(d.0.join(p))
            .unwrap()
            .lines()
            .nth(1)
            .unwrap()
            .to_owned()
    };
    assert_ne!(
        uid_of("maths/_stack.md"),
        uid_of("maths-2/_stack.md"),
        "every stack has its own id"
    );

    let reason = |r: Result<_, ServiceError>| match r {
        Err(ServiceError::Invalid(m)) => m,
        other => panic!("expected a refusal, got {other:?}"),
    };
    assert!(reason(create_stack(&mut session, "   ", ids(3))).contains("title"));
    assert!(reason(create_stack(&mut session, "two\nlines", ids(3))).contains("single line"));
    assert!(
        reason(create_card(
            &mut session,
            "maths",
            "t",
            "front",
            "  ",
            ids(4)
        ))
        .contains("front and a back")
    );
    assert!(
        reason(create_card(&mut session, "nowhere", "t", "f", "b", ids(4))).contains("not a stack")
    );
    assert!(
        reason(create_card(
            &mut session,
            "maths",
            "t",
            "a\n???\nb",
            "c",
            ids(4)
        ))
        .contains("???")
    );
    assert_eq!(
        fs::read_dir(d.0.join("maths")).unwrap().count(),
        1,
        "refused requests wrote nothing"
    );
}

#[test]
fn fresh_ids_are_random_and_valid() {
    let (a, b) = (fresh_id_source(1).unwrap(), fresh_id_source(1).unwrap());
    assert_ne!(a.random, b.random);
}

#[test]
fn bug_0015_long_card_titles_and_escaped_newlines_are_preserved_and_repaired() {
    let q = "Was sind die vier Hauptziele einer akkuraten psychologischen Diagnostik?";
    assert_eq!(default_card_title(q), q);
    let d = empty_vault("bug0015");
    fs::create_dir_all(d.0.join("diag")).unwrap();
    fs::write(
        d.0.join("diag/_stack.md"),
        "---\nid: 01000000000000000000000001\nkind: stack\ntitle: Diag\n---\n",
    )
    .unwrap();
    let truncated: String = q.chars().take(60).collect();
    fs::write(
        d.0.join("diag/card.md"),
        format!(
            "---\nid: 01000000000000000000000002\nkind: card\ntitle: {truncated}\n---\n{q}\n???\n1. Eins\\n2. Zwei\n"
        ),
    )
    .unwrap();
    let (s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let uid = studyfrog_md::ulid::parse_ulid("01000000000000000000000002").unwrap();
    let content = studyfrog_service::content::load_item_content(&s, uid).unwrap();
    assert_eq!(content.title, q);
    assert_eq!(content.answer, "1. Eins\n2. Zwei");
}
