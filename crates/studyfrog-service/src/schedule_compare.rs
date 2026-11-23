//! The adaptive schedule against the fixed one (STORY-0177). The first review gives the same
//! stability under both (it depends on the rating only), so the difference shows at the second
//! review: each item's first review is replayed with the plain prior and with the calibrated start,
//! then its second review gives an interval under each. Pure: the history is read by the caller.

use std::collections::HashMap;

use studyfrog_core::memory::MemState;
use studyfrog_core::uid::Uid;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::Params;
use studyfrog_store::reports::HistoryRow;

use crate::schedule::{calibrated_initial, timing_difficulty};

/// First-review difficulties the calibrated start learns from at most (as the store reads them).
const MAX_OBSERVED: usize = 500;

/// The intervals after each item's second review, under the two schemes.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScheduleCompare {
    /// Items with a second review in the history.
    pub items: usize,
    /// Mean interval after the second review with the plain prior, in days.
    pub fixed_days: f32,
    /// Mean interval after the second review with the calibrated start, in days.
    pub calibrated_days: f32,
    /// Items whose two intervals differ.
    pub differ: usize,
    /// Mean stability after the second review with the plain prior, in days.
    pub fixed_stability: f32,
    /// Mean stability after the second review with the calibrated start, in days.
    pub calibrated_stability: f32,
}

/// An item after its first review, under both schemes. Its second review is still to come.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Pending {
    fixed: MemState,
    calibrated: MemState,
}

/// Where an item is in the replay.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    /// First review done, second not yet.
    Once(Pending),
    /// Second review compared; later reviews are not looked at.
    Done,
}

/// The replay: the stage of each item, the first-review difficulties seen so far (oldest first), and
/// the running sums of the intervals and stabilities under the fixed and the calibrated scheme.
#[derive(Default)]
struct Replay {
    stages: HashMap<Uid, Stage>,
    observed: Vec<f64>,
    days: (f64, f64),
    stability: (f64, f64),
    items: usize,
    differ: usize,
}

/// The difficulties the calibrated start may learn from now: the newest [`MAX_OBSERVED`].
fn recent(observed: &[f64]) -> &[f64] {
    &observed[observed.len().saturating_sub(MAX_OBSERVED)..]
}

/// Replay a first review: both starting states are fixed now, from the difficulties seen so far.
fn first_review(replay: &mut Replay, row: &HistoryRow, params: &Params) {
    let pending = Pending {
        fixed: initial_state(row.rating, params),
        calibrated: calibrated_initial(row.rating, params, recent(&replay.observed)),
    };
    replay.stages.insert(row.item, Stage::Once(pending));
    if let (Some(answer), Some(expected)) = (row.answer_ms, row.expected_ms) {
        replay.observed.push(timing_difficulty(answer, expected));
    }
}

/// Replay a second review: both schemes step on, and their intervals are compared.
fn second_review(replay: &mut Replay, pending: Pending, row: &HistoryRow, params: &Params) {
    let fixed = next_state(&pending.fixed, row.rating, row.elapsed_days, params);
    let calibrated = next_state(&pending.calibrated, row.rating, row.elapsed_days, params);
    let (fixed_days, calibrated_days) = (
        interval_days(&fixed, params),
        interval_days(&calibrated, params),
    );
    replay.days.0 += f64::from(fixed_days);
    replay.days.1 += f64::from(calibrated_days);
    replay.stability.0 += f64::from(fixed.stability);
    replay.stability.1 += f64::from(calibrated.stability);
    replay.items += 1;
    replay.differ += usize::from(fixed_days != calibrated_days);
}

/// Feed one review (oldest first) into the replay.
fn step(replay: &mut Replay, row: &HistoryRow, params: &Params) {
    match replay.stages.get(&row.item).copied() {
        None => first_review(replay, row, params),
        Some(Stage::Once(pending)) => {
            second_review(replay, pending, row, params);
            replay.stages.insert(row.item, Stage::Done);
        }
        Some(Stage::Done) => {}
    }
}

/// Compare the intervals after each item's second review, replaying the history oldest first. An
/// item's first review is matched by item only, as the store does for first answers.
pub fn compare_second_reviews(rows: &[HistoryRow], params: &Params) -> ScheduleCompare {
    let mut replay = Replay::default();
    for row in rows {
        step(&mut replay, row, params);
    }
    let n = replay.items.max(1) as f64;
    ScheduleCompare {
        items: replay.items,
        fixed_days: (replay.days.0 / n) as f32,
        calibrated_days: (replay.days.1 / n) as f32,
        differ: replay.differ,
        fixed_stability: (replay.stability.0 / n) as f32,
        calibrated_stability: (replay.stability.1 / n) as f32,
    }
}
