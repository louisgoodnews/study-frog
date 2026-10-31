//! Writing new PDFs from old ones: rearranged, rotated, merged, split, with the user's highlights and
//! notes and new properties. The original is never changed; the result is always a new file.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use studyfrog_config::pdf_marks::{MarkKind, PdfMark, mark_rgb};
use studyfrog_pdf::PdfBytes;
use studyfrog_pdf::annotate::{Mark, MarkKind as PdfKind, pdf_with_marks};
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::edit::{PageSpec, Properties, pdf_build, pdf_extract_ranges};
use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::render::PageRect;

use crate::session::{ServiceError, VaultSession};

/// What to write.
#[derive(Debug, Clone, PartialEq)]
pub struct SaveRequest {
    /// The files the pages come from: vault-relative paths, or absolute paths of files picked elsewhere.
    pub sources: Vec<String>,
    /// The pages of the result, in order.
    pub plan: Vec<PageSpec>,
    /// Vault-relative path of the new file.
    pub out: String,
    /// The highlights and notes of the first source, to be written into the copy.
    pub marks: Vec<PdfMark>,
    /// New properties, if any.
    pub props: Option<Properties>,
}

/// What was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedCopy {
    /// Vault-relative path of the new file.
    pub path: String,
    /// Its page count.
    pub pages: usize,
    /// Marks left out because their page is rotated.
    pub skipped_marks: usize,
}

fn invalid(text: impl Into<String>) -> ServiceError {
    ServiceError::Invalid(text.into())
}

fn clean(rel: &str) -> bool {
    !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty())
}

fn source_path(root: &Path, s: &str) -> Result<PathBuf, ServiceError> {
    if Path::new(s).is_absolute() {
        return Ok(PathBuf::from(s));
    }
    if clean(s) && s.to_lowercase().ends_with(".pdf") {
        return Ok(root.join(s));
    }
    Err(invalid("only PDF files can be used"))
}

fn read_source(root: &Path, s: &str) -> Result<PdfBytes, ServiceError> {
    let path = source_path(root, s)?;
    let data = fs::read(&path).map_err(|e| ServiceError::Io(format!("cannot read {s}: {e}")))?;
    Ok(Arc::new(data))
}

/// The target of a new file: inside the vault, a `.pdf`, in a folder that exists, not taken yet.
fn target_of(root: &Path, out: &str) -> Result<PathBuf, ServiceError> {
    if !clean(out) || !out.to_lowercase().ends_with(".pdf") {
        return Err(invalid(
            "the new file needs a name ending in .pdf, inside the vault",
        ));
    }
    let target = root.join(out);
    if target.exists() {
        return Err(invalid(format!(
            "{out} already exists; choose another name"
        )));
    }
    if target.parent().is_some_and(|p| !p.is_dir()) {
        return Err(invalid("the folder for the new file does not exist"));
    }
    Ok(target)
}

fn write_new(target: &Path, bytes: &[u8]) -> Result<(), ServiceError> {
    let temp = target.with_extension("pdf.tmp");
    fs::write(&temp, bytes)
        .and_then(|()| fs::rename(&temp, target))
        .map_err(|e| ServiceError::Io(format!("cannot write {}: {e}", target.display())))
}

/// The marks of the first source on the pages of the result.
fn remap(marks: &[PdfMark], plan: &[PageSpec]) -> Vec<Mark> {
    let mut out = Vec::new();
    for m in marks {
        let places = plan
            .iter()
            .enumerate()
            .filter(|(_, s)| s.source == 0 && s.page == m.page);
        for (i, _) in places {
            out.push(Mark {
                page: i,
                kind: if m.kind == MarkKind::Note {
                    PdfKind::Note
                } else {
                    PdfKind::Highlight
                },
                rects: m
                    .rects
                    .iter()
                    .map(|r| PageRect {
                        x: r.x,
                        y: r.y,
                        width: r.width,
                        height: r.height,
                    })
                    .collect(),
                color: mark_rgb(&m.color),
                contents: if m.note.is_empty() {
                    m.covered.clone()
                } else {
                    m.note.clone()
                },
            });
        }
    }
    out
}

/// Write the pages of the plan as a new PDF.
pub fn save_pdf_copy(session: &VaultSession, req: &SaveRequest) -> Result<SavedCopy, ServiceError> {
    let target = target_of(&session.root, &req.out)?;
    let sources = req
        .sources
        .iter()
        .map(|s| read_source(&session.root, s))
        .collect::<Result<Vec<_>, _>>()?;
    let built = pdf_build(&sources, &req.plan).map_err(|e| invalid(describe_pdf_error(&e)))?;
    let marks = remap(&req.marks, &req.plan);
    let (bytes, skipped) = if marks.is_empty() && req.props.is_none() {
        (built, 0)
    } else {
        let copy = pdf_with_marks(&Arc::new(built), &marks, req.props.as_ref())
            .map_err(|e| invalid(describe_pdf_error(&e)))?;
        (copy.bytes, copy.skipped)
    };
    write_new(&target, &bytes)?;
    Ok(SavedCopy {
        path: req.out.clone(),
        pages: req.plan.len(),
        skipped_marks: skipped,
    })
}

/// The number of pages of a PDF, for a file about to be merged.
pub fn pdf_page_total(session: &VaultSession, source: &str) -> Result<usize, ServiceError> {
    let bytes = read_source(&session.root, source)?;
    pdf_info(&bytes)
        .map(|i| i.page_count)
        .map_err(|e| invalid(describe_pdf_error(&e)))
}

/// Cut a PDF into one new file per range, next to it, named after the pages. Nothing is written if a name is taken.
pub fn split_pdf(
    session: &VaultSession,
    rel: &str,
    ranges: &[(usize, usize)],
) -> Result<Vec<String>, ServiceError> {
    let bytes = read_source(&session.root, rel)?;
    let stem = rel
        .strip_suffix(".pdf")
        .or_else(|| rel.strip_suffix(".PDF"))
        .unwrap_or(rel);
    let names: Vec<String> = ranges
        .iter()
        .map(|(a, b)| format!("{stem}-pages-{}-{}.pdf", a + 1, b + 1))
        .collect();
    let targets = names
        .iter()
        .map(|n| target_of(&session.root, n))
        .collect::<Result<Vec<_>, _>>()?;
    for (range, target) in ranges.iter().zip(&targets) {
        let part = pdf_extract_ranges(&bytes, std::slice::from_ref(range))
            .map_err(|e| invalid(describe_pdf_error(&e)))?;
        write_new(target, &part)?;
    }
    Ok(names)
}
