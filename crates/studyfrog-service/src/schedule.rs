//! Applying a rating to the schedule: the pure core of "a review happened".

use rusqlite::Connection;
use studyfrog_core::memory::MemState;
use studyfrog_core::review::{Rating, Review};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::difficulty::{
    EXPECTED_MS_PER_WORD, Observation, calibrated_prior, empirical_difficulty,
};
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::Params;
use studyfrog_srs::sm2::{sm2_initial, sm2_next};
use studyfrog_store::error::StoreError;
use studyfrog_store::first_timing::first_review_timings;
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
    apply_rating_calibrated(prev, item, rating, now, params, &[])
}

/// The difficulties the user's first answers imply, on the scale of the difficulty engine. Each is
/// computed from the answer time and the reading time the words should take; semantic scores are
/// not stored, so a neutral score is used.
pub fn observed_difficulties(conn: &Connection) -> Result<Vec<f64>, StoreError> {
    Ok(first_review_timings(conn)?
        .into_iter()
        .map(|(answer_ms, expected_ms)| {
            empirical_difficulty(&Observation {
                latency_ms: f64::from(answer_ms),
                dwell_ms: f64::from(answer_ms),
                words: f64::from(expected_ms) / EXPECTED_MS_PER_WORD,
                semantic: None,
            })
        })
        .collect())
}

/// Like [`apply_rating`], but the starting difficulty of a first review is calibrated: the scheduler's
/// prior is blended with the difficulties of the user's earlier first answers (`observed`). With no
/// observations the prior is used unchanged.
pub fn apply_rating_calibrated(
    prev: Option<&StoredMem>,
    item: Uid,
    rating: Rating,
    now: UnixMs,
    params: &Params,
    observed: &[f64],
) -> (Review, StoredMem) {
    let elapsed_days = prev.map_or(0.0, |p| whole_days(p.last_review, now));
    let state = match prev {
        Some(p) => next_state(&p.state, rating, elapsed_days, params),
        None => calibrated_initial(rating, params, observed),
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

/// The first FSRS state, with its difficulty blended with `observed` when there are any.
pub fn calibrated_initial(rating: Rating, params: &Params, observed: &[f64]) -> MemState {
    let mut state = initial_state(rating, params);
    if !observed.is_empty() {
        let blended = calibrated_prior(f64::from(state.difficulty), observed);
        state.difficulty = blended as f32;
    }
    state
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
