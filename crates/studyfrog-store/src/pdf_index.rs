//! The index of PDF files: one row per file, one row of text per page. Plain data only; reading
//! the PDF itself belongs to the service (this crate does not know the PDF library).

use std::collections::HashMap;

use rusqlite::{Connection, params};

use crate::error::{StoreError, sql_err};

/// What the index remembers about a PDF file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdfRow {
    /// Vault-relative path.
    pub path: String,
    /// Size in bytes.
    pub size: u64,
    /// Modification time, nanoseconds since the Unix epoch.
    pub mtime_ns: i64,
    /// Content hash.
    pub hash: u64,
    /// Number of pages (0 when unreadable).
    pub pages: usize,
    /// Document title, else the file name without extension.
    pub title: String,
    /// Why the file could not be read, if so.
    pub error: Option<String>,
}

/// The text of one page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagePlain {
    /// The page's text.
    pub text: String,
    /// The page holds only pictures (a scan).
    pub image_only: bool,
}

/// Every indexed file by path.
pub fn pdf_rows(conn: &Connection) -> Result<HashMap<String, PdfRow>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT path, size, mtime_ns, hash, pages, title, error FROM pdf_doc")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| {
            Ok(PdfRow {
                path: r.get(0)?,
                size: r.get::<_, i64>(1)? as u64,
                mtime_ns: r.get(2)?,
                hash: r.get::<_, i64>(3)? as u64,
                pages: r.get::<_, i64>(4)? as usize,
                title: r.get(5)?,
                error: r.get(6)?,
            })
        })
        .map_err(sql_err)?;
    rows.map(|r| r.map(|p| (p.path.clone(), p)).map_err(sql_err))
        .collect()
}

/// Record a file and replace its page texts.
pub fn pdf_store(
    conn: &mut Connection,
    row: &PdfRow,
    pages: &[PagePlain],
) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    tx.execute("DELETE FROM pdf_doc WHERE path = ?1", [&row.path])
        .map_err(sql_err)?;
    tx.execute(
        "INSERT INTO pdf_doc (path, size, mtime_ns, hash, pages, title, error) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![row.path, row.size as i64, row.mtime_ns, row.hash as i64, row.pages as i64, row.title, row.error],
    )
    .map_err(sql_err)?;
    for (i, p) in pages.iter().enumerate() {
        tx.execute(
            "INSERT INTO pdf_page (path, page, text, image_only) VALUES (?1, ?2, ?3, ?4)",
            params![row.path, i as i64, p.text, p.image_only],
        )
        .map_err(sql_err)?;
    }
    tx.commit().map_err(sql_err)
}

/// Only the stat data of a file whose content is unchanged.
pub fn pdf_touch(
    conn: &Connection,
    path: &str,
    size: u64,
    mtime_ns: i64,
) -> Result<(), StoreError> {
    conn.execute(
        "UPDATE pdf_doc SET size = ?2, mtime_ns = ?3 WHERE path = ?1",
        params![path, size as i64, mtime_ns],
    )
    .map(|_| ())
    .map_err(sql_err)
}

/// Forget files that are gone (their pages go with them).
pub fn pdf_forget(conn: &mut Connection, paths: &[String]) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    for p in paths {
        tx.execute("DELETE FROM pdf_doc WHERE path = ?1", [p])
            .map_err(sql_err)?;
    }
    tx.commit().map_err(sql_err)
}

/// All indexed PDFs, by title.
pub fn list_pdf_rows(conn: &Connection) -> Result<Vec<PdfRow>, StoreError> {
    let mut all: Vec<PdfRow> = pdf_rows(conn)?.into_values().collect();
    all.sort_by(|a, b| {
        a.title
            .to_lowercase()
            .cmp(&b.title.to_lowercase())
            .then_with(|| a.path.cmp(&b.path))
    });
    Ok(all)
}

/// The text of one page (0-based), empty when the page is unknown.
pub fn pdf_page_plain(
    conn: &Connection,
    path: &str,
    page: usize,
) -> Result<Option<PagePlain>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT text, image_only FROM pdf_page WHERE path = ?1 AND page = ?2")
        .map_err(sql_err)?;
    let mut rows = stmt
        .query_map(params![path, page as i64], |r| {
            Ok(PagePlain {
                text: r.get(0)?,
                image_only: r.get(1)?,
            })
        })
        .map_err(sql_err)?;
    rows.next().transpose().map_err(sql_err)
}

/// The searchable part of one mark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkText {
    /// Id in the notes file.
    pub id: u32,
    /// Page, 0-based.
    pub page: usize,
    /// The text the mark covers.
    pub text: String,
    /// The user's note.
    pub note: String,
}

/// Size and modification time of each notes file as last read (`-1` size: there was none).
pub fn pdf_marks_seen(conn: &Connection) -> Result<HashMap<String, (i64, i64)>, StoreError> {
    let mut stmt = conn
        .prepare("SELECT path, marks_size, marks_mtime FROM pdf_doc")
        .map_err(sql_err)?;
    let rows = stmt
        .query_map([], |r| Ok((r.get(0)?, (r.get(1)?, r.get(2)?))))
        .map_err(sql_err)?;
    rows.map(|r| r.map_err(sql_err)).collect()
}

/// Replace the indexed marks of a PDF and remember which version of the notes file they came from.
pub fn pdf_marks_store(
    conn: &mut Connection,
    path: &str,
    stat: (i64, i64),
    marks: &[MarkText],
) -> Result<(), StoreError> {
    let tx = conn.transaction().map_err(sql_err)?;
    tx.execute("DELETE FROM pdf_mark WHERE path = ?1", [path])
        .map_err(sql_err)?;
    for m in marks {
        tx.execute(
            "INSERT INTO pdf_mark (path, id, page, text, note) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![path, m.id, m.page as i64, m.text, m.note],
        )
        .map_err(sql_err)?;
    }
    tx.execute(
        "UPDATE pdf_doc SET marks_size = ?2, marks_mtime = ?3 WHERE path = ?1",
        params![path, stat.0, stat.1],
    )
    .map_err(sql_err)?;
    tx.commit().map_err(sql_err)
}
