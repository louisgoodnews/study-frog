//! Which stacks a rating counts for: the stacks that hold an item, the stacks a run was set up over, and a stack's
//! own scheduling settings (ADR 0011).

use rusqlite::Connection;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};
use crate::settings::{ItemHome, home_of_row};

fn uids(conn: &Connection, sql: &str, key: Uid) -> Result<Vec<Uid>, StoreError> {
    let mut stmt = conn.prepare_cached(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([format_ulid(key)], |r| r.get::<_, String>(0))
        .map_err(sql_err)?;
    rows.map(|r| {
        let text = r.map_err(sql_err)?;
        parse_ulid(&text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
    })
    .collect()
}

/// The stacks that hold the item, its primary stack first; the Inbox (uid 0) for a loose item.
pub fn holders_of(conn: &Connection, item: Uid) -> Result<Vec<Uid>, StoreError> {
    let sql =
        "SELECT stack_uid FROM membership WHERE item_uid = ?1 ORDER BY is_primary DESC, stack_uid";
    let found = uids(conn, sql, item)?;
    Ok(if found.is_empty() {
        vec![Uid(0)]
    } else {
        found
    })
}

/// The stacks a run was set up over; empty when it covers everything.
pub fn run_selection(conn: &Connection, run: Uid) -> Result<Vec<Uid>, StoreError> {
    uids(
        conn,
        "SELECT stack_uid FROM run_filter_stack WHERE run_uid = ?1 ORDER BY pos",
        run,
    )
}

/// The stacks in which a rating of `item` in `run` counts: every stack that holds the item and is in the run's
/// selection, or every stack that holds it when the run covers everything.
pub fn rated_stacks(conn: &Connection, run: Uid, item: Uid) -> Result<Vec<Uid>, StoreError> {
    let holders = holders_of(conn, item)?;
    let selected = run_selection(conn, run)?;
    if selected.is_empty() {
        return Ok(holders);
    }
    let both: Vec<Uid> = holders
        .iter()
        .copied()
        .filter(|s| selected.contains(s))
        .collect();
    Ok(if both.is_empty() { holders } else { both })
}

/// The scheduling settings of one stack, `None` for the Inbox and for a stack that does not exist.
pub fn stack_home(conn: &Connection, stack: Uid) -> Result<Option<ItemHome>, StoreError> {
    let sql = "SELECT s.uid, s.scheduler, s.request_retention, s.familiarity FROM stack_setting s WHERE s.uid = ?1";
    let found = conn.query_row(sql, [format_ulid(stack)], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    });
    match found {
        Ok(row) => home_of_row(row).map(Some),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// The stacks that have reviews of the item, oldest stack row first.
pub fn review_stacks(conn: &Connection, item: Uid) -> Result<Vec<Uid>, StoreError> {
    uids(
        conn,
        "SELECT stack_uid FROM review WHERE item_uid = ?1 GROUP BY stack_uid ORDER BY MIN(id)",
        item,
    )
}

/// The scheduling settings of every stack that has a settings row.
pub fn stack_homes(
    conn: &Connection,
) -> Result<std::collections::HashMap<Uid, ItemHome>, StoreError> {
    let sql = "SELECT s.uid, s.scheduler, s.request_retention, s.familiarity FROM stack_setting s";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .map_err(sql_err)?;
    let mut homes = std::collections::HashMap::new();
    for row in rows {
        let home = home_of_row(row.map_err(sql_err)?)?;
        homes.insert(home.stack, home);
    }
    Ok(homes)
}
