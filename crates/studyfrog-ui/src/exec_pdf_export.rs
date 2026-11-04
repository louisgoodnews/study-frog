//! Worker execution of PDF export jobs: preview generation, file dialogs, and atomic exports.

use std::path::Path;
use std::sync::Arc;

use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::render::pdf_render_page;
use studyfrog_pdf::write::geometry::PageGeometry;
use studyfrog_pdf::write::primitives::{LayoutItem, PagedDocument};
use studyfrog_pdf::write::{LayoutOptions, layout_document, write_pdf};
use studyfrog_service::pdf_export::{
    ExportTarget, PdfExportOptions, PdfExportOutcome, execute_pdf_export, plan_pdf_export,
};
use studyfrog_service::session::VaultSession;

use crate::exec::Worker;
use crate::msg::Msg;
use crate::pdf_export_types::{PdfExportJob, PdfExportMsg, PdfExportPreview, PreviewPage};
use crate::pdf_types::PdfMsg;

fn extract_page_info(paged: &PagedDocument, idx: usize) -> (Vec<String>, String) {
    let Some(page) = paged.pages.get(idx) else {
        return (Vec::new(), String::new());
    };
    let mut headings = Vec::new();
    let mut lines = Vec::new();
    for item in &page.items {
        match item {
            LayoutItem::Heading(h) => headings.push(h.title.clone()),
            LayoutItem::Text(l) => {
                let text: String = l.runs.iter().map(|r| r.text.as_str()).collect();
                if !text.trim().is_empty() && lines.len() < 3 {
                    lines.push(text);
                }
            }
            _ => {}
        }
    }
    (headings, lines.join(" "))
}

fn generate_preview_pages(
    paged: &PagedDocument,
    pdf_bytes: &[u8],
    total_pages: usize,
) -> Vec<PreviewPage> {
    let arc_bytes = Arc::new(pdf_bytes.to_vec());
    let mut pages = Vec::new();
    for idx in 0..total_pages.min(3) {
        let (headings, snippet) = extract_page_info(paged, idx);
        let pixels = pdf_render_page(&arc_bytes, idx, 0.25).ok();
        pages.push(PreviewPage {
            page_number: idx + 1,
            headings,
            snippet,
            pixels,
        });
    }
    pages
}

fn build_layout_options(
    session: &VaultSession,
    title: &str,
    options: &PdfExportOptions,
) -> LayoutOptions {
    LayoutOptions {
        geometry: PageGeometry {
            size: options.page_size,
            margins: options.margins,
            header_height: 36.0,
            footer_height: 36.0,
        },
        title: Some(title.to_string()),
        author: Some("StudyFrog".to_string()),
        vault_name: session
            .root
            .file_name()
            .and_then(|n| n.to_str())
            .map(ToString::to_string),
        page_numbers: options.page_numbers,
        font_size: options.font_size,
    }
}

/// Generate a pure preview of the export document without writing files.
pub fn generate_preview(
    session: &VaultSession,
    target: &ExportTarget,
    options: &PdfExportOptions,
) -> Result<PdfExportPreview, String> {
    let plan = plan_pdf_export(session, target, options).map_err(|e| e.to_string())?;
    let layout_options = build_layout_options(session, &plan.title, options);
    let paged = layout_document(&plan.blocks, &layout_options);
    let total_pages = paged.pages.len();
    let pdf_bytes =
        write_pdf(&paged, &layout_options).map_err(|e| describe_pdf_error(&e).to_string())?;

    let pages = generate_preview_pages(&paged, &pdf_bytes, total_pages);
    Ok(PdfExportPreview {
        total_pages,
        item_count: plan.items.len(),
        title: plan.title,
        warnings: plan.warnings,
        pages,
    })
}

/// Execute the export operation and return the outcome.
pub fn run_export(
    session: &VaultSession,
    target: &ExportTarget,
    options: &PdfExportOptions,
    destination: &Path,
) -> Result<PdfExportOutcome, String> {
    execute_pdf_export(session, target, options, destination, None, None).map_err(|e| e.to_string())
}

/// Dispatch a PDF export job on the worker thread.
pub fn run(w: &mut Worker, job: PdfExportJob) -> Msg {
    match job {
        PdfExportJob::Browse => {
            let picked = w.file_picker.and_then(|pick| pick(&["pdf"]));
            Msg::Pdf(PdfMsg::Export(PdfExportMsg::DestinationPicked(
                picked.map(|p| p.display().to_string()),
            )))
        }
        PdfExportJob::Preview { target, options } => {
            let res = match &w.session {
                Some(session) => generate_preview(session, &target, &options),
                None => Err("no vault is open".to_string()),
            };
            Msg::Pdf(PdfMsg::Export(PdfExportMsg::PreviewReady(res)))
        }
        PdfExportJob::Export {
            target,
            options,
            destination,
        } => {
            let res = match &w.session {
                Some(session) => run_export(session, &target, &options, &destination),
                None => Err("no vault is open".to_string()),
            };
            Msg::Pdf(PdfMsg::Export(PdfExportMsg::Exported(res)))
        }
    }
}
