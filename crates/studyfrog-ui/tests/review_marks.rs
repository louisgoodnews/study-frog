#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Slow reads and the user's own mark as sources of a suggested rating.

mod common;

use common::runs::{finished_run, finished_run_marked};
use common::views::harness;
use egui_kittest::kittest::Queryable;
use studyfrog_core::event::Difficulty;
use studyfrog_core::review::Rating;
use studyfrog_ui::review::review_rows;

#[test]
fn a_slow_read_is_suggested_one_step_harder_with_its_reason() {
    let m = finished_run(12.0, Some(Rating::Good), Rating::Good);
    let r = &review_rows(m.state.rehearse.run.as_ref().unwrap())[0];
    let s = r.suggestion.unwrap();
    assert_eq!(
        (s.rating, s.reason),
        (Rating::Hard, studyfrog_rehearsal::reading::Reason::Slow)
    );
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("This took you a long time to read: it may be harder than you rated it.")
            .is_some()
    );
    assert!(
        h.query_by_label("Change the rating of Numbers to Hard")
            .is_some()
    );
}

#[test]
fn the_users_mark_suggests_a_rating_and_wins_over_reading_speed() {
    let m = finished_run_marked(4.0, Some(Rating::Good), Rating::Good, Difficulty::Difficult);
    let s = review_rows(m.state.rehearse.run.as_ref().unwrap())[0]
        .suggestion
        .unwrap();
    assert_eq!(
        (s.rating, s.reason),
        (
            Rating::Hard,
            studyfrog_rehearsal::reading::Reason::MarkedDifficult
        )
    );

    // A new item read fast would be suggested Hard by reading speed; the user said it was easy.
    let m = finished_run_marked(2.0, None, Rating::Good, Difficulty::Easy);
    let s = review_rows(m.state.rehearse.run.as_ref().unwrap())[0]
        .suggestion
        .unwrap();
    assert_eq!(
        (s.rating, s.reason),
        (
            Rating::Easy,
            studyfrog_rehearsal::reading::Reason::MarkedEasy
        )
    );
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("You marked this item easy, and it was rather easy for you.")
            .is_some()
    );

    let m = finished_run_marked(4.0, Some(Rating::Good), Rating::Hard, Difficulty::Easy);
    assert_eq!(
        review_rows(m.state.rehearse.run.as_ref().unwrap())[0].suggestion,
        None,
        "a low rating stands"
    );
}
