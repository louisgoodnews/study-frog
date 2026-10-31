//! PDF files in the vault: keep their index (title, page count, text per page) current. Reading
//! is slow, so a pass handles a few files and the next poll continues; unchanged files cost a stat.

use std::path::Path;
use std::sync::Arc;

use studyfrog_pdf::PdfBytes;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::text::pdf_all_text;
use studyfrog_store::error::StoreError;
use studyfrog_store::hash::hash_bytes;
use studyfrog_store::pdf_index::{
    PagePlain, PdfRow, list_pdf_rows, pdf_forget, pdf_rows, pdf_store, pdf_touch,
};
use studyfrog_store::pdf_search::{PdfHit, search_pdfs};
use studyfrog_store::vault::{VaultFile, list_pdfs};

use crate::session::{SETTLE_NS, ServiceError, VaultSession};

/// How many files one pass reads and extracts before it hands control back.
pub const FILES_PER_PASS: usize = 6;

fn store(e: StoreError) -> ServiceError {
    ServiceError::Store(e)
}

/// A file needs reading when it is new or its size or mtime moved.
fn changed(file: &VaultFile, row: Option<&PdfRow>) -> bool {
    row.is_none_or(|r| r.size != file.size || r.mtime_ns != file.mtime_ns)
}

fn stem(rel: &str) -> String {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name.rsplit_once('.').map_or(name, |(s, _)| s).to_owned()
}

/// Read a file into a row and its page texts. A file that cannot be read gets a row with the reason.
fn read_pdf(file: &VaultFile, bytes: Vec<u8>, hash: u64) -> (PdfRow, Vec<PagePlain>) {
    let data: PdfBytes = Arc::new(bytes);
    let mut row = PdfRow {
        path: file.rel.clone(),
        size: file.size,
        mtime_ns: file.mtime_ns,
        hash,
        pages: 0,
        title: stem(&file.rel),
        error: None,
    };
    let read = pdf_info(&data).and_then(|info| pdf_all_text(&data).map(|t| (info, t)));
    match read {
        Ok((info, texts)) => {
            row.pages = info.page_count;
            if let Some(t) = info.title.filter(|t| !t.trim().is_empty()) {
                row.title = t.trim().to_owned();
            }
            let pages = texts
                .into_iter()
                .map(|p| PagePlain {
                    text: p.text,
                    image_only: p.image_only,
                })
                .collect();
            (row, pages)
        }
        Err(e) => {
            row.error = Some(describe_pdf_error(&e));
            (row, Vec::new())
        }
    }
}

/// Bring one file up to date. `true` when its text was extracted again.
fn refresh(
    session: &mut VaultSession,
    file: &VaultFile,
    old: Option<&PdfRow>,
) -> Result<bool, ServiceError> {
    let Ok(bytes) = std::fs::read(&file.abs) else {
        return Ok(false);
    };
    let hash = hash_bytes(&bytes);
    if old.is_some_and(|r| r.hash == hash) {
        pdf_touch(&session.conn, &file.rel, file.size, file.mtime_ns).map_err(store)?;
        return Ok(false);
    }
    let (row, pages) = read_pdf(file, bytes, hash);
    pdf_store(&mut session.conn, &row, &pages).map_err(store)?;
    Ok(true)
}

/// Make the PDF index match the files on disk, at most [`FILES_PER_PASS`] files at a time.
/// `true` when anything in the index changed. Files modified moments ago are left for the next pass.
pub fn index_pdfs(session: &mut VaultSession, now_ns: i64) -> Result<bool, ServiceError> {
    let listed = list_pdfs(&session.root).map_err(store)?;
    let rows = pdf_rows(&session.conn).map_err(store)?;
    let gone: Vec<String> = rows
        .keys()
        .filter(|p| !listed.iter().any(|f| &f.rel == *p))
        .cloned()
        .collect();
    pdf_forget(&mut session.conn, &gone).map_err(store)?;
    let waiting: Vec<&VaultFile> = listed
        .iter()
        .filter(|f| changed(f, rows.get(&f.rel)))
        .filter(|f| now_ns.saturating_sub(f.mtime_ns) >= SETTLE_NS)
        .collect();
    session.pdf_pending = waiting.len().saturating_sub(FILES_PER_PASS);
    let mut any = !gone.is_empty();
    for file in waiting.into_iter().take(FILES_PER_PASS) {
        any |= refresh(session, file, rows.get(&file.rel))?;
    }
    any |= crate::pdf_marks::refresh_marks(session, &listed)?;
    Ok(any)
}

/// Make the PDF index current in one go (used when a file was just added by the app).
pub fn index_pdfs_fully(session: &mut VaultSession, now_ns: i64) -> Result<(), ServiceError> {
    while index_pdfs(session, now_ns)? && session.pdf_pending > 0 {}
    Ok(())
}

/// PDFs and pages matching a search.
pub fn find_pdfs(
    session: &VaultSession,
    query: &str,
    limit: usize,
) -> Result<Vec<PdfHit>, ServiceError> {
    search_pdfs(&session.conn, query, limit).map_err(store)
}

/// Every indexed PDF, by title.
pub fn pdf_library(session: &VaultSession) -> Result<Vec<PdfRow>, ServiceError> {
    list_pdf_rows(&session.conn).map_err(store)
}

/// The absolute path of an indexed PDF.
pub fn pdf_abs_path(session: &VaultSession, rel: &str) -> std::path::PathBuf {
    Path::new(&session.root).join(rel)
}
