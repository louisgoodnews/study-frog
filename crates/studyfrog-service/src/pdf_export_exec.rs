//! Safe atomic execution of PDF exports with progress reporting and cancellation.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::write::geometry::PageGeometry;
use studyfrog_pdf::write::{LayoutOptions, layout_document, write_pdf};

use crate::folder_ops_path::is_hostile_segment;
use crate::pdf_export_doc::convert_doc_to_blocks;
use crate::pdf_export_plan::plan_pdf_export;
use crate::pdf_export_scope::resolve_target_scope;
use crate::pdf_export_types::{
    ExportProgress, ExportTarget, PdfExportError, PdfExportOptions, PdfExportOutcome,
};
use crate::session::VaultSession;

fn is_hostile_path(dest: &Path, session: &VaultSession) -> Result<(), PdfExportError> {
    let path_str = dest.to_string_lossy();
    if path_str.trim().is_empty() || path_str.contains('\0') {
        return Err(PdfExportError::HostilePath(
            "empty or null byte".to_string(),
        ));
    }

    let ext = dest
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    if ext.as_deref() != Some("pdf") {
        return Err(PdfExportError::HostilePath(
            "file must end with .pdf".to_string(),
        ));
    }

    let file_stem = dest
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if is_hostile_segment(file_stem) {
        return Err(PdfExportError::HostilePath(format!(
            "invalid file name: {file_stem}"
        )));
    }

    if let Ok(rel) = dest.strip_prefix(&session.root) {
        let rel_str = rel.to_string_lossy();
        if rel_str == ".studyfrog" || rel_str.starts_with(".studyfrog/") {
            return Err(PdfExportError::HostilePath(
                "cannot write into internal index directory".to_string(),
            ));
        }
    }

    Ok(())
}

fn check_cancelled(cancel: Option<&AtomicBool>) -> Result<(), PdfExportError> {
    if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
        return Err(PdfExportError::Cancelled);
    }
    Ok(())
}

fn atomic_write_pdf(dest: &Path, bytes: &[u8]) -> Result<(), PdfExportError> {
    let parent = dest.parent().unwrap_or_else(|| Path::new("."));
    if !parent.as_os_str().is_empty() && !parent.exists() {
        fs::create_dir_all(parent)
            .map_err(|e| PdfExportError::Io(format!("could not create directory: {e}")))?;
    }

    let temp_name = format!(
        ".tmp-export-{}-{}.pdf",
        std::process::id(),
        Utc::now().timestamp_millis()
    );
    let temp_path = parent.join(temp_name);

    let write_res = (|| -> Result<(), std::io::Error> {
        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, dest)?;
        Ok(())
    })();

    if let Err(e) = write_res {
        let _ = fs::remove_file(&temp_path);
        return Err(PdfExportError::Io(format!(
            "failed writing destination {}: {e}",
            dest.display()
        )));
    }

    Ok(())
}

/// Execute a PDF export safely, writing atomically to the destination path.
pub fn execute_pdf_export(
    session: &VaultSession,
    target: &ExportTarget,
    options: &PdfExportOptions,
    destination: &Path,
    cancel: Option<&AtomicBool>,
    mut on_progress: Option<&mut dyn FnMut(ExportProgress)>,
) -> Result<PdfExportOutcome, PdfExportError> {
    is_hostile_path(destination, session)?;
    check_cancelled(cancel)?;

    let scope = resolve_target_scope(session, target)?;
    let total = scope.uids.len();

    let mut warnings = Vec::new();
    for (processed, uid) in scope.uids.iter().enumerate() {
        check_cancelled(cancel)?;
        if let Some(ref mut cb) = on_progress {
            cb(ExportProgress {
                current: processed,
                total,
            });
        }
        let _ = convert_doc_to_blocks(
            session,
            *uid,
            &Default::default(),
            options.show_answers,
            &mut warnings,
        );
    }

    if let Some(ref mut cb) = on_progress {
        cb(ExportProgress {
            current: total,
            total,
        });
    }

    let plan = plan_pdf_export(session, target, options)?;
    warnings.extend(plan.warnings);

    check_cancelled(cancel)?;

    let layout_options = LayoutOptions {
        geometry: PageGeometry {
            size: options.page_size,
            margins: options.margins,
            header_height: 36.0,
            footer_height: 36.0,
        },
        title: Some(plan.title.clone()),
        author: Some("StudyFrog".to_string()),
        vault_name: session
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .map(ToString::to_string),
        page_numbers: options.page_numbers,
        font_size: options.font_size,
    };

    let paged = layout_document(&plan.blocks, &layout_options);
    let page_count = paged.pages.len();

    check_cancelled(cancel)?;

    let pdf_bytes = write_pdf(&paged, &layout_options)
        .map_err(|e| PdfExportError::Pdf(describe_pdf_error(&e)))?;

    check_cancelled(cancel)?;

    atomic_write_pdf(destination, &pdf_bytes)?;

    Ok(PdfExportOutcome {
        destination: destination.to_path_buf(),
        bytes_written: pdf_bytes.len(),
        pages: page_count,
        item_count: total,
        warnings,
    })
}
