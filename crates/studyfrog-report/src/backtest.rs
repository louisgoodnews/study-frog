//! Checking the forecast against what happened, and estimating its model from history. Pure.

use std::collections::HashMap;

use studyfrog_core::memory::MemState;
use studyfrog_core::review::Rating;
use studyfrog_srs::fsrs::{initial_state, interval_days, next_state};
use studyfrog_srs::params::Params;

use crate::forecast::{Model, ScheduledItem, forecast, model_default};

/// One review on a day (local days since 1970-01-01).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DatedReview {
    /// The item, as a number.
    pub item: u64,
    /// The day.
    pub day: i64,
    /// The rating.
    pub rating: Rating,
}

/// How far a forecast was from what happened.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BacktestError {
    /// Days compared.
    pub days: usize,
    /// Mean absolute difference per day, in reviews.
    pub mae_per_day: f32,
    /// Mean absolute error of the weekly totals, as a share of the actual totals.
    pub weekly_error: f32,
    /// Predicted total minus actual total, as a share of the actual total (positive: over-forecast).
    pub bias: f32,
}

/// The state of every item after replaying the reviews up to and including `cut`.
fn replay(reviews: &[DatedReview], cut: i64, p: &Params) -> HashMap<u64, (MemState, i64)> {
    let mut items: HashMap<u64, (MemState, i64)> = HashMap::new();
    for r in reviews.iter().filter(|r| r.day <= cut) {
        let next = match items.get(&r.item) {
            Some((s, last)) => (next_state(s, r.rating, (r.day - last) as f32, p), r.day),
            None => (initial_state(r.rating, p), r.day),
        };
        items.insert(r.item, next);
    }
    items
}

fn median(mut v: Vec<f32>) -> Option<f32> {
    v.sort_by(f32::total_cmp);
    v.get(v.len().checked_sub(1)? / 2).copied()
}

/// Estimate how reviews unfold from the history up to `cut`: how fast intervals grow, how often
/// reviews lapse, how long the first gap is, and how many new items start per day (last 14 days).
pub fn estimate_model(reviews: &[DatedReview], cut: i64) -> Model {
    let mut last: HashMap<u64, (i64, Option<f32>)> = HashMap::new(); // day, previous gap
    let (mut growth, mut first, mut lapses, mut total) = (Vec::new(), Vec::new(), 0_u32, 0_u32);
    let mut new_recent = 0_u32;
    for r in reviews.iter().filter(|r| r.day <= cut) {
        match last.get(&r.item).copied() {
            None => new_recent += u32::from(r.day > cut - 14),
            Some((day, prev_gap)) => {
                let gap = (r.day - day) as f32;
                total += 1;
                lapses += u32::from(r.rating == Rating::Again);
                if gap >= 1.0 && r.rating != Rating::Again {
                    match prev_gap {
                        Some(g) => growth.push(gap / g),
                        None => first.push(gap),
                    }
                }
                last.insert(r.item, (r.day, (r.rating != Rating::Again).then_some(gap)));
                continue;
            }
        }
        last.insert(r.item, (r.day, None));
    }
    let d = model_default();
    Model {
        growth: median(growth).unwrap_or(d.growth).clamp(1.0, 5.0),
        first_interval_days: median(first).unwrap_or(d.first_interval_days).max(1.0),
        lapse_rate: if total > 0 {
            lapses as f32 / total as f32
        } else {
            d.lapse_rate
        },
        new_per_day: new_recent as f32 / 14.0,
    }
}

/// The items that have a review scheduled after replaying up to `cut`.
pub fn scheduled_items(reviews: &[DatedReview], cut: i64, p: &Params) -> Vec<ScheduledItem> {
    replay(reviews, cut, p)
        .into_values()
        .map(|(state, last)| {
            let interval = interval_days(&state, p);
            ScheduledItem {
                due_day: last + i64::from(interval),
                interval_days: interval as f32,
            }
        })
        .collect()
}

/// Forecast the `horizon` days after `cut` from the history up to `cut`, and compare with the reviews
/// that really happened on those days. `None` without any history before `cut` or any review after it.
pub fn backtest(
    reviews: &[DatedReview],
    cut: i64,
    horizon: usize,
    p: &Params,
) -> Option<BacktestError> {
    let items = scheduled_items(reviews, cut, p);
    let model = estimate_model(reviews, cut);
    let predicted = forecast(&items, &model, cut + 1, horizon);
    let mut actual = vec![0_u32; horizon];
    for r in reviews.iter().filter(|r| r.day > cut) {
        if let Some(slot) = usize::try_from(r.day - cut - 1)
            .ok()
            .and_then(|i| actual.get_mut(i))
        {
            *slot += 1;
        }
    }
    (!items.is_empty() && actual.iter().any(|a| *a > 0)).then(|| compare(&predicted, &actual))
}

/// Compare a forecast with the real counts.
pub fn compare(predicted: &[f32], actual: &[u32]) -> BacktestError {
    let days = predicted.len().min(actual.len());
    let abs: f32 = (0..days)
        .map(|i| (predicted[i] - actual[i] as f32).abs())
        .sum();
    let weeks: Vec<(f32, f32)> = (0..days)
        .step_by(7)
        .map(|s| {
            let e = (s + 7).min(days);
            (
                predicted[s..e].iter().sum(),
                actual[s..e].iter().sum::<u32>() as f32,
            )
        })
        .collect();
    let weekly_abs: f32 = weeks.iter().map(|(p, a)| (p - a).abs()).sum();
    let (total_p, total_a): (f32, f32) = weeks
        .iter()
        .fold((0.0, 0.0), |(p, a), w| (p + w.0, a + w.1));
    let share = |x: f32| if total_a > 0.0 { x / total_a } else { 0.0 };
    BacktestError {
        days,
        mae_per_day: if days > 0 { abs / days as f32 } else { 0.0 },
        weekly_error: share(weekly_abs),
        bias: share(total_p - total_a),
    }
}
