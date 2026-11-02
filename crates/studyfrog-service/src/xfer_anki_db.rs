//! Reading the tables of an Anki collection.

use std::collections::HashMap;

use rusqlite::Connection;

/// A note: id, model id, tags and fields.
pub(crate) type NoteRow = (i64, i64, String, String);

/// The notes, the deck of each note, the note of each card and the review log of an Anki collection.
pub(crate) type Rows = (
    Vec<NoteRow>,
    HashMap<i64, i64>,
    HashMap<i64, i64>,
    Vec<(i64, i64, i64)>,
);

pub(crate) fn rows(conn: &Connection) -> Result<Rows, String> {
    let bad = |e: rusqlite::Error| format!("the Anki collection is damaged: {e}");
    let notes = conn
        .prepare("SELECT id, mid, tags, flds FROM notes ORDER BY id LIMIT 30000")
        .and_then(|mut s| {
            s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(bad)?;
    let mut deck_of = HashMap::new();
    let mut note_of = HashMap::new();
    let cards: Vec<(i64, i64, i64)> = conn
        .prepare("SELECT id, nid, did FROM cards ORDER BY id LIMIT 200000")
        .and_then(|mut s| {
            s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(bad)?;
    for (cid, nid, did) in &cards {
        deck_of.entry(*nid).or_insert(*did);
        note_of.insert(*cid, *nid);
    }
    let log = conn
        .prepare("SELECT id, cid, ease FROM revlog ORDER BY id LIMIT 500000")
        .and_then(|mut s| {
            s.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<Result<Vec<_>, _>>()
        })
        .unwrap_or_default();
    Ok((notes, deck_of, note_of, log))
}
