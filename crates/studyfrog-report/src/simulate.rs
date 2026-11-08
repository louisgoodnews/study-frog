//! A simulated learner, for checking the forecast when there is no history (or not enough). Pure and
//! seeded, so the same inputs always give the same reviews.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state, retrievability};
use studyfrog_srs::params::Params;

use crate::backtest::DatedReview;

/// What the simulated learner does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimConfig {
    /// Starts the random sequence.
    pub seed: u64,
    /// Days to simulate.
    pub days: i64,
    /// New items started per day.
    pub new_per_day: u32,
    /// Percent of due reviews put off by a day.
    pub miss_percent: u32,
}

/// The next number in `[0, 1)` of a seeded SplitMix64 sequence (deterministic, no allocation).
pub fn next_random(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((z ^ (z >> 31)) >> 40) as f32 / (1_u64 << 24) as f32
}

fn rating_of(recalled: bool, roll: f32) -> Rating {
    match (recalled, roll) {
        (false, _) => Rating::Again,
        (true, r) if r < 0.1 => Rating::Hard,
        (true, r) if r < 0.9 => Rating::Good,
        (true, _) => Rating::Easy,
    }
}

struct Sim {
    id: u64,
    state: MemState,
    last: i64,
    due: i64,
}

/// Every review the simulated learner makes, in day order.
pub fn simulate_history(c: &SimConfig, p: &Params) -> Vec<DatedReview> {
    let (mut rng, mut items, mut out) = (c.seed, Vec::<Sim>::new(), Vec::new());
    for day in 0..c.days {
        for item in &mut items {
            if item.due > day || next_random(&mut rng) * 100.0 < c.miss_percent as f32 {
                continue;
            }
            let r = retrievability(&item.state, (day - item.last) as f32, p);
            let rating = rating_of(next_random(&mut rng) < r, next_random(&mut rng));
            item.state = next_state(&item.state, rating, (day - item.last) as f32, p);
            (item.last, item.due) = (day, day + i64::from(interval_days(&item.state, p)));
            out.push(DatedReview {
                item: item.id,
                day,
                rating,
            });
        }
        for _ in 0..c.new_per_day {
            let rating = rating_of(true, next_random(&mut rng));
            let state = initial_state(rating, p);
            let id = items.len() as u64;
            out.push(DatedReview {
                item: id,
                day,
                rating,
            });
            items.push(Sim {
                id,
                state,
                last: day,
                due: day + i64::from(interval_days(&state, p)),
            });
        }
    }
    out
}
