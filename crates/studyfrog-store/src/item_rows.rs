//! Reading item rows: the columns every item list selects and how a row becomes an [`ItemRow`].

use studyfrog_core::status::{effective_status, status_from_name};
use studyfrog_md::ulid::parse_ulid;

use crate::error::StoreError;
use crate::items::ItemRow;

/// The columns every item list selects, in the order `raw_item` reads them.
pub(crate) const ITEM_COLS: &str = "i.uid, i.title, i.kind, i.path, COALESCE(m.due_ms, s.due_ms) AS due, i.status,\n    EXISTS(SELECT 1 FROM review_once r WHERE r.item_uid = i.uid)";

pub(crate) type RawItem = (
    String,
    String,
    String,
    String,
    Option<i64>,
    Option<String>,
    bool,
);

pub(crate) fn raw_item(r: &rusqlite::Row<'_>) -> rusqlite::Result<RawItem> {
    Ok((
        r.get(0)?,
        r.get(1)?,
        r.get(2)?,
        r.get(3)?,
        r.get(4)?,
        r.get(5)?,
        r.get(6)?,
    ))
}

pub(crate) fn item_row(
    (id, title, kind, path, due, status, reviewed): RawItem,
) -> Result<ItemRow, StoreError> {
    let uid = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
    let kind = studyfrog_core::plan::item_kind_from_name(&kind)
        .ok_or_else(|| StoreError::Corrupt(format!("kind: {kind:?}")))?;
    Ok(ItemRow {
        uid,
        title,
        kind,
        path,
        due: due.map(studyfrog_core::time::UnixMs),
        status: effective_status(status.as_deref().and_then(status_from_name), reviewed),
    })
}
