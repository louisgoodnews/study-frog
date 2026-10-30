#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Bulk card creation for importers and card types.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use studyfrog_core::uid::Uid;
use studyfrog_ext::types::ImportedCard;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::create::IdSource;
use studyfrog_service::import::{MAX_IMPORT, import_cards};
use studyfrog_service::session::{ServiceError, open_vault};

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir = std::env::temp_dir().join(format!("studyfrog-import-{label}-{}", std::process::id()));
    fs::create_dir_all(dir.join("deck")).unwrap();
    let stack = format!(
        "---\nid: {}\nkind: stack\ntitle: Deck\n---\n",
        format_ulid(Uid(1))
    );
    fs::write(dir.join("deck/_stack.md"), stack).unwrap();
    Dir(dir)
}

fn card(front: &str, back: &str) -> ImportedCard {
    ImportedCard {
        front: front.into(),
        back: back.into(),
        ..ImportedCard::default()
    }
}

const IDS: IdSource = IdSource {
    now_ms: 1_800_000_000_000,
    random: [7; 10],
};

#[test]
fn many_cards_are_written_with_distinct_ids_titles_and_tags_and_indexed_once() {
    let d = vault("many");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let mut cards: Vec<ImportedCard> = (0..60)
        .map(|i| card(&format!("Question {i}?"), &format!("Answer {i}")))
        .collect();
    cards[0].tags = vec!["geo".into(), "europe".into()];
    cards[1].title = "Custom title".into();
    let (info, report) = import_cards(&mut s, "deck", &cards, IDS).unwrap();
    assert_eq!((report.created, report.skipped.len()), (60, 0));
    assert_eq!(info.overview.stacks[0].cards, 60);

    let files: Vec<_> = fs::read_dir(d.0.join("deck"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap() != "_stack.md")
        .collect();
    assert_eq!(files.len(), 60, "every card got its own file name");
    let texts: Vec<String> = files
        .iter()
        .map(|p| fs::read_to_string(p).unwrap())
        .collect();
    let ids: HashSet<&str> = texts
        .iter()
        .filter_map(|t| t.lines().find_map(|l| l.strip_prefix("id: ")))
        .collect();
    assert_eq!(ids.len(), 60, "ids are unique");
    assert!(texts.iter().any(|t| t.contains("title: Custom title")));
    let tagged = texts.iter().find(|t| t.contains("Question 0?")).unwrap();
    assert!(
        tagged.contains("geo") && tagged.contains("europe"),
        "{tagged}"
    );
}

#[test]
fn unusable_cards_are_skipped_and_reported_while_the_rest_are_kept() {
    let d = vault("skip");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let cards = [
        card("Fine?", "Yes"),
        card("  ", "no front"),
        card("no back", ""),
        card("Also fine?", "Yes"),
    ];
    let (info, report) = import_cards(&mut s, "deck", &cards, IDS).unwrap();
    assert_eq!(report.created, 2);
    assert_eq!(report.skipped.len(), 2);
    assert!(
        report.skipped[0].starts_with("card 2:") && report.skipped[1].starts_with("card 3:"),
        "{:?}",
        report.skipped
    );
    assert_eq!(info.overview.stacks[0].cards, 2);
}

#[test]
fn an_unknown_stack_or_an_oversized_import_is_refused_before_anything_is_written() {
    let d = vault("refuse");
    let (mut s, _) = open_vault(d.0.to_str().unwrap()).unwrap();
    let (_, report) = import_cards(&mut s, "nowhere", &[card("a?", "b")], IDS).unwrap();
    assert_eq!(
        (report.created, report.skipped.len()),
        (0, 1),
        "reported, not a crash"
    );
    let huge = vec![card("a?", "b"); MAX_IMPORT + 1];
    let e = import_cards(&mut s, "deck", &huge, IDS).unwrap_err();
    assert!(matches!(e, ServiceError::Invalid(m) if m.contains("limited")));
    assert_eq!(
        fs::read_dir(d.0.join("deck")).unwrap().count(),
        1,
        "only _stack.md"
    );
}
