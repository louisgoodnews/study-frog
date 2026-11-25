#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Grading a choice answer under each scheme: the score, the verdict, the suggested rating and the
//! engine text the run card shows (STORY-0231).

use std::fs;
use std::path::PathBuf;

use studyfrog_core::choice_grading::ChoiceGrading;
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::choice::{CHOICE_ENGINE, choice_rule_text, grade_choice_answer};
use studyfrog_service::session::{VaultSession, open_vault};
use studyfrog_slm::score::Verdict;

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// A vault with one single-choice card (correct: Mitochondrion) and one multiple-select card
/// (correct: Whale and Bat).
fn vault(label: &str) -> (Dir, VaultSession) {
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-choice-answer-{label}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    let files = [
        (
            "single.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: ATP\n---\nWhich organelle makes ATP?\n- [ ] Ribosome\n- [x] Mitochondrion\n- [ ] Golgi apparatus\n???\nMitochondrion\n",
                id(20)
            ),
        ),
        (
            "multiple.md",
            format!(
                "---\nid: {}\nkind: card\ntitle: Mammals\n---\nWhich are mammals?\n- [x] Whale\n- [ ] Shark\n- [x] Bat\n???\nWhale and bat\n",
                id(21)
            ),
        ),
    ];
    for (name, text) in files {
        fs::write(dir.join(name), text).unwrap();
    }
    let (session, _) = open_vault(dir.to_str().unwrap()).unwrap();
    (Dir(dir), session)
}

const SINGLE: Uid = Uid(20);
const MULTIPLE: Uid = Uid(21);

#[test]
fn single_choice_correct_is_full_credit_under_every_scheme() {
    let (_dir, session) = vault("single-correct");
    for scheme in [
        ChoiceGrading::AllOrNothing,
        ChoiceGrading::Zapechelnyuk,
        ChoiceGrading::CountPartial,
    ] {
        let out = grade_choice_answer(&session, SINGLE, &[false, true, false], scheme).unwrap();
        assert_eq!(out.scored.score.value, 1.0);
        assert_eq!(out.scored.score.verdict, Verdict::Correct);
        assert_eq!(out.scored.rating, Rating::Good);
    }
}

#[test]
fn single_choice_wrong_or_nothing_is_no_credit() {
    let (_dir, session) = vault("single-wrong");
    for picked in [[true, false, false], [false, false, false]] {
        let out =
            grade_choice_answer(&session, SINGLE, &picked, ChoiceGrading::Zapechelnyuk).unwrap();
        assert_eq!(out.scored.score.value, 0.0);
        assert_eq!(out.scored.score.verdict, Verdict::Incorrect);
        assert_eq!(out.scored.rating, Rating::Again);
    }
}

#[test]
fn multiple_select_partial_credit_suggests_hard() {
    let (_dir, session) = vault("multiple-partial");
    // Whale and Shark chosen: one correct of two, one wrong.
    let picked = [true, true, false];
    let out =
        grade_choice_answer(&session, MULTIPLE, &picked, ChoiceGrading::CountPartial).unwrap();
    assert_eq!(out.scored.score.value, 0.0, "one correct minus one wrong");
    assert_eq!(out.scored.rating, Rating::Again);

    // Whale only: one of two correct, nothing wrong.
    let out = grade_choice_answer(
        &session,
        MULTIPLE,
        &[true, false, false],
        ChoiceGrading::CountPartial,
    )
    .unwrap();
    assert_eq!(out.scored.score.value, 0.5);
    assert_eq!(out.scored.score.verdict, Verdict::Partial);
    assert_eq!(out.scored.rating, Rating::Hard);
    assert_eq!(out.scored.engine, "choice (count-based partial rule)");
}

#[test]
fn multiple_select_all_or_nothing_is_exact() {
    let (_dir, session) = vault("multiple-aon");
    let full = grade_choice_answer(
        &session,
        MULTIPLE,
        &[true, false, true],
        ChoiceGrading::AllOrNothing,
    )
    .unwrap();
    assert_eq!(full.scored.score.value, 1.0);
    assert_eq!(full.scored.rating, Rating::Good);

    let missing = grade_choice_answer(
        &session,
        MULTIPLE,
        &[true, false, false],
        ChoiceGrading::AllOrNothing,
    )
    .unwrap();
    assert_eq!(missing.scored.score.value, 0.0);
    assert_eq!(missing.scored.rating, Rating::Again);
}

#[test]
fn zapechelnyuk_credits_one_over_the_number_chosen() {
    let (_dir, session) = vault("zapechelnyuk");
    // Whale and Bat and Shark chosen: k = 3, two correct among them: 1/3.
    let out = grade_choice_answer(
        &session,
        MULTIPLE,
        &[true, true, true],
        ChoiceGrading::Zapechelnyuk,
    )
    .unwrap();
    assert!((out.scored.score.value - 1.0 / 3.0).abs() < 1e-6);
    assert_eq!(out.scored.score.verdict, Verdict::Partial);
    assert_eq!(out.scored.rating, Rating::Hard);
    assert_eq!(out.scored.engine, "choice (Zapechelnyuk rule)");
    assert!(!out.grade.correct);
}

#[test]
fn engine_text_names_the_rule_for_choice_scores_only() {
    assert_eq!(CHOICE_ENGINE, "choice");
    assert_eq!(
        choice_rule_text("choice (Zapechelnyuk rule)"),
        Some("Zapechelnyuk rule")
    );
    assert_eq!(choice_rule_text("words"), None);
    assert_eq!(choice_rule_text("choice"), None);
}
