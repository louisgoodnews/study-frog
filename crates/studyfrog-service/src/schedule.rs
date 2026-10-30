//! Applying a rating to the schedule: the pure core of "a review happened".

use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::Params;
use studyfrog_store::schedule::StoredMem;

const MS_PER_DAY: i64 = 86_400_000;

/// Whole days between two instants, never negative.
fn whole_days(from: UnixMs, to: UnixMs) -> f32 {
    ((to.0 - from.0).max(0) / MS_PER_DAY) as f32
}

/// The review record and new memory state after rating `item` at wall-clock time `now`.
/// `prev` is the item's stored state, `None` for its first review.
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
    let due = UnixMs(now.0 + i64::from(interval_days(&state, params)) * MS_PER_DAY);
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
