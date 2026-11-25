//! The three grading schemes for choice items, on single choice and multiple select, at the
//! boundaries: nothing chosen, everything chosen, one wrong, all correct (STORY-0231).

use studyfrog_core::choice_grading::ChoiceGrading;
use studyfrog_core::choice_grading::ChoiceGrading::{AllOrNothing, CountPartial, Zapechelnyuk};
use studyfrog_rehearsal::choice_grade::{ChoiceKey, grade_choice_with};
use studyfrog_rehearsal::choice_scheme::credited_fraction;

const EPS: f32 = 1e-6;

fn assert_fraction(scheme: ChoiceGrading, picked: &[bool], correct: &[bool], want: f32) {
    let got = credited_fraction(scheme, picked, correct);
    assert!(
        (got - want).abs() < EPS,
        "{scheme:?} picked {picked:?} correct {correct:?}: got {got}, want {want}"
    );
}

// Single choice: three options, the second is correct.
const SINGLE: [bool; 3] = [false, true, false];

#[test]
fn single_nothing_chosen_earns_nothing_under_every_scheme() {
    for scheme in [AllOrNothing, Zapechelnyuk, CountPartial] {
        assert_fraction(scheme, &[false, false, false], &SINGLE, 0.0);
    }
}

#[test]
fn single_correct_choice_is_full_under_every_scheme() {
    for scheme in [AllOrNothing, Zapechelnyuk, CountPartial] {
        assert_fraction(scheme, &[false, true, false], &SINGLE, 1.0);
    }
}

#[test]
fn single_one_wrong_earns_nothing_under_every_scheme() {
    for scheme in [AllOrNothing, Zapechelnyuk, CountPartial] {
        assert_fraction(scheme, &[true, false, false], &SINGLE, 0.0);
    }
}

#[test]
fn single_everything_chosen_dilutes_zapechelnyuk_and_clamps_count() {
    assert_fraction(AllOrNothing, &[true, true, true], &SINGLE, 0.0);
    // k = 3 chosen, the correct one among them: 1/3.
    assert_fraction(Zapechelnyuk, &[true, true, true], &SINGLE, 1.0 / 3.0);
    // 1 correct minus 2 wrong is negative: clamped to 0.
    assert_fraction(CountPartial, &[true, true, true], &SINGLE, 0.0);
}

// Multiple select: three options, the first and the third are correct (two correct in all).
const MULTIPLE: [bool; 3] = [true, false, true];

#[test]
fn multiple_nothing_chosen_earns_nothing_under_every_scheme() {
    for scheme in [AllOrNothing, Zapechelnyuk, CountPartial] {
        assert_fraction(scheme, &[false, false, false], &MULTIPLE, 0.0);
    }
}

#[test]
fn multiple_everything_chosen() {
    assert_fraction(AllOrNothing, &[true, true, true], &MULTIPLE, 0.0);
    // k = 3, two of them correct: 1/3.
    assert_fraction(Zapechelnyuk, &[true, true, true], &MULTIPLE, 1.0 / 3.0);
    // 2 correct minus 1 wrong over 2 correct: 0.5.
    assert_fraction(CountPartial, &[true, true, true], &MULTIPLE, 0.5);
}

#[test]
fn multiple_one_wrong_among_two_chosen() {
    // Chosen: one correct, one wrong.
    assert_fraction(AllOrNothing, &[true, true, false], &MULTIPLE, 0.0);
    // k = 2, one correct among them: 1/2.
    assert_fraction(Zapechelnyuk, &[true, true, false], &MULTIPLE, 0.5);
    // 1 correct minus 1 wrong: 0.
    assert_fraction(CountPartial, &[true, true, false], &MULTIPLE, 0.0);
}

#[test]
fn multiple_one_correct_chosen_alone() {
    assert_fraction(AllOrNothing, &[true, false, false], &MULTIPLE, 0.0);
    // k = 1, and it is correct: 1.
    assert_fraction(Zapechelnyuk, &[true, false, false], &MULTIPLE, 1.0);
    // 1 of 2 correct, nothing wrong: 0.5.
    assert_fraction(CountPartial, &[true, false, false], &MULTIPLE, 0.5);
}

#[test]
fn multiple_all_correct_chosen() {
    assert_fraction(AllOrNothing, &[true, false, true], &MULTIPLE, 1.0);
    // Owner override: every correct option and no wrong one is full credit, not 1/2.
    assert_fraction(Zapechelnyuk, &[true, false, true], &MULTIPLE, 1.0);
    assert_fraction(CountPartial, &[true, false, true], &MULTIPLE, 1.0);
}

#[test]
fn short_picked_slice_counts_missing_options_as_not_chosen() {
    assert_fraction(CountPartial, &[true], &MULTIPLE, 0.5);
}

#[test]
fn graded_choice_carries_the_fraction_of_its_scheme() {
    let key = ChoiceKey {
        options: vec!["Whale".into(), "Shark".into(), "Bat".into()],
        correct_indices: vec![0, 2],
    };
    let grade = grade_choice_with(&key, &[true, true, false], CountPartial).unwrap();
    assert!(!grade.correct, "the all-or-nothing flag stays exact");
    assert!(grade.fraction.abs() < EPS);
    let grade = grade_choice_with(&key, &[true, false, false], Zapechelnyuk).unwrap();
    assert!((grade.fraction - 1.0).abs() < EPS);
}
