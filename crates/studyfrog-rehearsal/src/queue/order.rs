//! Sequencing of admitted candidates. Every sort is total and deterministic: ties fall back to the
//! library position, then the uid.

use std::cmp::Ordering;

use studyfrog_core::plan::Order;
use studyfrog_core::time::UnixMs;
use studyfrog_srs::fsrs::retrievability;
use studyfrog_srs::params::Params;

use crate::queue::candidate::Candidate;
use crate::queue::shuffle::shuffle;

const MS_PER_DAY: f32 = 86_400_000.0;

fn base(a: &Candidate, b: &Candidate) -> Ordering {
    a.position.cmp(&b.position).then(a.uid.cmp(&b.uid))
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

fn difficulty(c: &Candidate) -> Option<f32> {
    c.memory.map(|m| m.difficulty)
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
pub fn sort_by_order(items: &mut [&Candidate], order: Order, seed: u64, now: UnixMs, p: &Params) {
    items.sort_by(|a, b| base(a, b));
    match order {
        Order::Due | Order::Sequential => {}
        Order::Shuffled => shuffle(items, seed),
        Order::HardestFirst => {
            items.sort_by(|a, b| {
                keyed(difficulty(a), difficulty(b), |x, y| y.total_cmp(x)).then_with(|| base(a, b))
            });
        }
        Order::EasiestFirst => {
            items.sort_by(|a, b| {
                keyed(difficulty(a), difficulty(b), f32::total_cmp).then_with(|| base(a, b))
            });
        }
        Order::StalestFirst => {
            items.sort_by(|a, b| a.last_review.cmp(&b.last_review).then_with(|| base(a, b)))
        }
        Order::LowestRetrievability => {
            items.sort_by(|a, b| {
                keyed(recall(a, now, p), recall(b, now, p), f32::total_cmp).then_with(|| base(a, b))
            });
        }
    }
}
