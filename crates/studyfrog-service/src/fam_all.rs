//! The familiarity of every item at once, for building queues.

use std::collections::HashMap;

use studyfrog_core::plan::ItemKind;
use studyfrog_core::status::Status;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_report::familiarity::FamContext;
use studyfrog_store::error::StoreError;
use studyfrog_store::overview::IndexedItem;

use crate::fam_refs::reference_scores;

/// What the value of an item depends on besides its history.
pub(crate) fn fam_context(item: &IndexedItem, status: Status, exam: Option<UnixMs>) -> FamContext {
    FamContext {
        is_note: item.kind == ItemKind::Note,
        mark: item.difficulty,
        done: status == Status::Done,
        exam,
    }
}

/// The familiarity of each (item, stack) of `items` that has a history, at `now`.
pub fn reference_familiarities(
    conn: &rusqlite::Connection,
    items: &[IndexedItem],
    now: UnixMs,
) -> Result<HashMap<(Uid, Uid), f32>, StoreError> {
    let wanted: std::collections::HashSet<Uid> = items.iter().map(|i| i.uid).collect();
    Ok(reference_scores(conn, now)?
        .into_iter()
        .filter(|s| wanted.contains(&s.uid))
        .filter_map(|s| s.fam.map(|f| ((s.uid, s.stack), f.value)))
        .collect())
}
