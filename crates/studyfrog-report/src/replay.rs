//! Replaying a review history to tell whether familiarity improves FSRS's predictions of recall
//! (`docs/research/familiarity-and-fsrs.md`, section 7). Pure: the caller supplies each item's events.
//!
//! At every review after an item's first, the plain prediction is FSRS's retrievability `R(t, S)`; the adjusted one is
//! `R(t, S * m)` with the factor `m` of the item's familiarity just before the review. Both are scored against whether
//! the user recalled (any rating but Again) by log loss and a table of calibration bins.

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_srs::familiarity::{FactorParams, factor_params_default, familiarity_factor};
use studyfrog_srs::fsrs::{initial_state, next_state, retrievability};
use studyfrog_srs::params::Params;

use crate::familiarity::{FamContext, FamEvent, familiarity};
use crate::familiarity_types::fam_params_default;

const DAY_MS: i64 = 86_400_000;
/// Reviews needed before values are fitted and a verdict is given.
pub const MIN_REVIEWS: usize = 1000;
/// Share of the history (the earlier part) the fit uses.
const TRAIN_SHARE: f64 = 0.8;
/// A difference in log loss smaller than this is no difference.
const MARGIN: f64 = 0.002;
const STRENGTHS: [f32; 7] = [0.0, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0];
const HALVES: [f32; 5] = [10.0, 20.0, 30.0, 60.0, 120.0];
const BINS: usize = 10;

/// One item's history.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayItem {
    /// Everything that happened to it, ratings included, in any order.
    pub events: Vec<FamEvent>,
    /// What else its value depends on.
    pub ctx: FamContext,
    /// Its ratings with their times.
    pub ratings: Vec<(UnixMs, Rating)>,
}

/// What the replay concludes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Fewer than [`MIN_REVIEWS`] reviews: scored with the defaults, nothing fitted.
    TooFewReviews,
    /// The familiarity-adjusted predictions have the lower log loss on the later part.
    FactorHelps,
    /// Plain FSRS predicts better on the later part: the factor may be switched off.
    PlainBetter,
    /// No clear difference.
    Similar,
}

/// One bin of the calibration table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bin {
    /// Reviews in the bin (by plain prediction).
    pub reviews: u32,
    /// Mean plain prediction.
    pub plain: f32,
    /// Mean adjusted prediction.
    pub adjusted: f32,
    /// Share actually recalled.
    pub recalled: f32,
}

/// The result of a replay.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayReport {
    /// Reviews scored (every review after an item's first).
    pub scored: usize,
    /// The verdict.
    pub verdict: Verdict,
    /// The strength and half-stability used for the adjusted predictions (fitted, or the defaults).
    pub used: FactorParams,
    /// Log loss of plain FSRS and of the adjusted prediction, on the tested part (all of it without a fit).
    pub loss: (f64, f64),
    /// Calibration of the tested part.
    pub bins: Vec<Bin>,
    /// Change in reviews for the same history with the factor, in percent (negative: fewer reviews).
    pub workload_percent: f32,
}

struct Record {
    at: UnixMs,
    recalled: bool,
    plain: f32,
    state: MemState,
    elapsed: f32,
    fam: f32,
    reviews: u32,
}

fn records(items: &[ReplayItem], params: &Params) -> Vec<Record> {
    let fp = fam_params_default();
    let mut out = Vec::new();
    for item in items {
        let mut events = item.events.clone();
        events.sort_by_key(event_time);
        let mut state: Option<(MemState, UnixMs)> = None;
        let mut ratings = item.ratings.clone();
        ratings.sort_by_key(|(t, _)| *t);
        for (k, (at, rating)) in ratings.iter().enumerate() {
            let next = match &state {
                None => initial_state(*rating, params),
                Some((s, last)) => {
                    let elapsed = ((at.0 - last.0).max(0) / DAY_MS) as f32;
                    let before = events.partition_point(|e| event_time(e) < *at);
                    let fam = familiarity(&events[..before], *at, &item.ctx, &fp)
                        .map_or(0.0, |f| f.value);
                    out.push(Record {
                        at: *at,
                        recalled: *rating != Rating::Again,
                        plain: retrievability(s, elapsed, params),
                        state: *s,
                        elapsed,
                        fam,
                        reviews: k as u32,
                    });
                    next_state(s, *rating, elapsed, params)
                }
            };
            state = Some((next, *at));
        }
    }
    out.sort_by_key(|r| r.at);
    out
}

fn event_time(e: &FamEvent) -> UnixMs {
    match e {
        FamEvent::Rating { at, .. } | FamEvent::View { at, .. } | FamEvent::Edit { at } => *at,
    }
}

fn factor(r: &Record, p: &FactorParams) -> f32 {
    familiarity_factor(r.fam, r.state.stability, r.reviews, p)
}

fn adjusted(r: &Record, p: &FactorParams, params: &Params) -> f32 {
    let state = MemState {
        stability: r.state.stability * factor(r, p),
        ..r.state
    };
    retrievability(&state, r.elapsed, params)
}

fn loss(list: &[Record], predict: impl Fn(&Record) -> f32) -> f64 {
    if list.is_empty() {
        return 0.0;
    }
    let sum: f64 = list
        .iter()
        .map(|r| {
            let p = f64::from(predict(r)).clamp(1e-4, 1.0 - 1e-4);
            if r.recalled { -p.ln() } else { -(1.0 - p).ln() }
        })
        .sum();
    sum / list.len() as f64
}

fn fit(train: &[Record], params: &Params) -> FactorParams {
    let base = factor_params_default();
    let mut best = (f64::MAX, base);
    for strength in STRENGTHS {
        for s_half in HALVES {
            let p = FactorParams {
                strength,
                s_half,
                ..base
            };
            let l = loss(train, |r| adjusted(r, &p, params));
            if l < best.0 {
                best = (l, p);
            }
        }
    }
    best.1
}

fn calibration(list: &[Record], p: &FactorParams, params: &Params) -> Vec<Bin> {
    let mut sums = [(0_u32, 0.0_f64, 0.0_f64, 0.0_f64); BINS];
    for r in list {
        let i = ((r.plain * BINS as f32) as usize).min(BINS - 1);
        let s = &mut sums[i];
        s.0 += 1;
        s.1 += f64::from(r.plain);
        s.2 += f64::from(adjusted(r, p, params));
        s.3 += f64::from(u8::from(r.recalled));
    }
    sums.iter()
        .filter(|s| s.0 > 0)
        .map(|s| {
            let n = f64::from(s.0);
            Bin {
                reviews: s.0,
                plain: (s.1 / n) as f32,
                adjusted: (s.2 / n) as f32,
                recalled: (s.3 / n) as f32,
            }
        })
        .collect()
}

/// Replay the histories of `items` with FSRS `params`.
pub fn replay(items: &[ReplayItem], params: &Params) -> ReplayReport {
    let all = records(items, params);
    let enough = all.len() >= MIN_REVIEWS;
    let cut = if enough {
        (all.len() as f64 * TRAIN_SHARE) as usize
    } else {
        0
    };
    let used = if enough {
        fit(&all[..cut], params)
    } else {
        factor_params_default()
    };
    let tested = &all[cut..];
    let (plain, adj) = (
        loss(tested, |r| r.plain),
        loss(tested, |r| adjusted(r, &used, params)),
    );
    let verdict = match (enough, adj - plain) {
        (false, _) => Verdict::TooFewReviews,
        (true, d) if d < -MARGIN => Verdict::FactorHelps,
        (true, d) if d > MARGIN => Verdict::PlainBetter,
        _ => Verdict::Similar,
    };
    let workload = if all.is_empty() {
        0.0
    } else {
        all.iter()
            .map(|r| 1.0 / f64::from(factor(r, &used)))
            .sum::<f64>()
            / all.len() as f64
            - 1.0
    };
    ReplayReport {
        scored: all.len(),
        verdict,
        used,
        loss: (plain, adj),
        bins: calibration(tested, &used, params),
        workload_percent: (workload * 100.0) as f32,
    }
}
