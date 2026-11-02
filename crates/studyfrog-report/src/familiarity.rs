//! Familiarity: how well the user knows an item, from 0.0 to 1.0 (`docs/specs/familiarity.md`). Pure: the history
//! and the clock are arguments, nothing is stored.

use studyfrog_core::event::Difficulty;
use studyfrog_core::priority::Priority;
use studyfrog_core::review::{Rating, rating_value};
use studyfrog_core::time::UnixMs;

const MS_PER_DAY: f64 = 86_400_000.0;

pub use crate::familiarity_types::{
    FamContext, FamEvent, FamParams, Familiarity, fam_params_default,
};

fn at_of(e: &FamEvent) -> UnixMs {
    match e {
        FamEvent::Rating { at, .. } | FamEvent::View { at, .. } | FamEvent::Edit { at } => *at,
    }
}

/// A total order for events: by time, then by kind and size, so that events at the same instant give the same
/// result whatever order they were stored in.
fn sort_key(e: &FamEvent) -> (i64, u8, u32, u32) {
    match *e {
        FamEvent::Edit { at } => (at.0, 0, 0, 0),
        FamEvent::Rating {
            at,
            rating,
            answer_ms,
            ..
        } => (
            at.0,
            1,
            u32::from(rating_value(rating)),
            answer_ms.unwrap_or(0),
        ),
        FamEvent::View { at, active_ms, .. } => (at.0, 2, active_ms, 0),
    }
}

fn decay(f: f32, (from, to): (UnixMs, UnixMs), longest_gap_days: f64, p: &FamParams) -> f32 {
    let days = (to.0 - from.0).max(0) as f64 / MS_PER_DAY;
    let half_life = f64::from(p.half_life_days).max(f64::from(p.gap_factor) * longest_gap_days);
    f * 0.5_f64.powf(days / half_life) as f32
}

/// `expected / actual`, kept between the bounds; 1 when either time is missing.
fn speed(actual: Option<u32>, expected: Option<u32>, (lo, hi): (f32, f32)) -> f32 {
    match (actual, expected) {
        (Some(a), Some(e)) if a > 0 => (e as f32 / a as f32).clamp(lo, hi),
        _ => 1.0,
    }
}

fn mark_factor(mark: Option<Difficulty>, p: &FamParams) -> f32 {
    match mark {
        Some(Difficulty::Easy) => p.mark_factors.0,
        Some(Difficulty::Difficult) => p.mark_factors.1,
        _ => 1.0,
    }
}

fn rated(f: f32, e: (Rating, f32, f32), p: &FamParams) -> f32 {
    let (rating, s, m) = e;
    match rating {
        Rating::Easy => f + p.gain_easy * (1.0 - f) * s * m,
        Rating::Good => f + p.gain_good * (1.0 - f) * s * m,
        Rating::Hard => f - p.loss_hard * f,
        Rating::Again => f - p.loss_again * f,
    }
}

/// The value and confidence after the whole history, at `now`. `None` when there is no history.
pub fn familiarity(
    events: &[FamEvent],
    now: UnixMs,
    ctx: &FamContext,
    p: &FamParams,
) -> Option<Familiarity> {
    let mut sorted: Vec<&FamEvent> = events.iter().collect();
    sorted.sort_by_key(|e| sort_key(e));
    let first = at_of(sorted.first()?);
    let (mut f, mut last) = (0.0_f32, first);
    let (mut ratings, mut views, mut last_view) = (0_u32, 0_u32, None::<UnixMs>);
    let (mut longest_gap, mut last_rating) = (0.0_f64, None::<UnixMs>);
    let m = mark_factor(ctx.mark, p);
    for e in sorted {
        f = decay(f, (last, at_of(e)), longest_gap, p);
        last = at_of(e);
        match *e {
            FamEvent::Rating {
                rating,
                answer_ms,
                expected_ms,
                ..
            } => {
                ratings += 1;
                if let Some(prev) = last_rating {
                    longest_gap = longest_gap.max((at_of(e).0 - prev.0) as f64 / MS_PER_DAY);
                }
                last_rating = Some(at_of(e));
                f = rated(
                    f,
                    (rating, speed(answer_ms, expected_ms, p.speed_bounds), m),
                    p,
                );
            }
            FamEvent::View {
                at,
                active_ms,
                expected_ms,
            } => {
                let apart = last_view.is_none_or(|v| at.0 - v.0 >= p.view_gap_ms);
                if active_ms >= p.min_view_ms && apart {
                    let bounds = (p.view_speed_min, p.speed_bounds.1);
                    f += p.view_gain * (1.0 - f) * speed(Some(active_ms), expected_ms, bounds);
                    (views, last_view) = (views + 1, Some(at));
                }
            }
            FamEvent::Edit { .. } => {}
        }
    }
    let end = match (ctx.done, ctx.exam) {
        (true, _) => last,
        (false, Some(x)) => UnixMs(now.0.min(x.0.max(last.0))),
        (false, None) => now,
    };
    f = decay(f, (last, end), longest_gap, p);
    let confidence = if ctx.is_note {
        (views as f32 / p.full_confidence.1).min(0.5)
    } else {
        (ratings as f32 / p.full_confidence.0).min(1.0)
    };
    let value = if ctx.is_note { f.min(p.note_cap) } else { f };
    Some(Familiarity {
        value: value.clamp(0.0, 1.0),
        confidence,
    })
}

/// The weight of an item in its stack's roll-up.
fn weight(priority: Priority) -> f32 {
    match priority {
        Priority::Lowest => 0.5,
        Priority::Low => 0.75,
        Priority::Medium => 1.0,
        Priority::High => 1.5,
        Priority::Highest => 2.0,
    }
}

/// The stack's value: the mean of its items weighted by priority; `None` for no items.
pub fn stack_familiarity(items: &[(Priority, Familiarity)]) -> Option<Familiarity> {
    let total: f32 = items.iter().map(|(p, _)| weight(*p)).sum();
    if items.is_empty() || total <= 0.0 {
        return None;
    }
    let mean = |pick: fn(&Familiarity) -> f32| {
        items.iter().map(|(p, f)| weight(*p) * pick(f)).sum::<f32>() / total
    };
    Some(Familiarity {
        value: mean(|f| f.value),
        confidence: mean(|f| f.confidence),
    })
}
