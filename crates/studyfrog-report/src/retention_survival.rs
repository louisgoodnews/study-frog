//! True retention and empirical forgetting curves by stability tier (STORY-0107). Pure.
//!
//! The history is replayed through FSRS so every review knows the stability `S` and the interval `I`
//! the item had, and the time `t` since its previous review. Reviews are then counted by maturity
//! (`S >= 21` days), by timeliness (`t / I` early, on time, overdue) and, per stability tier, by the
//! relative elapsed time `t / S`, next to what FSRS predicted (`R(t, S)`).

use std::collections::HashMap;

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::fsrs::{initial_state, interval_exact, next_state, retrievability};
use studyfrog_srs::params::Params;

pub use crate::survival_types::*;

/// Stability (days) from which an item counts as mature.
pub const MATURE_DAYS: f32 = 21.0;
/// Edges of the stability tiers in days.
pub const TIER_EDGES: [f32; 4] = [0.0, 7.0, 30.0, 90.0];
/// Edges of the `t / S` bins of a forgetting curve.
pub const RATIO_EDGES: [f32; 5] = [0.0, 0.5, 1.0, 2.0, 4.0];
/// Reviews needed before the memory scale and half-life are fitted.
pub const MIN_FIT_REVIEWS: u32 = 30;

/// One review of an item, in the order they happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurvivalReview {
    /// The item, as a number.
    pub item: u64,
    /// The rating.
    pub rating: Rating,
    /// Days since the item's previous review (0 for the first).
    pub elapsed_days: f32,
}

fn add(c: &mut RateCount, recalled: bool, predicted: f32, ratio: f32) {
    c.reviews += 1;
    c.recalled += u32::from(recalled);
    c.predicted_sum += predicted;
    c.ratio_sum += ratio;
}

fn edge_index(edges: &[f32], x: f32) -> usize {
    edges.iter().rposition(|e| x >= *e).unwrap_or(0)
}

fn empty_tiers() -> Vec<TierCurve> {
    let upper = |edges: &[f32], i: usize| edges.get(i + 1).copied().unwrap_or(f32::INFINITY);
    (0..TIER_EDGES.len())
        .map(|t| TierCurve {
            from_days: TIER_EDGES[t],
            to_days: upper(&TIER_EDGES, t),
            bins: (0..RATIO_EDGES.len())
                .map(|b| CurveBin {
                    from_ratio: RATIO_EDGES[b],
                    to_ratio: upper(&RATIO_EDGES, b),
                    count: RateCount::default(),
                })
                .collect(),
            total: RateCount::default(),
            rmse: None,
        })
        .collect()
}

fn count(report: &mut SurvivalReport, prev: &MemState, rating: Rating, t: f32, p: &Params) {
    let s = prev.stability.max(0.001);
    let (recalled, predicted) = (rating != Rating::Again, retrievability(prev, t, p));
    let (ratio_s, ratio_i) = (t / s, t / interval_exact(prev, p).max(1.0));
    let mature = s >= MATURE_DAYS;
    add(
        if mature {
            &mut report.mature
        } else {
            &mut report.young
        },
        recalled,
        predicted,
        ratio_s,
    );
    let timing = match ratio_i {
        r if r < 0.8 => &mut report.early,
        r if r <= 1.2 => &mut report.on_time,
        _ => &mut report.overdue,
    };
    add(timing, recalled, predicted, ratio_s);
    if mature && (0.8..=1.2).contains(&ratio_i) {
        add(&mut report.true_retention, recalled, predicted, ratio_s);
    }
    let tier = &mut report.tiers[edge_index(&TIER_EDGES, s)];
    add(&mut tier.total, recalled, predicted, ratio_s);
    let bin = &mut tier.bins[edge_index(&RATIO_EDGES, ratio_s)].count;
    add(bin, recalled, predicted, ratio_s);
}

fn rmse(bins: &[CurveBin]) -> Option<f32> {
    let (mut sq, mut n) = (0.0_f32, 0_u32);
    for b in bins.iter().filter(|b| b.count.reviews > 0) {
        let d = rate_of(&b.count)? - predicted_of(&b.count)?;
        sq += d * d * b.count.reviews as f32;
        n += b.count.reviews;
    }
    (n > 0).then(|| (sq / n as f32).sqrt())
}

/// Predicted recall at relative time `ratio = t / S` for memory that lasts `scale` times longer.
fn curve(ratio: f32, scale: f32, p: &Params) -> f32 {
    (1.0 + p.factor * ratio / scale).powf(p.decay)
}

/// The memory scale that best fits the measured recall of every bin (grid search, weighted by reviews).
pub fn fit_memory_scale(tiers: &[TierCurve], p: &Params) -> Option<f32> {
    let bins: Vec<(f32, f32, f32)> = tiers
        .iter()
        .flat_map(|t| t.bins.iter())
        .filter_map(|b| {
            let n = b.count.reviews as f32;
            Some((b.count.ratio_sum / n.max(1.0), rate_of(&b.count)?, n))
        })
        .collect();
    let total: f32 = bins.iter().map(|b| b.2).sum();
    if total < MIN_FIT_REVIEWS as f32 {
        return None;
    }
    let error = |scale: f32| -> f32 {
        bins.iter()
            .map(|(ratio, rate, n)| n * (curve(*ratio, scale, p) - rate).powi(2))
            .sum()
    };
    (0..=120)
        .map(|i| 0.25 * 1.0236_f32.powi(i))
        .min_by(|a, b| error(*a).total_cmp(&error(*b)))
}

/// Days until recall falls to one half for an item of stability `s`.
pub fn half_life_days(s: f32, p: &Params) -> f32 {
    s / p.factor * (0.5_f32.powf(1.0 / p.decay) - 1.0)
}

fn median(mut v: Vec<f32>) -> Option<f32> {
    v.sort_by(f32::total_cmp);
    v.get(v.len().checked_sub(1)? / 2).copied()
}

/// True retention, retention by maturity and timing, and the forgetting curves of every stability tier.
pub fn survival_report(reviews: &[SurvivalReview], p: &Params) -> SurvivalReport {
    let mut report = SurvivalReport {
        tiers: empty_tiers(),
        ..SurvivalReport::default()
    };
    let mut states: HashMap<u64, MemState> = HashMap::new();
    for r in reviews {
        let next = match states.get(&r.item) {
            None => initial_state(r.rating, p),
            Some(prev) => {
                if r.elapsed_days >= 1.0 {
                    count(&mut report, prev, r.rating, r.elapsed_days, p);
                }
                next_state(prev, r.rating, r.elapsed_days, p)
            }
        };
        states.insert(r.item, next);
    }
    for t in &mut report.tiers {
        t.rmse = rmse(&t.bins);
    }
    report.memory_scale = fit_memory_scale(&report.tiers, p);
    let typical = median(states.values().map(|s| s.stability).collect());
    report.model_half_life_days = typical.map(|s| half_life_days(s, p));
    report.half_life_days = report
        .memory_scale
        .zip(report.model_half_life_days)
        .map(|(m, h)| m * h);
    report
}
