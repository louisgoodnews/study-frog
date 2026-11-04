//! Replaying the whole review history to see whether familiarity improves FSRS's predictions.
//! Each stack's history of an item is replayed on its own, with that stack's exam (ADR 0011).

use studyfrog_core::time::UnixMs;
use studyfrog_report::familiarity::FamEvent;
use studyfrog_report::replay::{ReplayItem, ReplayReport, replay};
use studyfrog_srs::params::Params;
use studyfrog_store::error::StoreError;

use crate::fam_refs::reference_scores;

/// Replay the history every stack has of every item that has been rated there.
pub fn replay_history(
    conn: &rusqlite::Connection,
    now: UnixMs,
    params: &Params,
) -> Result<ReplayReport, StoreError> {
    let items: Vec<ReplayItem> = reference_scores(conn, now)?
        .into_iter()
        .filter(|s| s.reviews > 0)
        .map(|s| ReplayItem {
            ratings: s
                .events
                .iter()
                .filter_map(|e| match e {
                    FamEvent::Rating { at, rating, .. } => Some((*at, *rating)),
                    _ => None,
                })
                .collect(),
            events: s.events,
            ctx: s.ctx,
        })
        .collect();
    Ok(replay(&items, params))
}
