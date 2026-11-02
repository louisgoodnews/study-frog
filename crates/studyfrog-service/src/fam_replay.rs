//! Replaying the whole review history to see whether familiarity improves FSRS's predictions.

use studyfrog_core::status::{effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_report::replay::{ReplayItem, ReplayReport, replay};
use studyfrog_srs::params::Params;
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{load_indexed_items, load_overview};
use studyfrog_store::reviews::load_all_reviews_timed;
use studyfrog_store::views::load_all_views;

use crate::fam_all::fam_context;
use crate::library_rows::{events_of, exams};

const DAY_MS: i64 = 86_400_000;

/// Replay the history of every item that has been rated.
pub fn replay_history(
    conn: &rusqlite::Connection,
    now: UnixMs,
    params: &Params,
) -> Result<ReplayReport, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let by_folder = exams(conn, &stacks, now.0.div_euclid(DAY_MS))?;
    let (reviews, views) = (load_all_reviews_timed(conn)?, load_all_views(conn)?);
    let meta = load_item_meta(conn)?;
    let mut items = Vec::new();
    for item in load_indexed_items(conn)? {
        let (Some(m), Some(rated)) = (meta.get(&item.uid), reviews.get(&item.uid)) else {
            continue;
        };
        let status = effective_status(m.status.as_deref().and_then(status_from_name), true);
        let exam = m
            .stack
            .as_ref()
            .and_then(|f| by_folder.get(f))
            .map(|d| UnixMs(d * DAY_MS));
        items.push(ReplayItem {
            events: events_of(Some(rated), views.get(&item.uid)),
            ctx: fam_context(&item, status, exam),
            ratings: rated.iter().map(|(r, _)| (r.at, r.rating)).collect(),
        });
    }
    Ok(replay(&items, params))
}
