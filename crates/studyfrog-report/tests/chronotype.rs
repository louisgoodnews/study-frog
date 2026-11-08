#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Recall and answer time by local hour and weekday (STORY-0102).

use proptest::prelude::*;
use studyfrog_core::review::Rating;
use studyfrog_report::chronotype::{
    TimedLatency, TimedReview, chronotype, local_hour, local_weekday, with_windows,
};

/// Midnight UTC of Monday 2027-01-11.
const MONDAY: i64 = 1_799_625_600_000;
const HOUR: i64 = 3_600_000;

fn reviews_at(hour: i64, n: u32, again: u32) -> Vec<TimedReview> {
    (0..n)
        .map(|k| TimedReview {
            at_ms: MONDAY + hour * HOUR + i64::from(k) * 60_000,
            offset_minutes: 0,
            rating: if k < again {
                Rating::Again
            } else {
                Rating::Good
            },
        })
        .collect()
}

#[test]
fn the_local_hour_and_weekday_follow_the_offset() {
    assert_eq!((local_hour(MONDAY, 0), local_weekday(MONDAY, 0)), (0, 0));
    // 23:00 UTC on Monday is 01:00 on Tuesday two hours east, and 18:00 on Monday five hours west.
    let late = MONDAY + 23 * HOUR;
    assert_eq!((local_hour(late, 120), local_weekday(late, 120)), (1, 1));
    assert_eq!((local_hour(late, -300), local_weekday(late, -300)), (18, 0));
    // Sunday before the epoch-independent check: one day earlier is weekday 6.
    assert_eq!(local_weekday(MONDAY - HOUR, 0), 6);
}

#[test]
fn the_peak_is_the_best_window_and_the_trough_the_worst() {
    let mut reviews = reviews_at(9, 20, 1);
    reviews.extend(reviews_at(10, 20, 1));
    reviews.extend(reviews_at(20, 20, 10));
    reviews.extend(reviews_at(21, 20, 10));
    let lat = |hour: i64, ms: u32| TimedLatency {
        at_ms: MONDAY + hour * HOUR,
        offset_minutes: 0,
        ms,
    };
    let latencies = [
        lat(9, 3_000),
        lat(10, 3_000),
        lat(20, 9_000),
        lat(21, 9_000),
    ];
    let r = chronotype(&reviews, &latencies, 20);
    assert_eq!(r.hours.len(), 24);
    assert_eq!(r.weekdays.len(), 7);
    assert_eq!((r.hours[9].reviews, r.hours[9].recalled), (20, 19));
    assert_eq!(r.hours[9].median_ms, Some(3_000));
    assert_eq!(r.weekdays[0].reviews, 80, "all on Monday");
    let peak = r.peak.unwrap();
    assert_eq!((peak.start_hour, peak.hours, peak.reviews), (9, 2, 40));
    assert!((peak.retention - 0.95).abs() < 1e-6);
    assert_eq!(peak.median_ms, Some(3_000));
    let trough = r.trough.unwrap();
    assert_eq!(trough.start_hour, 20);
    assert!((trough.retention - 0.5).abs() < 1e-6);
    assert_eq!((r.best_weekday, r.worst_weekday), (Some(0), Some(0)));
}

#[test]
fn buckets_under_the_threshold_give_no_recommendation() {
    let mut reviews = reviews_at(9, 20, 1);
    reviews.extend(reviews_at(10, 20, 1));
    let r = chronotype(&reviews, &[], 25);
    assert_eq!((r.peak, r.trough), (None, None));
    assert_eq!(r.best_weekday, Some(0), "Monday has 40 reviews");
    assert_eq!(r.min_reviews, 25);
    let lower = with_windows(r, 20);
    assert_eq!(lower.peak.unwrap().start_hour, 9);
    assert_eq!(lower.min_reviews, 20);
    assert_eq!(lower.trough, None, "the only window is the peak");
}

#[test]
fn good_and_easy_are_counted_and_p90_is_reported() {
    let reviews: Vec<TimedReview> = [Rating::Hard, Rating::Good, Rating::Easy, Rating::Again]
        .iter()
        .map(|r| TimedReview {
            at_ms: MONDAY + 7 * HOUR,
            offset_minutes: 0,
            rating: *r,
        })
        .collect();
    let latencies: Vec<TimedLatency> = (1..=10)
        .map(|k| TimedLatency {
            at_ms: MONDAY + 7 * HOUR,
            offset_minutes: 0,
            ms: k * 1_000,
        })
        .collect();
    let r = chronotype(&reviews, &latencies, 1);
    let b = &r.hours[7];
    assert_eq!(
        (b.reviews, b.recalled, b.good_easy, b.answers),
        (4, 3, 2, 10)
    );
    assert_eq!((b.median_ms, b.p90_ms), (Some(5_000), Some(9_000)));
}

proptest! {
    #[test]
    fn every_review_lands_in_one_hour_and_one_weekday(
        times in proptest::collection::vec((0_i64..2_000_000_000_000, -720_i32..=840), 0..100)
    ) {
        let reviews: Vec<TimedReview> = times.iter().map(|(at, off)| TimedReview {
            at_ms: *at, offset_minutes: *off, rating: Rating::Good,
        }).collect();
        let r = chronotype(&reviews, &[], 20);
        prop_assert_eq!(r.hours.iter().map(|b| b.reviews).sum::<u32>() as usize, reviews.len());
        prop_assert_eq!(r.weekdays.iter().map(|b| b.reviews).sum::<u32>() as usize, reviews.len());
    }
}
