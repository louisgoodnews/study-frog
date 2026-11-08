#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The contrastive drill's items and the distinction kept as hints or a comparison note (STORY-0111).

mod recall_common;

use recall_common::*;
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::bulk_undo;
use studyfrog_service::compare::score_item_answer;
use studyfrog_service::contrast::{ContrastTarget, answer_core, load_contrast, save_contrast};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::Verdict;
use studyfrog_store::overview::linking_to;

fn bonds(label: &str) -> Dir {
    vault(
        label,
        &[
            (
                "bio/ionic.md",
                card(
                    10,
                    "Ionic bond",
                    "What is an ionic bond?",
                    "electrons are transferred",
                ),
            ),
            (
                "bio/covalent.md",
                card(
                    11,
                    "Covalent bond",
                    "What is a covalent bond?",
                    "electrons are shared",
                ),
            ),
            (
                "bio/metal.md",
                card(
                    12,
                    "Metallic bond",
                    "What is a metallic bond?",
                    "a sea of electrons",
                ),
            ),
        ],
    )
}

#[test]
fn the_items_load_in_order_and_the_size_is_checked() {
    let dir = bonds("contrast-load");
    let s = open(&dir);
    let items = load_contrast(&s, &[Uid(11), Uid(10)]).unwrap();
    let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
    assert_eq!(titles, ["Covalent bond", "Ionic bond"]);
    assert_eq!(items[0].answer, "electrons are shared");
    assert!(load_contrast(&s, &[Uid(10)]).is_err());
    assert!(load_contrast(&s, &[Uid(10); 5]).is_err());
}

#[test]
fn a_hint_goes_on_each_item_links_the_others_and_is_replaced_later() {
    let dir = bonds("contrast-hints");
    let mut s = open(&dir);
    let uids = [Uid(10), Uid(11), Uid(12)];
    save_contrast(
        &mut s,
        &uids,
        "who\nowns the electrons",
        ContrastTarget::Hints,
        ids(1_700_000_000_000),
    )
    .unwrap();
    let ionic = read(&dir, "bio/ionic.md");
    assert!(
        ionic.contains("electrons are transferred\n\n> Distinction (vs [[Covalent bond]], [[Metallic bond]]): who owns the electrons"),
        "{ionic}"
    );
    let done = save_contrast(
        &mut s,
        &uids,
        "sharing or giving",
        ContrastTarget::Hints,
        ids(1_700_000_000_100),
    )
    .unwrap();
    let ionic = read(&dir, "bio/ionic.md");
    assert_eq!(
        ionic.matches("> Distinction").count(),
        1,
        "replaced: {ionic}"
    );
    assert!(ionic.contains("sharing or giving"));
    let scored = score_item_answer(&s, &Lexical, Uid(10), "electrons are transferred").unwrap();
    assert_eq!(
        scored.score.verdict,
        Verdict::Correct,
        "the hint is not part of the answer"
    );
    assert_eq!(answer_core("x\n\n> Distinction (vs [[Y]]): z"), "x");
    bulk_undo(&mut s, &done.undo).unwrap();
    assert!(
        read(&dir, "bio/ionic.md").contains("who owns the electrons"),
        "undo goes back one save"
    );
}

#[test]
fn a_comparison_note_links_every_item_and_is_updated_not_duplicated() {
    let dir = bonds("contrast-note");
    let mut s = open(&dir);
    let uids = [Uid(10), Uid(11)];
    let first = save_contrast(
        &mut s,
        &uids,
        "transfer versus sharing",
        ContrastTarget::Note,
        ids(1_700_000_000_000),
    )
    .unwrap();
    assert_eq!(
        first.written,
        ["bio/comparison-ionic-bond-vs-covalent-bond.md"]
    );
    let note = read(&dir, &first.written[0]);
    assert!(
        note.contains("title: 'Comparison: Ionic bond vs Covalent bond'")
            || note.contains("Comparison: Ionic bond vs Covalent bond"),
        "{note}"
    );
    assert!(note.contains("compares: "), "{note}");
    assert!(
        note.contains("## [[Ionic bond]]") && note.contains("## [[Covalent bond]]"),
        "{note}"
    );
    assert_eq!(
        linking_to(&s.conn, "Ionic bond", Uid(10)).unwrap().len(),
        1,
        "the card sees the note"
    );
    let before = files_in(&dir, "bio").len();
    let again = save_contrast(
        &mut s,
        &[Uid(11), Uid(10)],
        "who keeps the electrons",
        ContrastTarget::Note,
        ids(1_700_000_000_500),
    )
    .unwrap();
    assert_eq!(again.written, first.written, "the same note, in any order");
    assert_eq!(files_in(&dir, "bio").len(), before);
    assert!(read(&dir, &first.written[0]).contains("who keeps the electrons"));
    bulk_undo(&mut s, &first.undo).unwrap();
    assert_eq!(
        files_in(&dir, "bio").len(),
        before - 1,
        "undo removes the new note"
    );
}

#[test]
fn an_empty_distinction_is_refused() {
    let dir = bonds("contrast-empty");
    let mut s = open(&dir);
    assert!(
        save_contrast(
            &mut s,
            &[Uid(10), Uid(11)],
            "  ",
            ContrastTarget::Hints,
            ids(1)
        )
        .is_err()
    );
}
