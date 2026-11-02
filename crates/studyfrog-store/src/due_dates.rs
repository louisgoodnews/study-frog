//! Reading and moving the due dates of scheduled items without touching their memory state, and the statuses of stacks.

use std::collections::HashMap;

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::error::{StoreError, sql_err};

/// The due dates an item has in each algorithm's state (`None` where that algorithm never scheduled it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dues {
    /// The FSRS state's due time.
    pub fsrs: Option<i64>,
    /// The SM-2 state's due time.
    pub sm2: Option<i64>,
}

fn one(conn: &Connection, table: &str, id: &str) -> Result<Option<i64>, StoreError> {
    let sql = format!("SELECT due_ms FROM {table} WHERE item_uid = ?1");
    match conn.query_row(&sql, [id], |r| r.get(0)) {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// Both due dates of an item.
pub fn load_dues(conn: &Connection, item: Uid) -> Result<Dues, StoreError> {
    let id = format_ulid(item);
    Ok(Dues {
        fsrs: one(conn, "mem_state", &id)?,
        sm2: one(conn, "sm2_state", &id)?,
    })
}

/// Set the due dates of an item; a `None` leaves that algorithm's state alone. Stability, difficulty and
/// every other column stay as they are.
pub fn set_dues(conn: &Connection, item: Uid, dues: Dues) -> Result<(), StoreError> {
    let id = format_ulid(item);
    for (table, due) in [("mem_state", dues.fsrs), ("sm2_state", dues.sm2)] {
        if let Some(ms) = due {
            let sql = format!("UPDATE {table} SET due_ms = ?2 WHERE item_uid = ?1");
            conn.execute(&sql, params![id, ms]).map_err(sql_err)?;
        }
    }
    Ok(())
}

/// The `status` word of every stack's `_stack.md`, by the stack's folder (`None` where it states none).
pub fn stack_statuses(conn: &Connection) -> Result<HashMap<String, Option<String>>, StoreError> {
    let sql = "SELECT s.folder, i.status FROM stack_setting s JOIN item i ON i.uid = s.uid";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
        })
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}
