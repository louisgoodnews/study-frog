//! What the analytics need to know about items besides their reviews: title, file, stack, tags and links.

use rusqlite::Connection;
use studyfrog_core::uid::Uid;

use crate::error::{StoreError, sql_err};
use crate::runs::uid_of;

/// A card or question of the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemFact {
    /// The item.
    pub uid: Uid,
    /// Its title.
    pub title: String,
    /// Its file, relative to the vault.
    pub path: String,
    /// The folder of its stack, `None` for a loose item.
    pub stack_path: Option<String>,
}

fn pairs(conn: &Connection, sql: &str) -> Result<Vec<(Uid, String)>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (uid, text) = row.map_err(sql_err)?;
        out.push((uid_of(&uid)?, text));
    }
    Ok(out)
}

/// Every card and question.
pub fn load_item_facts(conn: &Connection) -> Result<Vec<ItemFact>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid, title, path, stack_path FROM item WHERE kind IN ('card', 'question') ORDER BY path")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (uid, title, path, stack_path) = row.map_err(sql_err)?;
        out.push(ItemFact {
            uid: uid_of(&uid)?,
            title,
            path,
            stack_path,
        });
    }
    Ok(out)
}

/// Every (item, tag).
pub fn load_all_tags(conn: &Connection) -> Result<Vec<(Uid, String)>, StoreError> {
    pairs(conn, "SELECT uid, tag FROM tag")
}

/// Every (item, link target as written).
pub fn load_all_links(conn: &Connection) -> Result<Vec<(Uid, String)>, StoreError> {
    pairs(conn, "SELECT src_uid, target FROM link")
}

/// The scheduler state of one (item, stack): FSRS (stability, difficulty) or SM-2 (ease, repetitions, interval).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateRow {
    /// The item.
    pub item: Uid,
    /// The stack.
    pub stack: Uid,
    /// FSRS stability and difficulty.
    pub fsrs: Option<(f32, f32)>,
    /// SM-2 ease, repetitions and interval in days.
    pub sm2: Option<(f32, u32, u32)>,
    /// When it is due.
    pub due_ms: i64,
    /// When it was last reviewed.
    pub last_ms: i64,
}

type StateTuple = (String, String, f32, f32, u32, i64, i64);

fn states(conn: &Connection, sql: &str, fsrs: bool) -> Result<Vec<StateRow>, StoreError> {
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for row in rows {
        let (item, stack, a, b, c, due_ms, last_ms): StateTuple = row.map_err(sql_err)?;
        out.push(StateRow {
            item: uid_of(&item)?,
            stack: uid_of(&stack)?,
            fsrs: fsrs.then_some((a, b)),
            sm2: (!fsrs).then_some((a, b as u32, c)),
            due_ms,
            last_ms,
        });
    }
    Ok(out)
}

/// Every scheduler state, FSRS and SM-2: of every (item, stack), or only of each item's primary stack.
pub fn load_state_rows(conn: &Connection, primary: bool) -> Result<Vec<StateRow>, StoreError> {
    let (mem, sm2) = if primary {
        ("mem_primary", "sm2_primary")
    } else {
        ("mem_state", "sm2_state")
    };
    let mut out = states(
        conn,
        &format!(
            "SELECT item_uid, stack_uid, stability, difficulty, 0, due_ms, last_review_ms FROM {mem}"
        ),
        true,
    )?;
    out.extend(states(
        conn,
        &format!("SELECT item_uid, stack_uid, ease, repetitions, interval_days, due_ms, last_review_ms FROM {sm2}"),
        false,
    )?);
    Ok(out)
}
