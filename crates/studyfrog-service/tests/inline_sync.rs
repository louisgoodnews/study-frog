#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Syncing the inline cards of a note into its stack (STORY-0110).

mod recall_common;

use recall_common::*;
use studyfrog_core::uid::Uid;
use studyfrog_service::compare::score_item_answer;
use studyfrog_service::inline_sync::{
    SyncChange, apply_inline_sync, count_of, plan_inline_sync, without_source,
};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::Verdict;
use studyfrog_store::overview::linking_to;

const BODY: &str = "# Cells\n- Mitosis :: cell division\n```\nskip :: me\n```\nQ: What splits?\nA: The cell\n## Why\n??: Why divide?\nA: To grow.\nMitosis :: duplicate front is synced once\n";

fn lecture(label: &str) -> Dir {
    vault(label, &[("bio/lecture.md", note(5, "Lecture", BODY))])
}

fn files_are_linked(dir: &Dir) {
    let card = read(dir, "bio/mitosis.md");
    assert!(
        card.contains(&format!("extracted_from: {}", id(5))),
        "{card}"
    );
    assert!(card.contains("extract_key: "), "{card}");
    assert!(
        card.contains("tags:\n- cells"),
        "inherits the note's tags: {card}"
    );
    assert!(
        card.contains("cell division\n\nSource: [[Lecture#Cells]]"),
        "{card}"
    );
    let question = read(dir, "bio/why-divide.md");
    assert!(question.contains("kind: question"), "{question}");
    assert!(question.contains("Source: [[Lecture#Why]]"), "{question}");
}

#[test]
fn a_first_sync_creates_linked_items_and_a_second_changes_nothing() {
    let dir = lecture("sync-first");
    let mut s = open(&dir);
    let plan = plan_inline_sync(&s, Uid(5), None).unwrap();
    assert_eq!(plan.entries.len(), 3, "the repeated front counts once");
    assert_eq!(count_of(&plan, SyncChange::New), 3);
    assert_eq!(plan.folder, "bio");
    let done = apply_inline_sync(&mut s, &plan, ids(1_700_000_000_000)).unwrap();
    assert_eq!((done.created, done.updated), (3, 0));
    let files = files_in(&dir, "bio");
    assert_eq!(files.len(), 5, "{files:?}");
    files_are_linked(&dir);
    let back = linking_to(&s.conn, "Lecture", Uid(5)).unwrap();
    assert_eq!(
        back.len(),
        3,
        "the note sees every item that links back to it"
    );
    let again = plan_inline_sync(&s, Uid(5), None).unwrap();
    assert_eq!(count_of(&again, SyncChange::Unchanged), 3);
    let none = apply_inline_sync(&mut s, &again, ids(1_700_000_000_100)).unwrap();
    assert_eq!((none.created, none.updated), (0, 0));
    assert_eq!(files_in(&dir, "bio").len(), 5);
}

#[test]
fn a_changed_definition_updates_the_same_item_and_keeps_its_id() {
    let dir = lecture("sync-change");
    let mut s = open(&dir);
    let plan = plan_inline_sync(&s, Uid(5), None).unwrap();
    apply_inline_sync(&mut s, &plan, ids(1_700_000_000_000)).unwrap();
    let before = read(&dir, "bio/mitosis.md");
    let edited = BODY.replace("cell division", "division of the nucleus");
    let plan = plan_inline_sync(&s, Uid(5), Some(&edited)).unwrap();
    let changes: Vec<SyncChange> = plan.entries.iter().map(|e| e.change).collect();
    assert_eq!(
        changes,
        [
            SyncChange::Updated,
            SyncChange::Unchanged,
            SyncChange::Unchanged
        ]
    );
    let done = apply_inline_sync(&mut s, &plan, ids(1_700_000_000_500)).unwrap();
    assert_eq!((done.created, done.updated), (0, 1));
    let after = read(&dir, "bio/mitosis.md");
    assert!(after.contains("division of the nucleus\n\nSource: [[Lecture#Cells]]"));
    let id_line = |t: &str| t.lines().find(|l| l.starts_with("id:")).unwrap().to_owned();
    assert_eq!(
        id_line(&before),
        id_line(&after),
        "same item, same schedule"
    );
    assert_eq!(files_in(&dir, "bio").len(), 5, "no duplicate");
    studyfrog_service::bulk::bulk_undo(&mut s, &done.undo).unwrap();
    assert_eq!(
        read(&dir, "bio/mitosis.md"),
        before,
        "undo restores the old answer"
    );
}

#[test]
fn the_source_line_is_not_part_of_the_answer_compared() {
    assert_eq!(without_source("Paris\n\nSource: [[Geo#Capitals]]"), "Paris");
    assert_eq!(
        without_source("Paris\nSource: not a link"),
        "Paris\nSource: not a link"
    );
    let dir = lecture("sync-source");
    let mut s = open(&dir);
    let plan = plan_inline_sync(&s, Uid(5), None).unwrap();
    apply_inline_sync(&mut s, &plan, ids(1_700_000_000_000)).unwrap();
    let uid = linking_to(&s.conn, "Lecture", Uid(5))
        .unwrap()
        .into_iter()
        .find(|r| r.title == "Mitosis")
        .unwrap()
        .uid;
    let scored = score_item_answer(&s, &Lexical, uid, "cell division").unwrap();
    assert_eq!(scored.score.verdict, Verdict::Correct);
}

#[test]
fn a_note_outside_a_stack_or_another_kind_is_refused() {
    let dir = vault(
        "sync-refused",
        &[
            ("loose.md", note(6, "Loose", "A :: b\n")),
            ("bio/card.md", card(7, "Card", "x", "y")),
        ],
    );
    let s = open(&dir);
    assert!(plan_inline_sync(&s, Uid(6), None).is_err());
    assert!(plan_inline_sync(&s, Uid(7), None).is_err());
}
