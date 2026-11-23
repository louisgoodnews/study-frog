//! Repository functions for classroom sessions (EPIC-068). Every write runs in one transaction, so a
//! failed write leaves the stored session as it was. Deleting a session cascades to all its rows.

use rusqlite::Connection;
use studyfrog_core::classroom::ClassroomSession;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;

use crate::classroom_markdown::render_session_markdown;
use crate::classroom_read::read_session;
use crate::classroom_write::write_session;
use crate::error::{StoreError, sql_err};
use crate::runs::uid_of;

/// Stores a new session with all its rows. Fails, and stores nothing, if the id is already used.
pub fn save_session(conn: &Connection, session: &ClassroomSession) -> Result<(), StoreError> {
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    write_session(&tx, session)?;
    tx.commit().map_err(sql_err)
}

/// Replaces a stored session with this one, rows and all. Returns `false`, and writes nothing,
/// when no session has the id.
pub fn update_session(conn: &Connection, session: &ClassroomSession) -> Result<bool, StoreError> {
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    if delete_rows(&tx, session.id)? == 0 {
        return Ok(false);
    }
    write_session(&tx, session)?;
    tx.commit().map_err(sql_err)?;
    Ok(true)
}

/// Deletes a session and every row that belongs to it. Returns whether a session was deleted.
pub fn delete_session(conn: &Connection, id: Uid) -> Result<bool, StoreError> {
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    let deleted = delete_rows(&tx, id)? > 0;
    tx.commit().map_err(sql_err)?;
    Ok(deleted)
}

fn delete_rows(conn: &Connection, id: Uid) -> Result<usize, StoreError> {
    conn.execute(
        "DELETE FROM classroom_session WHERE uid = ?1",
        [format_ulid(id)],
    )
    .map_err(sql_err)
}

/// The full session with this id, or `None`.
pub fn load_session(conn: &Connection, id: Uid) -> Result<Option<ClassroomSession>, StoreError> {
    read_session(conn, id)
}

/// Every stored session in full, newest first.
pub fn list_sessions(conn: &Connection) -> Result<Vec<ClassroomSession>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid FROM classroom_session ORDER BY created_at_ms DESC, uid DESC")
        .map_err(sql_err)?;
    let uids = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(sql_err)?
        .map(|r| r.map_err(sql_err))
        .collect::<Result<Vec<String>, StoreError>>()?;
    let mut out = Vec::with_capacity(uids.len());
    for text in &uids {
        let id = uid_of(text)?;
        out.extend(read_session(conn, id)?);
    }
    Ok(out)
}

/// The session rendered as a Markdown transcript, or `None` when no session has the id.
pub fn export_session_markdown(conn: &Connection, id: Uid) -> Result<Option<String>, StoreError> {
    Ok(read_session(conn, id)?.map(|s| render_session_markdown(&s)))
}
