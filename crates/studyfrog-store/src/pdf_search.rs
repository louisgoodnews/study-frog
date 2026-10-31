//! Searching the PDF index by title and by page text.

use rusqlite::{Connection, params};

use crate::error::{StoreError, sql_err};

/// A page that matched a search.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfHit {
    /// Vault-relative path.
    pub path: String,
    /// Document title.
    pub title: String,
    /// 1-based page of the match, `None` when only the title matched.
    pub page: Option<usize>,
    /// A short stretch of text around the match.
    pub snippet: String,
    /// The match is in one of the user's highlights or notes.
    pub annotation: bool,
}

fn like(needle: &str) -> String {
    let escaped = needle
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// A stretch of `text` around the first case-insensitive match of `needle`.
pub fn snippet(text: &str, needle: &str) -> String {
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let lower = flat.to_lowercase();
    let at = lower.find(&needle.to_lowercase()).unwrap_or(0);
    let at = if flat.is_char_boundary(at) { at } else { 0 };
    let start = flat[..at]
        .char_indices()
        .rev()
        .nth(30)
        .map_or(0, |(i, _)| i);
    flat[start..].chars().take(90).collect()
}

/// PDFs whose title matches, then pages whose text matches (at most `limit` in all).
/// An empty query finds nothing.
pub fn search_pdfs(
    conn: &Connection,
    query: &str,
    limit: usize,
) -> Result<Vec<PdfHit>, StoreError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let pattern = like(needle);
    let mut hits = Vec::new();
    let mut stmt = conn
        .prepare("SELECT path, title FROM pdf_doc WHERE title LIKE ?1 ESCAPE '\\' ORDER BY title COLLATE NOCASE LIMIT ?2")
        .map_err(sql_err)?;
    let titles = stmt
        .query_map(params![pattern, limit as i64], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(sql_err)?;
    for t in titles {
        let (path, title) = t.map_err(sql_err)?;
        hits.push(PdfHit {
            path,
            title,
            page: None,
            snippet: String::new(),
            annotation: false,
        });
    }
    let room = limit.saturating_sub(hits.len());
    let mut stmt = conn
        .prepare("SELECT p.path, d.title, p.page, p.text FROM pdf_page p JOIN pdf_doc d ON d.path = p.path WHERE p.text LIKE ?1 ESCAPE '\\' ORDER BY d.title COLLATE NOCASE, p.page LIMIT ?2")
        .map_err(sql_err)?;
    let pages = stmt
        .query_map(params![pattern, room as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(sql_err)?;
    for p in pages {
        let (path, title, page, text) = p.map_err(sql_err)?;
        hits.push(PdfHit {
            path,
            title,
            page: Some(page as usize + 1),
            snippet: snippet(&text, needle),
            annotation: false,
        });
    }
    let room = limit.saturating_sub(hits.len());
    hits.extend(mark_hits(conn, &pattern, needle, room)?);
    Ok(hits)
}

/// The pages (0-based) of one PDF whose text contains `query`, in page order.
pub fn pages_containing(
    conn: &Connection,
    path: &str,
    query: &str,
) -> Result<Vec<usize>, StoreError> {
    let needle = query.trim();
    if needle.is_empty() {
        return Ok(Vec::new());
    }
    let mut stmt = conn
        .prepare(
            "SELECT page FROM pdf_page WHERE path = ?1 AND text LIKE ?2 ESCAPE '\\' ORDER BY page",
        )
        .map_err(sql_err)?;
    let rows = stmt
        .query_map(params![path, like(needle)], |r| r.get::<_, i64>(0))
        .map_err(sql_err)?;
    rows.map(|r| r.map(|p| p as usize).map_err(sql_err))
        .collect()
}

/// Highlights and notes whose covered text or note contains the query.
fn mark_hits(
    conn: &Connection,
    pattern: &str,
    needle: &str,
    room: usize,
) -> Result<Vec<PdfHit>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT m.path, d.title, m.page, m.text, m.note FROM pdf_mark m JOIN pdf_doc d ON d.path = m.path WHERE m.text LIKE ?1 ESCAPE '\\' OR m.note LIKE ?1 ESCAPE '\\' ORDER BY d.title COLLATE NOCASE, m.page LIMIT ?2")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map(params![pattern, room as i64], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(sql_err)?;
    let mut hits = Vec::new();
    for r in rows {
        let (path, title, page, text, note) = r.map_err(sql_err)?;
        let from = if note.to_lowercase().contains(&needle.to_lowercase()) {
            note
        } else {
            text
        };
        hits.push(PdfHit {
            path,
            title,
            page: Some(page as usize + 1),
            snippet: snippet(&from, needle),
            annotation: true,
        });
    }
    Ok(hits)
}

/// The pages (0-based) of one PDF that hold only pictures and no text, from the index.
pub fn scanned_pages(conn: &Connection, path: &str) -> Result<Vec<usize>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT page FROM pdf_page WHERE path = ?1 AND image_only = 1 ORDER BY page")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([path], |r| r.get::<_, i64>(0))
        .map_err(sql_err)?;
    rows.map(|r| r.map(|p| p as usize).map_err(sql_err))
        .collect()
}
