//! Recorded views of items: how long the item page was open and active.

use rusqlite::{Connection, params};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_core::views::{ViewEvent, view_kind_from_name, view_kind_name};
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

/// Append a view to the history.
pub fn insert_view(conn: &Connection, e: &ViewEvent) -> Result<(), StoreError> {
    conn.prepare_cached(
        "INSERT INTO item_view (item_uid, kind, at_ms, active_ms, expected_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .and_then(|mut s| {
        s.execute(params![
            format_ulid(e.item),
            view_kind_name(e.kind),
            e.at.0,
            e.active_ms,
            e.expected_ms
        ])
    })
    .map(drop)
    .map_err(sql_err)
}

/// The recorded views of an item, oldest first.
pub fn load_views(conn: &Connection, item: Uid) -> Result<Vec<ViewEvent>, StoreError> {
    let mut stmt = conn
        .prepare(
            "SELECT kind, at_ms, active_ms, expected_ms FROM item_view WHERE item_uid = ?1 ORDER BY at_ms, id",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(item)], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, u32>(2)?,
                r.get::<_, Option<u32>>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (kind, at, active_ms, expected_ms) = row.map_err(sql_err)?;
        let kind = view_kind_from_name(&kind)
            .ok_or_else(|| StoreError::Corrupt(format!("view kind: {kind:?}")))?;
        out.push(ViewEvent {
            item,
            kind,
            at: UnixMs(at),
            active_ms,
            expected_ms,
        });
    }
    Ok(out)
}

/// Every uid that has recorded views (for tests and reports).
pub fn viewed_items(conn: &Connection) -> Result<Vec<Uid>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT DISTINCT item_uid FROM item_view ORDER BY item_uid")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let id = row.map_err(sql_err)?;
        out.push(parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?);
    }
    Ok(out)
}

/// The recorded views of every item, oldest first, by item.
pub fn load_all_views(
    conn: &Connection,
) -> Result<std::collections::HashMap<Uid, Vec<ViewEvent>>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT item_uid, kind, at_ms, active_ms, expected_ms FROM item_view ORDER BY item_uid, at_ms, id")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, u32>(3)?,
                r.get::<_, Option<u32>>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out: std::collections::HashMap<Uid, Vec<ViewEvent>> = std::collections::HashMap::new();
    for row in rows {
        let (id, kind, at, active_ms, expected_ms) = row.map_err(sql_err)?;
        let item = parse_ulid(&id).ok_or_else(|| StoreError::Corrupt(format!("ulid: {id:?}")))?;
        let kind = view_kind_from_name(&kind)
            .ok_or_else(|| StoreError::Corrupt(format!("view kind: {kind:?}")))?;
        out.entry(item).or_default().push(ViewEvent {
            item,
            kind,
            at: UnixMs(at),
            active_ms,
            expected_ms,
        });
    }
    Ok(out)
}
