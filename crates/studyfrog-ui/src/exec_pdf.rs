//! Performing the PDF viewer's effects. The file is read once and kept for the page requests that follow.

use std::sync::Arc;

use studyfrog_pdf::PdfBytes;
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::error::describe_pdf_error;
use studyfrog_pdf::render::pdf_render_page;
use studyfrog_pdf::text::pdf_page_text;
use studyfrog_service::pdfs::find_pdfs;
use studyfrog_store::pdf_search::pages_containing;

use crate::chrome::{ChromeMsg, MAX_RESULTS};
use crate::exec::Worker;
use crate::msg::Msg;
use crate::pdf_arrange_types::ArrangeMsg;
use crate::pdf_extract_types::ExtractMsg;
use crate::pdf_mark_types::MarkMsg;
use crate::pdf_types::{PageKey, PdfEffect, PdfMsg};
use studyfrog_config::pdf_marks::PdfMark;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::pdf_marks::{load_marks, mark_pages_now, save_marks};
use studyfrog_service::pdf_notes::{NoteRequest, note_from_pdf};
use studyfrog_service::pdf_ops::{pdf_page_total, save_pdf_copy, split_pdf};
use studyfrog_service::pdf_pictures::{save_page_pictures, save_page_region};
use studyfrog_service::session::ServiceError;
use studyfrog_store::pdf_notes::items_made_from;
use studyfrog_store::pdf_search::scanned_pages;

/// A path inside the vault: relative, no `..`, ends in `.pdf`.
fn is_vault_pdf(rel: &str) -> bool {
    let clean = !rel.starts_with('/') && !rel.split('/').any(|p| p == ".." || p.is_empty());
    clean && rel.to_lowercase().ends_with(".pdf")
}

/// The bytes of `rel`, from the cache when it is the file read last.
fn bytes_of(w: &mut Worker, rel: &str) -> Result<PdfBytes, String> {
    if let Some((cached, bytes)) = &w.pdf_bytes
        && cached == rel
    {
        return Ok(bytes.clone());
    }
    let session = w.session.as_ref().ok_or("no vault is open")?;
    let path = if is_vault_pdf(rel) {
        session.root.join(rel)
    } else {
        std::path::PathBuf::from(rel)
    };
    let data = std::fs::read(&path).map_err(|e| format!("cannot read the file: {e}"))?;
    let bytes: PdfBytes = Arc::new(data);
    w.pdf_bytes = Some((rel.to_owned(), bytes.clone()));
    Ok(bytes)
}

/// Read the structure of a PDF.
pub fn open(w: &mut Worker, rel: &str) -> Msg {
    w.pdf_bytes = None;
    let result = bytes_of(w, rel)
        .and_then(|b| pdf_info(&b).map_err(|e| describe_pdf_error(&e)))
        .map(Box::new);
    Msg::Pdf(PdfMsg::Opened(rel.to_owned(), result))
}

/// Draw one page.
pub fn render(w: &mut Worker, rel: &str, key: PageKey) -> Msg {
    let scale = key.scale_milli as f32 / 1000.0;
    let result = bytes_of(w, rel)
        .and_then(|b| pdf_render_page(&b, key.page, scale).map_err(|e| describe_pdf_error(&e)))
        .map(Arc::new);
    Msg::Pdf(PdfMsg::Rendered(rel.to_owned(), key, result))
}

/// Read the words of one page.
pub fn text(w: &mut Worker, rel: &str, page: usize) -> Msg {
    let result = bytes_of(w, rel)
        .and_then(|b| pdf_page_text(&b, page).map_err(|e| describe_pdf_error(&e)))
        .map(Arc::new);
    Msg::Pdf(PdfMsg::TextLoaded(rel.to_owned(), page, result))
}

/// The pages of one PDF that contain a text, from the index.
pub fn find(w: &Worker, rel: &str, query: &str) -> Msg {
    let result = match &w.session {
        Some(s) => pages_containing(&s.conn, rel, query).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::Pdf(PdfMsg::Found(rel.to_owned(), query.to_owned(), result))
}

/// PDFs and pages for the global search.
pub fn search(w: &Worker, query: String) -> Msg {
    let result = match &w.session {
        Some(s) => find_pdfs(s, &query, MAX_RESULTS).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::Chrome(ChromeMsg::PdfResults(query, result))
}

/// Perform one of the viewer's effects.
pub fn run(w: &mut Worker, effect: PdfEffect, now_ns: i64) -> Msg {
    match effect {
        PdfEffect::Open(path) => open(w, &path),
        PdfEffect::Render { path, key } => render(w, &path, key),
        PdfEffect::Text { path, page } => text(w, &path, page),
        PdfEffect::Find { path, query } => find(w, &path, &query),
        PdfEffect::Search(query) => search(w, query),
        PdfEffect::MakeNote {
            path,
            first,
            last,
            selection,
            folder,
            title,
            pictures,
        } => make_note(
            w,
            now_ns,
            &path,
            (first, last),
            (selection.as_deref(), &folder, &title, &pictures),
        ),
        PdfEffect::SavePictures { path, page } => {
            Msg::Pdf(PdfMsg::Extract(ExtractMsg::PicturesSaved(
                w.session
                    .as_ref()
                    .ok_or("no vault is open".to_owned())
                    .and_then(|s| save_page_pictures(s, &path, page).map_err(|e| e.to_string())),
            )))
        }
        PdfEffect::SaveRegion { path, page, rect } => {
            Msg::Pdf(PdfMsg::Extract(ExtractMsg::SnapshotSaved(
                w.session
                    .as_ref()
                    .ok_or("no vault is open".to_owned())
                    .and_then(|s| {
                        save_page_region(s, &path, page, rect).map_err(|e| e.to_string())
                    }),
            )))
        }
        PdfEffect::Library(job) => crate::exec_pdf_library::run(w, job, now_ns),
        PdfEffect::MakeItem(job) => crate::exec_pdf_items::make(w, &job, now_ns),
        PdfEffect::LoadScanned(path) => {
            let result = match &w.session {
                Some(s) => scanned_pages(&s.conn, &path).map_err(|e| e.to_string()),
                None => Ok(Vec::new()),
            };
            Msg::Pdf(PdfMsg::Scanned(path, result))
        }
        PdfEffect::LoadSourced(path) => sourced(w, &path),
        PdfEffect::PickMerge => pick_merge(w),
        PdfEffect::SaveCopy(req) => Msg::Pdf(PdfMsg::Arrange(ArrangeMsg::Saved(
            w.session
                .as_ref()
                .ok_or("no vault is open".to_owned())
                .and_then(|s| save_pdf_copy(s, &req).map_err(|e| e.to_string())),
        ))),
        PdfEffect::Split { path, ranges } => Msg::Pdf(PdfMsg::Arrange(ArrangeMsg::Splitted(
            w.session
                .as_ref()
                .ok_or("no vault is open".to_owned())
                .and_then(|s| split_pdf(s, &path, &ranges).map_err(|e| e.to_string())),
        ))),
        PdfEffect::LoadMarks(path) => load_marks_msg(w, &path),
        PdfEffect::SaveMarks { path, marks } => save_marks_msg(w, &path, &marks),
        PdfEffect::Export(job) => crate::exec_pdf_export::run(w, job),
    }
}

fn load_marks_msg(w: &Worker, rel: &str) -> Msg {
    let result = match &w.session {
        Some(s) => load_marks(s, rel).and_then(|marks| {
            let now = mark_pages_now(s, rel, &marks)?;
            Ok(Box::new((marks, now)))
        }),
        None => Err(ServiceError::Invalid("no vault is open".to_owned())),
    };
    Msg::Pdf(PdfMsg::Marks(MarkMsg::Loaded(
        rel.to_owned(),
        result.map_err(|e| e.to_string()),
    )))
}

fn save_marks_msg(w: &mut Worker, rel: &str, marks: &[PdfMark]) -> Msg {
    let result = match &mut w.session {
        Some(s) => save_marks(s, rel, marks).map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Pdf(PdfMsg::Marks(MarkMsg::Saved(result)))
}

/// Ask for a PDF with the native dialog and count its pages.
fn pick_merge(w: &Worker) -> Msg {
    let (Some(pick), Some(session)) = (w.file_picker, w.session.as_ref()) else {
        return Msg::Pdf(PdfMsg::Arrange(ArrangeMsg::FileAdded(Err(
            "this build has no file dialog".to_owned(),
        ))));
    };
    let result = match pick(&["pdf"]) {
        None => Ok(None),
        Some(path) => {
            let path = path.display().to_string();
            pdf_page_total(session, &path)
                .map(|n| Some((path, n)))
                .map_err(|e| e.to_string())
        }
    };
    Msg::Pdf(PdfMsg::Arrange(ArrangeMsg::FileAdded(result)))
}

fn make_note(
    w: &mut Worker,
    now_ns: i64,
    pdf: &str,
    pages: (usize, usize),
    (selection, folder, title, pictures): (Option<&str>, &str, &str, &[String]),
) -> Msg {
    let result = match &mut w.session {
        Some(s) => fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
            .and_then(|ids| {
                let req = NoteRequest {
                    pdf,
                    first: pages.0,
                    last: pages.1,
                    selection,
                    folder,
                    title,
                    pictures,
                };
                note_from_pdf(s, &req, ids)
            })
            .map(|(info, made)| Box::new((info, made.path)))
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Pdf(PdfMsg::Extract(ExtractMsg::Made(result)))
}

fn sourced(w: &Worker, pdf: &str) -> Msg {
    let result = match &w.session {
        Some(s) => items_made_from(&s.conn, pdf, None).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    };
    Msg::Pdf(PdfMsg::Extract(ExtractMsg::Sourced(pdf.to_owned(), result)))
}
