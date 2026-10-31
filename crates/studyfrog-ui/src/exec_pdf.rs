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
use crate::pdf_types::{PageKey, PdfEffect, PdfMsg};

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
    if !is_vault_pdf(rel) {
        return Err("only PDF files inside the vault can be opened".to_owned());
    }
    let data =
        std::fs::read(session.root.join(rel)).map_err(|e| format!("cannot read the file: {e}"))?;
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
pub fn run(w: &mut Worker, effect: PdfEffect) -> Msg {
    match effect {
        PdfEffect::Open(path) => open(w, &path),
        PdfEffect::Render { path, key } => render(w, &path, key),
        PdfEffect::Text { path, page } => text(w, &path, page),
        PdfEffect::Find { path, query } => find(w, &path, &query),
        PdfEffect::Search(query) => search(w, query),
    }
}
