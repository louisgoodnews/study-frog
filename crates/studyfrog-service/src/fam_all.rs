//! The familiarity of every item at once, for building queues.

use std::collections::HashMap;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::status::{Status, effective_status, status_from_name};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::{FamContext, fam_params_default, familiarity};
use studyfrog_store::error::StoreError;
use studyfrog_store::item_meta::load_item_meta;
use studyfrog_store::overview::{IndexedItem, load_overview};
use studyfrog_store::reviews::load_all_reviews_timed;
use studyfrog_store::views::load_all_views;

use crate::library_rows::{events_of, exams};

const DAY_MS: i64 = 86_400_000;

/// What the value of an item depends on besides its history.
pub(crate) fn fam_context(item: &IndexedItem, status: Status, exam: Option<UnixMs>) -> FamContext {
    FamContext {
        is_note: item.kind == ItemKind::Note,
        mark: item.difficulty,
        done: status == Status::Done,
        exam,
    }
}

/// The familiarity of each of `items` that has a history, at `now`.
pub fn item_familiarities(
    conn: &rusqlite::Connection,
    items: &[IndexedItem],
    now: UnixMs,
) -> Result<HashMap<Uid, f32>, StoreError> {
    let stacks = load_overview(conn)?.stacks;
    let by_folder = exams(conn, &stacks, now.0.div_euclid(DAY_MS))?;
    let (reviews, views) = (load_all_reviews_timed(conn)?, load_all_views(conn)?);
    let meta = load_item_meta(conn)?;
    let params = fam_params_default();
    let mut out = HashMap::new();
    for item in items {
        let Some(m) = meta.get(&item.uid) else {
            continue;
        };
        let status = effective_status(
            m.status.as_deref().and_then(status_from_name),
            m.reviews > 0,
        );
        let exam = m
            .stack
            .as_ref()
            .and_then(|f| by_folder.get(f))
            .map(|d| UnixMs(d * DAY_MS));
        let events = events_of(reviews.get(&item.uid), views.get(&item.uid));
        let ctx = fam_context(item, status, exam);
        if let Some(f) = familiarity(&events, now, &ctx, &params) {
            out.insert(item.uid, f.value);
        }
    }
    Ok(out)
}
