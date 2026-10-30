#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The scoring contract, checked on the lexical runtime and on a second, hand-made runtime.

use std::collections::HashMap;

use proptest::prelude::*;
use studyfrog_core::review::Rating;
use studyfrog_slm::lexical::Lexical;
use studyfrog_slm::rating::suggested_rating;
use studyfrog_slm::runtime::{SlmError, SlmRuntime};
use studyfrog_slm::score::{Comparison, Score, Verdict, score_answer, verdict_for};

fn cmp<'a>(reference: &'a str, answer: &'a str) -> Comparison<'a> {
    Comparison {
        reference,
        answer,
        accepted: &[],
        rubric: &[],
        min_score: 0.7,
    }
}

fn lexical(reference: &str, answer: &str) -> Score {
    score_answer(&Lexical, &cmp(reference, answer)).unwrap()
}

/// A runtime with fixed vectors per text: proves scoring does not care which runtime it gets.
struct Table(HashMap<&'static str, Vec<f32>>);

impl SlmRuntime for Table {
    fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, SlmError> {
        texts
            .iter()
            .map(|t| {
                self.0
                    .get(t)
                    .cloned()
                    .ok_or_else(|| SlmError::Failed(format!("unknown text {t}")))
            })
            .collect()
    }
}

#[test]
fn a_second_runtime_drives_the_same_scoring_logic() {
    let t = Table(HashMap::from([
        ("paris", vec![1.0, 0.0]),
        ("the french capital", vec![0.8, 0.6]),
        ("lyon", vec![0.0, 1.0]),
    ]));
    let near = score_answer(&t, &cmp("paris", "the french capital")).unwrap();
    assert!((near.value - 0.8).abs() < 1e-6);
    assert_eq!(near.verdict, Verdict::Correct);
    let far = score_answer(&t, &cmp("paris", "lyon")).unwrap();
    assert_eq!((far.value, far.verdict), (0.0, Verdict::Incorrect));
    let err = score_answer(&t, &cmp("paris", "rome")).unwrap_err();
    assert!(
        err.to_string().contains("rome"),
        "a runtime failure reaches the caller"
    );
}

#[test]
fn accepted_answers_and_rubric_points_change_the_score() {
    let reference = "photosynthesis converts light into chemical energy";
    let answer = "plants make sugar from carbon dioxide";
    let alone = lexical(reference, answer).value;
    let accepted = ["plants make sugar from carbon dioxide and water".to_owned()];
    let with_accepted = score_answer(
        &Lexical,
        &Comparison {
            accepted: &accepted,
            ..cmp(reference, answer)
        },
    )
    .unwrap();
    assert!(
        with_accepted.value > alone,
        "a closer accepted answer raises the score"
    );

    let rubric = [
        "carbon dioxide".to_owned(),
        "chlorophyll absorbs light".to_owned(),
    ];
    let s = score_answer(
        &Lexical,
        &Comparison {
            rubric: &rubric,
            ..cmp(reference, answer)
        },
    )
    .unwrap();
    assert_eq!(s.rubric_hits, [true, false], "one of two points is covered");
}

#[test]
fn nothing_typed_scores_zero_and_asks_nothing_of_the_runtime() {
    struct Never;
    impl SlmRuntime for Never {
        fn embed(&self, _: &[&str]) -> Result<Vec<Vec<f32>>, SlmError> {
            Err(SlmError::Unavailable("must not be called".into()))
        }
    }
    let rubric = ["a".to_owned()];
    let s = score_answer(
        &Never,
        &Comparison {
            rubric: &rubric,
            ..cmp("x", "   \n ")
        },
    )
    .unwrap();
    assert_eq!(
        (s.value, s.verdict, s.rubric_hits),
        (0.0, Verdict::Incorrect, vec![false])
    );
}

#[test]
fn verdict_bands_follow_the_spec_and_map_to_ratings() {
    assert_eq!(verdict_for(0.7, 0.7), Verdict::Correct);
    assert_eq!(verdict_for(0.69, 0.7), Verdict::Partial);
    assert_eq!(verdict_for(0.5, 0.7), Verdict::Partial);
    assert_eq!(verdict_for(0.49, 0.7), Verdict::Incorrect);
    assert_eq!(suggested_rating(Verdict::Correct), Rating::Good);
    assert_eq!(suggested_rating(Verdict::Partial), Rating::Hard);
    assert_eq!(suggested_rating(Verdict::Incorrect), Rating::Again);
}

#[test]
fn the_lexical_runtime_ranks_right_above_close_above_wrong() {
    let reference = "The mitochondria is the powerhouse of the cell";
    let right = lexical(reference, "mitochondria are the powerhouse of the cell").value;
    let close = lexical(reference, "the mitochondria makes energy").value;
    let wrong = lexical(reference, "the capital of France is Paris").value;
    assert!(right > close && close > wrong, "{right} {close} {wrong}");
    assert_eq!(
        lexical(reference, "the mitochondria is the powerhouse of the cell").verdict,
        Verdict::Correct
    );
    // Honest limit: word order only nudges the score; meaning is not understood.
    let swapped = lexical("dog bites man", "man bites dog").value;
    assert!(swapped < 0.95 && swapped > 0.5, "{swapped}");
}

proptest! {
    #[test]
    fn scores_stay_in_range_and_are_deterministic(r in "\\PC{0,60}", a in "\\PC{0,60}") {
        let s1 = lexical(&r, &a);
        let s2 = lexical(&r, &a);
        prop_assert!((0.0..=1.0).contains(&s1.value));
        prop_assert_eq!(s1, s2);
    }

    #[test]
    fn identical_text_scores_one(t in "[a-zA-Z]{1,8}( [a-zA-Z]{1,8}){0,8}") {
        prop_assert!((lexical(&t, &t).value - 1.0).abs() < 1e-5);
    }

    #[test]
    fn surrounding_whitespace_changes_nothing(r in "[a-z]{1,8}( [a-z]{1,8}){0,6}", a in "[a-z]{1,8}( [a-z]{1,8}){0,6}") {
        let padded = format!("  \n{a}\t ");
        prop_assert_eq!(lexical(&r, &a), lexical(&r, &padded));
    }

    #[test]
    fn an_empty_answer_scores_zero(r in "\\PC{0,40}", spaces in "[ \\t\\n]{0,5}") {
        prop_assert_eq!(lexical(&r, &spaces).value, 0.0);
    }

    #[test]
    fn more_rubric_points_hit_never_lowers_the_score(a in "[a-z]{1,6}( [a-z]{1,6}){0,5}") {
        let reference = "alpha beta gamma";
        let none = lexical(reference, &a).value;
        let own = [a.clone()];
        let s = score_answer(&Lexical, &Comparison { rubric: &own, ..cmp(reference, &a) }).unwrap();
        prop_assert!(s.rubric_hits == [true]);
        prop_assert!(s.value + 1e-6 >= 0.6 * none);
    }
}
