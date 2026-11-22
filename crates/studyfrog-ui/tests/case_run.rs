#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The case study inside a run (STORY-0199): the suggested rating follows the check, and a wrong choice
//! never suggests more than Hard.

use studyfrog_agent::rubric_eval::fallback_evaluate;
use studyfrog_core::review::Rating;
use studyfrog_ui::setup::case::run::suggested_rating;

fn criteria() -> Vec<String> {
    vec![
        "addresses scoring speed".to_owned(),
        "cites criterion validity".to_owned(),
    ]
}

#[test]
fn a_full_answer_with_the_right_choice_suggests_good_or_better() {
    let r = fallback_evaluate(
        "The short screen addresses scoring speed for busy staff and cites criterion validity for this group.",
        true,
        &criteria(),
    );
    assert!(suggested_rating(&r) >= Rating::Good, "{r:?}");
}

#[test]
fn a_wrong_choice_suggests_at_most_hard_however_well_it_is_argued() {
    let r = fallback_evaluate(
        "The short screen addresses scoring speed and cites criterion validity.",
        false,
        &criteria(),
    );
    assert!(suggested_rating(&r) <= Rating::Hard);
}

#[test]
fn an_empty_answer_suggests_again() {
    let r = fallback_evaluate("", true, &criteria());
    assert_eq!(suggested_rating(&r), Rating::Again);
}
