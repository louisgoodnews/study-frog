#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Comparing typed answers with a real item's reference.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::compare::score_item_answer;
use studyfrog_service::session::{ServiceError, open_vault};
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::score::Verdict;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn vault(label: &str) -> Dir {
    let dir =
        std::env::temp_dir().join(format!("studyfrog-compare-{label}-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    let files = [
        (
            "card.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: Capital\n---\nCapital of France?\n???\nParis\n",
                id(10)
            ),
        ),
        (
            "question.md",
            format!(
                "---\nid: {}\nkind: question\ntitle: Cells\nmin_score: 0.6\naccepted:\n  - cells make energy in mitochondria\nrubric:\n  - mitochondria\n---\nWhat makes energy in a cell?\n???\nThe mitochondria\n",
                id(11)
            ),
        ),
        (
            "note.md",
            format!(
                "---\nid: {}\nkind: note\ntitle: Just a note\n---\nbody\n",
                id(12)
            ),
        ),
    ];
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    Dir(dir)
}

#[test]
fn cards_and_questions_are_scored_and_a_rating_is_suggested() {
    let dir = vault("scored");
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();

    let right = score_item_answer(&session, &Lexical, Uid(10), "paris").unwrap();
    assert_eq!(
        (right.score.verdict, right.rating),
        (Verdict::Correct, Rating::Good)
    );
    let wrong = score_item_answer(&session, &Lexical, Uid(10), "Berlin").unwrap();
    assert_eq!(
        (wrong.score.verdict, wrong.rating),
        (Verdict::Incorrect, Rating::Again)
    );

    let q = score_item_answer(&session, &Lexical, Uid(11), "the mitochondria").unwrap();
    assert_eq!(q.score.rubric_hits, [true], "the question's rubric is used");
    assert_eq!(q.score.verdict, Verdict::Correct);
}

#[test]
fn notes_and_missing_items_have_nothing_to_compare_with() {
    let dir = vault("none");
    let (session, _) = open_vault(dir.0.to_str().unwrap()).unwrap();
    let e = score_item_answer(&session, &Lexical, Uid(12), "x").unwrap_err();
    assert!(matches!(e, ServiceError::Invalid(m) if m.contains("cards and questions")));
    assert!(score_item_answer(&session, &Lexical, Uid(99), "x").is_err());
}
