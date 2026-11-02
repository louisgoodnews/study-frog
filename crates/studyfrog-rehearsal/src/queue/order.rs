//! Sequencing of admitted candidates. Every sort is total and deterministic: ties fall back to the
//! library position, then the uid.

use std::cmp::Ordering;

use studyfrog_core::event::Difficulty;
use studyfrog_core::plan::Order;
use studyfrog_core::time::UnixMs;
use studyfrog_srs::fsrs::retrievability;
use studyfrog_srs::params::Params;

use crate::queue::candidate::Candidate;

const MS_PER_DAY: f32 = 86_400_000.0;

fn base(a: &Candidate, b: &Candidate) -> Ordering {
    a.position.cmp(&b.position).then(a.uid.cmp(&b.uid))
}

/// The more important first, then the library position.
fn by_priority(a: &Candidate, b: &Candidate) -> Ordering {
    b.priority.cmp(&a.priority).then_with(|| base(a, b))
}

/// Items with a key first (by `cmp`), keyless items after them in base order.
fn keyed<T>(a: Option<T>, b: Option<T>, cmp: impl Fn(&T, &T) -> Ordering) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => cmp(&x, &y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// The memory difficulty, or for an item never reviewed what its owner said (on the same 1 to 10
/// scale), if anything.
fn difficulty(c: &Candidate) -> Option<f32> {
    c.memory
        .map(|m| m.difficulty)
        .or(c.difficulty_hint.map(|d| match d {
            Difficulty::Easy => 2.5,
            Difficulty::Medium => 5.5,
            Difficulty::Difficult => 8.5,
        }))
}

/// Predicted recall right now; `None` for items without FSRS state.
fn recall(c: &Candidate, now: UnixMs, p: &Params) -> Option<f32> {
    let memory = c.memory?;
    let elapsed = c
        .last_review
        .map_or(0.0, |t| (now.0 - t.0) as f32 / MS_PER_DAY);
    Some(retrievability(&memory, elapsed, p))
}

/// Sort `items` into the sequence the order asks for. `Order::Due` is handled by `spaced`.
pub fn sort_by_order(items: &mut [&Candidate], order: Order, now: UnixMs, p: &Params) {
    items.sort_by(|a, b| base(a, b));
    match order {
        Order::Due | Order::Sequential => {}
        Order::HardestFirst => {
            items.sort_by(|a, b| {
                keyed(difficulty(a), difficulty(b), |x, y| y.total_cmp(x))
                    .then_with(|| by_priority(a, b))
            });
        }
        Order::EasiestFirst => {
            items.sort_by(|a, b| {
                keyed(difficulty(a), difficulty(b), f32::total_cmp).then_with(|| by_priority(a, b))
            });
        }
        Order::StalestFirst => items.sort_by(|a, b| {
            a.last_review
                .cmp(&b.last_review)
                .then_with(|| by_priority(a, b))
        }),
        Order::LeastFamiliar => items.sort_by(|a, b| {
            let (fa, fb) = (a.familiarity.unwrap_or(0.0), b.familiarity.unwrap_or(0.0));
            fa.total_cmp(&fb).then_with(|| by_priority(a, b))
        }),
        Order::LowestRetrievability => {
            // The recall of an item costs a `powf`; compute it once per item, not once per comparison.
            let mut keyed_items: Vec<(Option<f32>, &Candidate)> =
                items.iter().map(|c| (recall(c, now, p), *c)).collect();
            keyed_items.sort_by(|(ka, a), (kb, b)| {
                keyed(*ka, *kb, f32::total_cmp).then_with(|| by_priority(a, b))
            });
            for (slot, (_, c)) in items.iter_mut().zip(keyed_items) {
                *slot = c;
            }
        }
    }
}
