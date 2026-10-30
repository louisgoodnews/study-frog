#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Performance numbers of a set of reviews.

use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_report::explore::{DayPoint, Perf, ReviewPoint, daily, perf_of, success_percent};
use studyfrog_report::latency::FlipSample;

const DAY: i64 = 86_400_000;

fn rev(item: u128, rating: Rating, at: i64) -> ReviewPoint {
    ReviewPoint {
        item: Uid(item),
        rating,
        at_ms: at,
    }
}

fn sample(item: u128, ms: u32) -> FlipSample {
    FlipSample {
        item: Uid(item),
        mark: None,
        ms,
    }
}

#[test]
fn counts_ratings_items_times_and_the_last_review() {
    let reviews = [
        rev(1, Rating::Good, 100),
        rev(1, Rating::Again, 300),
        rev(2, Rating::Easy, 200),
        rev(3, Rating::Hard, 50),
    ];
    let samples = [
        sample(1, 2_000),
        sample(2, 4_000),
        sample(9, 99_000),
        sample(1, 6_000),
    ];
    let p = perf_of(&reviews, &samples);
    assert_eq!(
        (p.reviews, p.items, p.ratings, p.last_ms),
        (4, 3, [1, 1, 1, 1], Some(300))
    );
    assert_eq!(
        (p.median_ms, p.total_ms),
        (Some(4_000), 12_000),
        "item 9 is not part of this set"
    );
    assert_eq!(success_percent(&p), Some(50));
}

#[test]
fn an_empty_set_has_no_rates() {
    let p = perf_of(&[], &[]);
    assert_eq!(p, Perf::default());
    assert_eq!(success_percent(&p), None);
}

#[test]
fn daily_points_fill_quiet_days_and_use_the_users_clock() {
    let today = 20_000;
    let reviews = [
        rev(1, Rating::Good, today * DAY + 1),
        rev(1, Rating::Again, today * DAY + 2),
        rev(2, Rating::Easy, (today - 2) * DAY + 5),
        rev(3, Rating::Good, (today - 40) * DAY),
        rev(4, Rating::Good, (today + 1) * DAY),
    ];
    let d = daily(&reviews, today, 3, 0);
    assert_eq!(
        d,
        [
            DayPoint {
                day: today - 2,
                reviews: 1,
                good: 1
            },
            DayPoint {
                day: today - 1,
                reviews: 0,
                good: 0
            },
            DayPoint {
                day: today,
                reviews: 2,
                good: 1
            },
        ],
        "outside the window is ignored"
    );
    let late = daily(
        &[rev(1, Rating::Good, today * DAY + DAY - 60_000)],
        today,
        1,
        120,
    );
    assert_eq!(late[0].reviews, 0, "23:59 UTC is already tomorrow at UTC+2");
}
