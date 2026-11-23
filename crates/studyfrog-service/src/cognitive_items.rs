//! The estimated difficulty of each item (EPIC-060, STORY-0177): the empirical difficulty index of
//! each of its timed answers, averaged over the item. Pure: the telemetry and titles are read by
//! the caller.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_report::difficulty::{Observation, empirical_difficulty};
use studyfrog_store::analytics_rows::ItemFact;
use studyfrog_store::telemetry::TelemetryPoint;

/// Items shown at most.
pub const MAX_ITEMS: usize = 20;

/// An item's estimated difficulty, with what the screen shows of it.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemDifficulty {
    /// The item.
    pub item: Uid,
    /// Its title (empty when the item is no longer in the index).
    pub title: String,
    /// The folder of its stack (`""` for a loose item).
    pub folder: String,
    /// Mean empirical difficulty over its answers, on `[1, 10]`.
    pub difficulty: f64,
    /// Answers the mean is over.
    pub answers: usize,
}

/// The engine's observation for one answer.
fn observation(p: &TelemetryPoint) -> Observation {
    Observation {
        latency_ms: f64::from(p.latency_ms),
        dwell_ms: f64::from(p.dwell_ms),
        words: f64::from(p.text_complexity),
        semantic: p.semantic_score.map(f64::from),
    }
}

/// Per item: the sum of its answers' difficulties and their count.
fn sums(telemetry: &[TelemetryPoint]) -> HashMap<Uid, (f64, usize)> {
    let mut by: HashMap<Uid, (f64, usize)> = HashMap::new();
    for p in telemetry {
        let e = by.entry(p.item).or_insert((0.0, 0));
        e.0 += empirical_difficulty(&observation(p));
        e.1 += 1;
    }
    by
}

/// Title and folder of every indexed item, by uid.
pub fn titles_of(facts: &[ItemFact]) -> HashMap<Uid, (String, String)> {
    facts
        .iter()
        .map(|f| {
            (
                f.uid,
                (f.title.clone(), f.stack_path.clone().unwrap_or_default()),
            )
        })
        .collect()
}

/// The items by estimated difficulty, hardest first (ties by title), at most [`MAX_ITEMS`].
pub fn item_difficulties(
    telemetry: &[TelemetryPoint],
    titles: &HashMap<Uid, (String, String)>,
) -> Vec<ItemDifficulty> {
    let mut out: Vec<ItemDifficulty> = sums(telemetry)
        .into_iter()
        .map(|(item, (sum, answers))| {
            let (title, folder) = titles.get(&item).cloned().unwrap_or_default();
            ItemDifficulty {
                item,
                title,
                folder,
                difficulty: sum / answers as f64,
                answers,
            }
        })
        .collect();
    out.sort_by(|a, b| {
        b.difficulty
            .total_cmp(&a.difficulty)
            .then_with(|| a.title.cmp(&b.title))
    });
    out.truncate(MAX_ITEMS);
    out
}
