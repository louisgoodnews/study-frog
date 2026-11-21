//! The cognitive profile (STORY-0178): built from review history rows.
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_service::cognitive_report::profile;
use studyfrog_store::reports::HistoryRow;

const WEEK_MS: i64 = 7 * 24 * 3_600_000;

fn row(rating: Rating, at_ms: i64, answer_ms: Option<u32>, expected_ms: Option<u32>) -> HistoryRow {
    HistoryRow {
        item: Uid(1),
        rating,
        at_ms,
        elapsed_days: 1.0,
        answer_ms,
        expected_ms,
    }
}

#[test]
fn empty_history_gives_an_empty_profile() {
    let p = profile(&[]);
    assert_eq!(p.answers, 0);
    assert_eq!(p.timed, 0);
    assert!(p.velocity.is_none());
    assert!(p.calibration.is_none());
    assert!(p.weekly_pass.is_empty());
}

#[test]
fn untimed_answers_count_but_are_not_plotted() {
    let rows = [
        row(Rating::Good, 0, None, None),
        row(Rating::Good, 1, Some(4_000), None),
    ];
    let p = profile(&rows);
    assert_eq!(p.answers, 2);
    assert_eq!(p.timed, 0);
    assert!(p.length_points.is_empty());
}

#[test]
fn calibration_compares_easy_ratings_with_in_time_answers() {
    let rows = [
        // Easy and in time.
        row(Rating::Easy, 0, Some(3_000), Some(4_000)),
        // Easy but slow: the rating over-states the recall.
        row(Rating::Easy, 1, Some(9_000), Some(4_000)),
        row(Rating::Good, 2, Some(2_000), Some(4_000)),
        row(Rating::Hard, 3, Some(8_000), Some(4_000)),
    ];
    let cal = profile(&rows).calibration.expect("calibration");
    assert!((cal.easy_share - 0.5).abs() < 1e-6);
    assert!((cal.in_time_share - 0.5).abs() < 1e-6);
}

#[test]
fn weekly_pass_is_the_cumulative_share_per_week() {
    let rows = [
        row(Rating::Good, 0, None, None),
        row(Rating::Again, 1, None, None),
        row(Rating::Easy, WEEK_MS + 5, None, None),
    ];
    let weeks = profile(&rows).weekly_pass;
    assert_eq!(weeks.len(), 2);
    assert_eq!(weeks[0], (0.0, 0.5));
    // Cumulative: two of three answers so far passed.
    assert_eq!(weeks[1].0, 1.0);
    assert!((weeks[1].1 - 2.0 / 3.0).abs() < 1e-6);
}

#[test]
fn velocity_is_passes_per_minute_of_answering() {
    // Every answer takes one minute and passes: one more pass per minute.
    let rows: Vec<HistoryRow> = (0..5)
        .map(|i| row(Rating::Good, i, Some(60_000), Some(4_000)))
        .collect();
    let v = profile(&rows).velocity.expect("velocity");
    assert!((v - 1.0).abs() < 1e-9, "{v}");
}

#[test]
fn length_points_keep_the_most_recent_five_hundred() {
    let rows: Vec<HistoryRow> = (0..600)
        .map(|i| row(Rating::Good, i, Some(1_000 + i as u32), Some(2_000)))
        .collect();
    let p = profile(&rows);
    assert_eq!(p.timed, 600);
    assert_eq!(p.length_points.len(), 500);
    assert!((p.length_points[0].1 - 1.1).abs() < 1e-3);
}
