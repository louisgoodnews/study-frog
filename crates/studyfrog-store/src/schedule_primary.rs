//! The schedule of an item in its primary stack: what every caller that has no stack in mind gets (ADR 0011).

use rusqlite::Connection;
use studyfrog_core::uid::Uid;

use crate::error::{StoreError, sql_err};
use crate::schedule::{
    StoredMem, StoredSm2, load_mem_state_in, load_sm2_state_in, save_mem_state_in,
    save_sm2_state_in,
};

/// The stack that holds the item's file, or the Inbox (uid 0) for a loose or unknown item.
pub fn primary_stack(conn: &Connection, item: Uid) -> Result<Uid, StoreError> {
    let sql = "SELECT s.uid FROM item i JOIN stack_identity s ON s.folder = i.stack_path WHERE i.uid = ?1";
    let id = studyfrog_md::ulid::format_ulid(item);
    match conn.query_row(sql, [id], |r| r.get::<_, String>(0)) {
        Ok(text) => studyfrog_md::ulid::parse_ulid(&text)
            .ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}"))),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Uid(0)),
        Err(e) => Err(sql_err(e)),
    }
}

/// The FSRS state of an item in its primary stack.
pub fn load_mem_state(conn: &Connection, item: Uid) -> Result<Option<StoredMem>, StoreError> {
    load_mem_state_in(conn, (item, primary_stack(conn, item)?))
}

/// Save the FSRS state of an item in its primary stack.
pub fn save_mem_state(conn: &Connection, item: Uid, m: &StoredMem) -> Result<(), StoreError> {
    save_mem_state_in(conn, (item, primary_stack(conn, item)?), m)
}

/// The SM-2 state of an item in its primary stack.
pub fn load_sm2_state(conn: &Connection, item: Uid) -> Result<Option<StoredSm2>, StoreError> {
    load_sm2_state_in(conn, (item, primary_stack(conn, item)?))
}

/// Save the SM-2 state of an item in its primary stack.
pub fn save_sm2_state(conn: &Connection, item: Uid, m: &StoredSm2) -> Result<(), StoreError> {
    save_sm2_state_in(conn, (item, primary_stack(conn, item)?), m)
}
