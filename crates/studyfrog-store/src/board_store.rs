//! Persistence of the kanban board. The board is small, so every save replaces it in one transaction.

use rusqlite::{Connection, params};
use studyfrog_board::model::{Board, Card, Column};
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};

fn uid(text: &str) -> Result<studyfrog_core::uid::Uid, StoreError> {
    parse_ulid(text).ok_or_else(|| StoreError::Corrupt(format!("ulid: {text:?}")))
}

/// Replace the stored board with `b`.
pub fn save_board(conn: &mut Connection, b: &Board) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    tx.execute("DELETE FROM board_column", [])
        .map_err(sql_err)?; // cards cascade
    for (pos, col) in b.columns.iter().enumerate() {
        tx.execute(
            "INSERT INTO board_column (id, pos, name, wip_limit) VALUES (?1, ?2, ?3, ?4)",
            params![format_ulid(col.id), pos as i64, col.name, col.wip_limit],
        )
        .map_err(sql_err)?;
        for (i, card) in col.cards.iter().enumerate() {
            tx.execute(
                "INSERT INTO board_card (id, column_id, pos, title, notes, stack_uid, points) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![format_ulid(card.id), format_ulid(col.id), i as i64, card.title, card.notes, card.stack.map(format_ulid), card.points],
            )
            .map_err(sql_err)?;
        }
    }
    tx.commit().map_err(sql_err)
}

fn cards_of(conn: &Connection, column: &str) -> Result<Vec<Card>, StoreError> {
    let sql = "SELECT id, title, notes, stack_uid, points FROM board_card WHERE column_id = ?1 ORDER BY pos";
    let mut stmt = conn.prepare(sql).map_err(sql_err)?;
    let rows = stmt
        .query_map([column], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<u32>>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut cards = Vec::new();
    for row in rows {
        let (id, title, notes, stack, points) = row.map_err(sql_err)?;
        let stack = stack.as_deref().map(uid).transpose()?;
        cards.push(Card {
            id: uid(&id)?,
            title,
            notes,
            stack,
            points,
        });
    }
    Ok(cards)
}

/// The stored board, or `None` when none has been saved yet.
pub fn load_board(conn: &Connection) -> Result<Option<Board>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT id, name, wip_limit FROM board_column ORDER BY pos")
        .map_err(sql_err)?;
    let rows: Vec<(String, String, Option<u32>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(sql_err)?
        .collect::<Result<_, _>>()
        .map_err(sql_err)?;
    if rows.is_empty() {
        return Ok(None);
    }
    let mut columns = Vec::new();
    for (id, name, wip_limit) in rows {
        columns.push(Column {
            id: uid(&id)?,
            name,
            wip_limit,
            cards: cards_of(conn, &id)?,
        });
    }
    Ok(Some(Board { columns }))
}
