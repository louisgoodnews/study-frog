//! One simulated future of a deck: every card's FSRS or SM-2 state is carried forward day by day, each review
//! recalled with the probability the model gives, rated like the learner rates, with days missed now and then and
//! new cards started at a steady pace (STORY-0105). Pure and seeded.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state, retrievability};
use studyfrog_srs::params::Params;
use studyfrog_srs::sm2::{sm2_initial, sm2_next};

pub use crate::sim_types::*;
use crate::simulate::next_random;

/// Stability (days) from which a card counts as mature.
pub const MATURE_DAYS: f32 = 21.0;

/// The stability a card's memory stands for: FSRS stability, or the SM-2 interval (both are "days until recall
/// falls to 90 %").
pub fn stability_of(m: &SimMemory) -> f32 {
    match m {
        SimMemory::Fsrs(s) => s.stability,
        SimMemory::Sm2(s) => (s.interval_days as f32).max(1.0),
    }
}

/// Probability of recall on `day`, `None` for a new card.
pub fn recall_on(card: &SimCard, day: i64, p: &Params) -> Option<f32> {
    let m = card.memory?;
    let s = MemState {
        stability: stability_of(&m).max(0.01),
        difficulty: 5.0,
    };
    Some(retrievability(&s, (day - card.last_day) as f32, p))
}

fn rating(recalled: bool, mix: &RatingMix, roll: f32) -> Rating {
    match (recalled, roll) {
        (false, _) => Rating::Again,
        (true, r) if r < mix.hard => Rating::Hard,
        (true, r) if r < mix.hard + mix.good => Rating::Good,
        (true, _) => Rating::Easy,
    }
}

/// Rate `card` on `day` and give its next due day.
fn rate(card: &mut SimCard, day: i64, r: Rating, p: &Params) -> i64 {
    let elapsed = (day - card.last_day) as f32;
    let (memory, interval) = match card.memory {
        Some(SimMemory::Fsrs(s)) => {
            let n = next_state(&s, r, elapsed, p);
            (SimMemory::Fsrs(n), interval_days(&n, p))
        }
        Some(SimMemory::Sm2(s)) => {
            let n = sm2_next(&s, r, p.maximum_interval_days);
            (SimMemory::Sm2(n), n.interval_days.max(1))
        }
        None if card.sm2 => {
            let n = sm2_next(&sm2_initial(), r, p.maximum_interval_days);
            (SimMemory::Sm2(n), n.interval_days.max(1))
        }
        None => {
            let n = initial_state(r, p);
            (SimMemory::Fsrs(n), interval_days(&n, p))
        }
    };
    (card.memory, card.last_day, card.relearning) = (Some(memory), day, r == Rating::Again);
    day + i64::from(interval)
}

fn pace_of(s: &SimSetup, card: &SimCard) -> Pace {
    s.paces
        .get(usize::from(card.pace))
        .copied()
        .unwrap_or_else(pace_default)
}

fn review(
    card: &mut SimCard,
    day: i64,
    s: &SimSetup,
    p: &Params,
    rng: &mut u64,
    count: &mut DayCount,
) -> i64 {
    let pace = pace_of(s, card);
    let recall = recall_on(card, day, p).unwrap_or(1.0);
    let mature = card.memory.is_some_and(|m| stability_of(&m) >= MATURE_DAYS);
    match (card.relearning, mature) {
        (true, _) => count.learning += 1,
        (false, true) => count.mature += 1,
        (false, false) => count.young += 1,
    }
    count.seconds += if card.relearning {
        pace.learn_s
    } else {
        pace.review_s
    };
    let recalled = next_random(rng) < recall;
    rate(card, day, rating(recalled, &s.mix, next_random(rng)), p)
}

fn learn(
    card: &mut SimCard,
    day: i64,
    s: &SimSetup,
    p: &Params,
    rng: &mut u64,
    count: &mut DayCount,
) -> i64 {
    (count.new, count.seconds) = (count.new + 1, count.seconds + pace_of(s, card).new_s);
    let recalled = next_random(rng) >= s.mix.first_again;
    rate(card, day, rating(recalled, &s.mix, next_random(rng)), p)
}

fn schedule(buckets: &mut [Vec<u32>], today: i64, due: i64, idx: u32) {
    let at = usize::try_from(due - today).unwrap_or(0);
    if let Some(b) = buckets.get_mut(at) {
        b.push(idx);
    }
}

/// A missed day: its reviews move to the next day (or out of the horizon).
fn postpone(buckets: &mut [Vec<u32>], to: usize, due: Vec<u32>) {
    if let Some(next) = buckets.get_mut(to) {
        next.extend(due);
    }
}

/// Simulate `cards` (changed in place to their state at the end) for `s.days` days; the reviews of each day.
pub fn rollout(cards: &mut [SimCard], s: &SimSetup, p: &Params, rng: &mut u64) -> Vec<DayCount> {
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); s.days];
    let mut unseen: Vec<u32> = Vec::new();
    for (i, c) in cards.iter().enumerate() {
        match c.memory {
            Some(_) => schedule(&mut buckets, s.today, c.due_day.max(s.today), i as u32),
            None => unseen.push(i as u32),
        }
    }
    unseen.reverse();
    let (mut out, mut credit) = (vec![DayCount::default(); s.days], 0.0_f32);
    for d in 0..s.days {
        let day = s.today + d as i64;
        let due = std::mem::take(&mut buckets[d]);
        if next_random(rng) < s.miss_probability {
            postpone(&mut buckets, d + 1, due);
            continue;
        }
        for idx in due {
            let next = review(&mut cards[idx as usize], day, s, p, rng, &mut out[d]);
            schedule(&mut buckets, s.today, next, idx);
        }
        credit += if d < s.new_days { s.new_per_day } else { 0.0 };
        while credit >= 1.0 {
            let Some(idx) = unseen.pop() else { break };
            credit -= 1.0;
            let next = learn(&mut cards[idx as usize], day, s, p, rng, &mut out[d]);
            schedule(&mut buckets, s.today, next, idx);
        }
    }
    out
}

/// The seed of trajectory `t`.
pub fn trajectory_seed(seed: u64, t: u32) -> u64 {
    seed ^ u64::from(t).wrapping_mul(0x9E37_79B9_7F4A_7C15)
}
