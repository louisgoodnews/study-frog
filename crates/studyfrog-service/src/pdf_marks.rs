//! The highlights and notes of a PDF: read and written as a plain-text file next to it, and kept in
//! the search index. The PDF itself is never touched.

use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use studyfrog_config::error::describe_config_error;
use studyfrog_config::pdf_marks::{PdfMark, marks_path, parse_marks, relocate, render_marks};
use studyfrog_store::pdf_index::{MarkText, pdf_marks_seen, pdf_marks_store, pdf_page_plain};
use studyfrog_store::vault::VaultFile;

use crate::session::{ServiceError, VaultSession};

/// A path inside the vault that ends in `.pdf`.
fn checked(rel: &str) -> Result<(), ServiceError> {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    if clean && rel.to_lowercase().ends_with(".pdf") {
        return Ok(());
    }
    Err(ServiceError::Invalid(
        "only PDF files inside the vault have notes".to_owned(),
    ))
}

fn stat_of(path: &Path) -> (i64, i64) {
    let Ok(meta) = fs::metadata(path) else {
        return (-1, 0);
    };
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| i64::try_from(d.as_nanos()).unwrap_or(i64::MAX));
    (i64::try_from(meta.len()).unwrap_or(i64::MAX), mtime)
}

fn texts(marks: &[PdfMark]) -> Vec<MarkText> {
    marks
        .iter()
        .map(|m| MarkText {
            id: m.id,
            page: m.page,
            text: m.covered.clone(),
            note: m.note.clone(),
        })
        .collect()
}

/// The marks of a PDF; none when it has no notes file. A damaged file is an error that names the line.
pub fn load_marks(session: &VaultSession, rel: &str) -> Result<Vec<PdfMark>, ServiceError> {
    checked(rel)?;
    let path = session.root.join(marks_path(rel));
    let text = match fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(ServiceError::Io(format!("cannot read the notes: {e}"))),
    };
    parse_marks(&text).map_err(|e| {
        ServiceError::Invalid(format!(
            "the notes file {} cannot be used: {}",
            marks_path(rel),
            describe_config_error(&e)
        ))
    })
}

/// Write the marks of a PDF (no file at all when there are none) and update the index.
pub fn save_marks(
    session: &mut VaultSession,
    rel: &str,
    marks: &[PdfMark],
) -> Result<(), ServiceError> {
    checked(rel)?;
    let target = session.root.join(marks_path(rel));
    if marks.is_empty() {
        let _ = fs::remove_file(&target);
    } else {
        let temp = target.with_extension("toml.tmp");
        fs::write(&temp, render_marks(marks))
            .and_then(|()| fs::rename(&temp, &target))
            .map_err(|e| ServiceError::Io(format!("cannot save the notes: {e}")))?;
    }
    let stat = stat_of(&target);
    pdf_marks_store(&mut session.conn, rel, stat, &texts(marks)).map_err(ServiceError::Store)
}

/// Read again the notes files that changed since the index saw them (also those edited by hand).
pub fn refresh_marks(
    session: &mut VaultSession,
    listed: &[VaultFile],
) -> Result<bool, ServiceError> {
    let seen = pdf_marks_seen(&session.conn).map_err(ServiceError::Store)?;
    let mut any = false;
    for file in listed {
        let stat = stat_of(&session.root.join(marks_path(&file.rel)));
        if seen.get(&file.rel).is_none_or(|s| *s == stat) {
            continue;
        }
        let marks = load_marks(session, &file.rel).unwrap_or_default();
        pdf_marks_store(&mut session.conn, &file.rel, stat, &texts(&marks))
            .map_err(ServiceError::Store)?;
        any = true;
    }
    Ok(any)
}

/// Where each mark is now, for a PDF that may have been replaced by a new revision: `None` when the
/// text it covered is gone.
pub fn mark_pages_now(
    session: &VaultSession,
    rel: &str,
    marks: &[PdfMark],
) -> Result<Vec<Option<usize>>, ServiceError> {
    let count = marks.iter().map(|m| m.page).max().map_or(0, |p| p + 1);
    let mut pages = Vec::new();
    let mut page = 0;
    loop {
        match pdf_page_plain(&session.conn, rel, page).map_err(ServiceError::Store)? {
            Some(p) => pages.push(p.text),
            None if page >= count => break,
            None => pages.push(String::new()),
        }
        page += 1;
    }
    Ok(marks.iter().map(|m| relocate(m, &pages)).collect())
}
