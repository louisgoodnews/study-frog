//! The behavioural analytics of the reports (EPIC-034): true retention and forgetting curves, read from
//! the review history and computed by the pure functions of `studyfrog-report`.

use std::collections::HashMap;

use studyfrog_core::uid::Uid;
use studyfrog_report::retention_survival::{SurvivalReport, SurvivalReview, survival_report};
use studyfrog_srs::params::Params;
use studyfrog_store::reports::HistoryRow;

/// Everything EPIC-034 adds to the reports.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Analytics {
    /// True retention, retention by maturity and timing, forgetting curves by stability tier.
    pub survival: SurvivalReport,
}

/// A small number for every item, in order of first appearance.
pub fn item_numbers(rows: &[HistoryRow]) -> HashMap<Uid, u64> {
    let mut ids: HashMap<Uid, u64> = HashMap::new();
    for r in rows {
        let next = ids.len() as u64;
        ids.entry(r.item).or_insert(next);
    }
    ids
}

/// The survival report of the history (every review once, oldest first).
pub fn survival_of(rows: &[HistoryRow], ids: &HashMap<Uid, u64>, p: &Params) -> SurvivalReport {
    let reviews: Vec<SurvivalReview> = rows
        .iter()
        .map(|r| SurvivalReview {
            item: ids.get(&r.item).copied().unwrap_or_default(),
            rating: r.rating,
            elapsed_days: r.elapsed_days,
        })
        .collect();
    survival_report(&reviews, p)
}

/// All analytics of the history.
pub fn analytics_of(rows: &[HistoryRow], p: &Params) -> Analytics {
    let ids = item_numbers(rows);
    Analytics {
        survival: survival_of(rows, &ids, p),
    }
}
