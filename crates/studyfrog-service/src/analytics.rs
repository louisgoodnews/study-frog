//! The behavioural analytics of the reports (EPIC-034): true retention and forgetting curves, the hour and
//! weekday profile and within-session fatigue, read from the review history and the run logs and computed by the
//! pure functions of `studyfrog-report`.

use std::collections::HashMap;

use studyfrog_core::event::RunEvent;
use studyfrog_core::uid::Uid;
use studyfrog_report::chronotype::{
    ChronotypeReport, MIN_BUCKET_REVIEWS, TimedLatency, TimedReview, chronotype,
};
use studyfrog_report::fatigue::{FatigueReport, fatigue_report};
use studyfrog_report::retention_survival::{SurvivalReport, SurvivalReview, survival_report};
use studyfrog_report::session_log::{SessionSample, session_samples};
use studyfrog_srs::params::Params;
use studyfrog_store::error::StoreError;
use studyfrog_store::reports::{HistoryRow, load_run_starts};

use crate::session::VaultSession;

/// Everything EPIC-034 adds to the reports.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Analytics {
    /// True retention, retention by maturity and timing, forgetting curves by stability tier.
    pub survival: SurvivalReport,
    /// Recall and answer time by local hour and weekday.
    pub chronotype: ChronotypeReport,
    /// Recall and answer time by minutes into a run and position in it.
    pub fatigue: FatigueReport,
}

/// What the analytics are computed from.
#[derive(Clone, Copy)]
pub struct History<'a> {
    /// Every review once, oldest first.
    pub rows: &'a [HistoryRow],
    /// Every run event, grouped by run and in order.
    pub events: &'a [RunEvent],
    /// The local UTC offset at an instant (Unix milliseconds).
    pub offset_of: &'a dyn Fn(i64) -> i32,
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

/// The hour and weekday profile: reviews by their own local time, answer times placed by the start of their run.
pub fn chronotype_of(
    h: &History,
    samples: &[SessionSample],
    starts: &HashMap<Uid, i64>,
) -> ChronotypeReport {
    // The offset is asked once per hour of history: a time-zone lookup per review would be slow.
    let mut cache: HashMap<i64, i32> = HashMap::new();
    let mut offset = |at: i64| {
        *cache
            .entry(at.div_euclid(3_600_000))
            .or_insert_with(|| (h.offset_of)(at))
    };
    let reviews: Vec<TimedReview> = h
        .rows
        .iter()
        .map(|r| TimedReview {
            at_ms: r.at_ms,
            offset_minutes: offset(r.at_ms),
            rating: r.rating,
        })
        .collect();
    let latencies: Vec<TimedLatency> = samples
        .iter()
        .filter_map(|s| {
            let at_ms = starts.get(&s.run)? + i64::from(s.shown_t_ms);
            Some(TimedLatency {
                at_ms,
                offset_minutes: offset(at_ms),
                ms: s.latency_ms?,
            })
        })
        .collect();
    chronotype(&reviews, &latencies, MIN_BUCKET_REVIEWS)
}

/// All analytics of the history.
pub fn analytics_of(
    session: &VaultSession,
    h: History,
    p: &Params,
) -> Result<Analytics, StoreError> {
    let ids = item_numbers(h.rows);
    let samples = session_samples(h.events);
    let starts = load_run_starts(&session.conn)?;
    Ok(Analytics {
        survival: survival_of(h.rows, &ids, p),
        chronotype: chronotype_of(&h, &samples, &starts),
        fatigue: fatigue_report(&samples),
    })
}
