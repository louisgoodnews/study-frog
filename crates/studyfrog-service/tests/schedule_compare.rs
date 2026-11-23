#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The adaptive schedule against the fixed one (STORY-0177): with no timing the two agree; slow
//! first answers make the calibrated start harder, which shortens the interval after the second review.

use studyfrog_core::review::Rating::{Good, Hard};
use studyfrog_core::uid::Uid;
use studyfrog_service::schedule_compare::compare_second_reviews;
use studyfrog_srs::params::params_default;
use studyfrog_store::reports::HistoryRow;

fn row(
    item: u128,
    rating: studyfrog_core::review::Rating,
    at_ms: i64,
    elapsed: f32,
    timing: Option<(u32, u32)>,
) -> HistoryRow {
    HistoryRow {
        item: Uid(item),
        rating,
        at_ms,
        elapsed_days: elapsed,
        answer_ms: timing.map(|t| t.0),
        expected_ms: timing.map(|t| t.1),
    }
}

#[test]
fn without_timing_the_two_schedules_agree() {
    let rows = [
        row(1, Good, 0, 0.0, None),
        row(1, Good, 3 * 86_400_000, 3.0, None),
        row(2, Hard, 0, 0.0, None),
        row(2, Good, 2 * 86_400_000, 2.0, None),
    ];
    let c = compare_second_reviews(&rows, &params_default());
    assert_eq!(c.items, 2);
    assert_eq!(c.differ, 0);
    assert_eq!(c.fixed_days, c.calibrated_days);
}

#[test]
fn slow_first_answers_shorten_the_interval_after_the_second_review() {
    let day = 86_400_000_i64;
    let mut rows: Vec<HistoryRow> = (10..30)
        .map(|i| row(i, Good, i as i64, 0.0, Some((30_000, 2_000))))
        .collect();
    rows.push(row(1, Good, 40 * day, 0.0, Some((2_000, 2_000))));
    rows.push(row(1, Good, 43 * day, 3.0, None));
    let c = compare_second_reviews(&rows, &params_default());
    assert_eq!(c.items, 1);
    assert_eq!(c.differ, 1);
    assert!(c.calibrated_days < c.fixed_days, "{c:?}");
    assert!(c.calibrated_stability < c.fixed_stability, "{c:?}");
}

#[test]
fn items_with_one_review_are_not_compared() {
    let rows = [row(1, Good, 0, 0.0, None), row(2, Hard, 1, 0.0, None)];
    let c = compare_second_reviews(&rows, &params_default());
    assert_eq!(c.items, 0);
}
