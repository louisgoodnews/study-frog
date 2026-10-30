#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Golden test against the py-fsrs reference (`tests/test_basic.py::test_review_card`):
//! `Scheduler(enable_fuzzing=False)`, default parameters, 13 ratings, each review taken when the
//! card is due. Expected intervals are copied from that test.
//!
//! The reference also models learning steps (1 min, 10 min). This harness emulates them with
//! plain data: reviews inside a step happen the same day (elapsed 0), graduation yields days.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating::{self, Again, Good};
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::{Params, params_default};

const RATINGS: [Rating; 13] = [
    Good, Good, Good, Good, Good, Good, Again, Again, Good, Good, Good, Good, Good,
];
const EXPECTED: [u32; 13] = [0, 2, 11, 46, 163, 498, 0, 0, 2, 4, 7, 12, 21];

#[derive(Clone, Copy)]
enum Phase {
    New,
    Learning,
    Review { interval: u32 },
    Relearning,
}

#[derive(Clone, Copy)]
struct Sim {
    state: Option<MemState>,
    phase: Phase,
}

/// Apply one rating; returns the new simulation and the scheduled interval in days (0 = a step).
fn step(sim: Sim, rating: Rating, p: &Params) -> (Sim, u32) {
    match (sim.phase, sim.state) {
        (Phase::New, _) => (
            Sim {
                state: Some(initial_state(rating, p)),
                phase: Phase::Learning,
            },
            0,
        ),
        (Phase::Learning | Phase::Relearning, Some(s)) => same_day_step(s, sim.phase, rating, p),
        (Phase::Review { interval }, Some(s)) => review_step(s, interval, rating, p),
        _ => unreachable!("state exists after the first review"),
    }
}

fn same_day_step(prev: MemState, phase: Phase, rating: Rating, p: &Params) -> (Sim, u32) {
    let next = next_state(&prev, rating, 0.0, p);
    if rating == Again {
        return (
            Sim {
                state: Some(next),
                phase,
            },
            0,
        );
    }
    let days = interval_days(&next, p);
    (
        Sim {
            state: Some(next),
            phase: Phase::Review { interval: days },
        },
        days,
    )
}

fn review_step(prev: MemState, elapsed: u32, rating: Rating, p: &Params) -> (Sim, u32) {
    let next = next_state(&prev, rating, elapsed as f32, p);
    if rating == Again {
        return (
            Sim {
                state: Some(next),
                phase: Phase::Relearning,
            },
            0,
        );
    }
    let days = interval_days(&next, p);
    (
        Sim {
            state: Some(next),
            phase: Phase::Review { interval: days },
        },
        days,
    )
}

#[test]
fn matches_py_fsrs_reference_intervals() {
    let p = params_default();
    let mut sim = Sim {
        state: None,
        phase: Phase::New,
    };
    let mut got = Vec::new();
    for rating in RATINGS {
        let (next, days) = step(sim, rating, &p);
        sim = next;
        got.push(days);
    }
    assert_eq!(got, EXPECTED);
}
