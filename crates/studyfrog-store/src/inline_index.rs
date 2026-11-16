//! Virtual inline card indexing and lifecycle sync (TASK-0270, STORY-0161).
//!
//! Indexes inline flashcards parsed from note markdown into SQLite as virtual items,
//! deriving deterministic Uids to ensure stable identity across edits.
//! Pure functions, zero inherent impls (P1).

use rusqlite::{Connection, params};
use studyfrog_core::uid::Uid;
use studyfrog_md::inline_cards::scan_inline_cards;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::{StoreError, sql_err};
use crate::hash::{combine, hash_bytes};

/// A virtual inline card extracted from a markdown note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualInlineCard {
    /// Owning note item UID.
    pub parent_uid: Uid,
    /// Deterministically derived virtual item UID.
    pub virtual_uid: Uid,
    /// Front or prompt of the card.
    pub term: String,
    /// Back or answer of the card.
    pub definition: String,
    /// 1-based line number in source markdown note.
    pub line_number: usize,
}

/// Derives a deterministic 128-bit UID for a virtual card from parent UID, line number, and term.
///
/// Ensures edits to the definition on the same line keep a stable identity.
pub fn derive_virtual_uid(parent_uid: &Uid, line_number: usize, term: &str) -> Uid {
    let mut payload = Vec::with_capacity(16 + 8 + term.len());
    payload.extend_from_slice(&parent_uid.0.to_le_bytes());
    payload.extend_from_slice(&(line_number as u64).to_le_bytes());
    payload.extend_from_slice(term.as_bytes());

    let h1 = hash_bytes(&payload);
    let h2 = combine(
        0x9e37_79b9_7f4a_7c15,
        hash_bytes(&[b"virtual_inline_card:", payload.as_slice()].concat()),
    );
    let raw = ((h1 as u128) << 64) | (h2 as u128);
    Uid(if raw == 0 { 1 } else { raw })
}

/// Extracts virtual inline cards from note markdown for a given parent item UID.
pub fn extract_virtual_cards(parent_uid: Uid, markdown: &str) -> Vec<VirtualInlineCard> {
    let scanned = scan_inline_cards(markdown);
    let mut cards = Vec::with_capacity(scanned.len());
    for item in scanned {
        let virtual_uid = derive_virtual_uid(&parent_uid, item.line_number, &item.term);
        cards.push(VirtualInlineCard {
            parent_uid,
            virtual_uid,
            term: item.term,
            definition: item.definition,
            line_number: item.line_number,
        });
    }
    cards
}

/// Ensures the `inline_cards` SQLite table and its parent index exist.
pub fn ensure_inline_cards_table(conn: &Connection) -> Result<(), StoreError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS inline_cards (
            parent_uid  TEXT NOT NULL,
            virtual_uid TEXT PRIMARY KEY,
            term        TEXT NOT NULL,
            definition  TEXT NOT NULL,
            line_number INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_inline_cards_parent ON inline_cards(parent_uid);",
    )
    .map_err(sql_err)?;
    Ok(())
}

/// Replaces all inline cards for `parent_uid` with the provided slice in a transaction.
pub fn sync_inline_cards(
    conn: &Connection,
    parent_uid: Uid,
    cards: &[VirtualInlineCard],
) -> Result<(), StoreError> {
    ensure_inline_cards_table(conn)?;
    let tx = conn.unchecked_transaction().map_err(sql_err)?;
    let p_str = format_ulid(parent_uid);
    tx.execute("DELETE FROM inline_cards WHERE parent_uid = ?1", [&p_str])
        .map_err(sql_err)?;
    {
        let mut stmt = tx
            .prepare_cached(
                "INSERT OR REPLACE INTO inline_cards (parent_uid, virtual_uid, term, definition, line_number)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )
            .map_err(sql_err)?;
        for card in cards {
            let v_str = format_ulid(card.virtual_uid);
            stmt.execute(params![
                &p_str,
                &v_str,
                &card.term,
                &card.definition,
                card.line_number as i64,
            ])
            .map_err(sql_err)?;
        }
    }
    tx.commit().map_err(sql_err)?;
    Ok(())
}

/// Loads all virtual inline cards associated with `parent_uid`, ordered by line number.
pub fn load_inline_cards(
    conn: &Connection,
    parent_uid: Uid,
) -> Result<Vec<VirtualInlineCard>, StoreError> {
    ensure_inline_cards_table(conn)?;
    let p_str = format_ulid(parent_uid);
    let mut stmt = conn
        .prepare(
            "SELECT parent_uid, virtual_uid, term, definition, line_number
             FROM inline_cards
             WHERE parent_uid = ?1
             ORDER BY line_number, virtual_uid",
        )
        .map_err(sql_err)?;

    let rows = stmt
        .query_map([&p_str], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, i64>(4)?,
            ))
        })
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;

    let mut out = Vec::with_capacity(rows.len());
    for (p, v, term, definition, line) in rows {
        let parent = parse_ulid(&p)
            .ok_or_else(|| StoreError::Corrupt(format!("invalid parent_uid ULID: {p:?}")))?;
        let virtual_uid = parse_ulid(&v)
            .ok_or_else(|| StoreError::Corrupt(format!("invalid virtual_uid ULID: {v:?}")))?;
        out.push(VirtualInlineCard {
            parent_uid: parent,
            virtual_uid,
            term,
            definition,
            line_number: line as usize,
        });
    }
    Ok(out)
}
