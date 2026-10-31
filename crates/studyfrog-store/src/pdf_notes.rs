//! Items made from a PDF carry a `source` key such as `bio/cells.pdf#page=3`; this finds them again.

use rusqlite::{Connection, params};

use crate::error::{StoreError, sql_err};

/// An item whose source is a page of a PDF.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcedItem {
    /// Item identity, as in the index.
    pub uid: String,
    /// Title.
    pub title: String,
    /// `note`, `card` or `question`.
    pub kind: String,
    /// Vault-relative path of the item.
    pub path: String,
    /// The page it was made from (0-based), `None` when the source names no page.
    pub page: Option<usize>,
}

/// Split `path#page=N` into the path and the 0-based page.
pub fn split_source(source: &str) -> (&str, Option<usize>) {
    match source.split_once("#page=") {
        Some((path, n)) => (
            path,
            n.trim()
                .parse::<usize>()
                .ok()
                .filter(|n| *n > 0)
                .map(|n| n - 1),
        ),
        None => (source, None),
    }
}

/// Items made from this PDF; with a page, only those made from that page.
pub fn items_made_from(
    conn: &Connection,
    pdf: &str,
    page: Option<usize>,
) -> Result<Vec<SourcedItem>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT uid, title, kind, path, source FROM item WHERE source = ?1 OR source LIKE ?2 ESCAPE '\\' ORDER BY path")
        .map_err(sql_err)?;
    let escaped = pdf
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let rows = stmt
        .query_map(params![pdf, format!("{escaped}#page=%")], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut out = Vec::new();
    for r in rows {
        let (uid, title, kind, path, source) = r.map_err(sql_err)?;
        let found = split_source(&source).1;
        if page.is_none() || found == page {
            out.push(SourcedItem {
                uid,
                title,
                kind,
                path,
                page: found,
            });
        }
    }
    Ok(out)
}
