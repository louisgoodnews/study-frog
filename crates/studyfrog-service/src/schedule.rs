//! Applying a rating to the schedule: the pure core of "a review happened".

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::Params;
use studyfrog_srs::sm2::{sm2_initial, sm2_next};
use studyfrog_store::schedule::{StoredMem, StoredSm2};

const MS_PER_DAY: i64 = 86_400_000;

/// Whole days between two instants, never negative.
fn whole_days(from: UnixMs, to: UnixMs) -> f32 {
    ((to.0 - from.0).max(0) / MS_PER_DAY) as f32
}

fn due_in(now: UnixMs, days: u32) -> UnixMs {
    UnixMs(now.0 + i64::from(days) * MS_PER_DAY)
}

/// FSRS: the review record and new memory state after rating `item` at wall-clock time `now`.
/// `prev` is the item's stored FSRS state, `None` for its first FSRS review.
pub fn apply_rating(
    prev: Option<&StoredMem>,
    item: Uid,
    rating: Rating,
    now: UnixMs,
    params: &Params,
) -> (Review, StoredMem) {
    let elapsed_days = prev.map_or(0.0, |p| whole_days(p.last_review, now));
    let state = match prev {
        Some(p) => next_state(&p.state, rating, elapsed_days, params),
        None => initial_state(rating, params),
    };
    let due = due_in(now, interval_days(&state, params));
    (
        Review {
            item,
            rating,
            at: now,
            elapsed_days,
        },
        StoredMem {
            state,
            due,
            last_review: now,
        },
    )
}

/// SM-2: the review record and new state. `prev` is the stored SM-2 state, `None` for the first
/// SM-2 review. Intervals are capped at `maximum_interval_days`.
pub fn apply_rating_sm2(
    prev: Option<&StoredSm2>,
    item: Uid,
    rating: Rating,
    now: UnixMs,
    maximum_interval_days: u32,
) -> (Review, StoredSm2) {
    let elapsed_days = prev.map_or(0.0, |p| whole_days(p.last_review, now));
    let before = prev.map_or_else(sm2_initial, |p| p.state);
    let state = sm2_next(&before, rating, maximum_interval_days);
    let due = due_in(now, state.interval_days);
    (
        Review {
            item,
            rating,
            at: now,
            elapsed_days,
        },
        StoredSm2 {
            state,
            due,
            last_review: now,
        },
    )
}
