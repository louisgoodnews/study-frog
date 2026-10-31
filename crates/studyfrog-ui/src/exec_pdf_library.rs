//! Adding, listing and renaming PDFs on the worker.

use std::path::Path;

use studyfrog_service::pdf_library::{add_pdf, rename_pdf};
use studyfrog_service::pdfs::{index_pdfs, pdf_library};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::pdf_library_types::{LibraryJob, LibraryMsg};
use crate::pdf_types::PdfMsg;

fn reply(m: LibraryMsg) -> Msg {
    Msg::Pdf(PdfMsg::Library(m))
}

fn add(w: &mut Worker, folder: &str, now_ns: i64) -> LibraryMsg {
    let (Some(pick), Some(session)) = (w.file_picker, w.session.as_mut()) else {
        return LibraryMsg::Added(Err("this build has no file dialog".to_owned()));
    };
    let Some(source) = pick(&["pdf"]) else {
        return LibraryMsg::Added(Ok(None));
    };
    let result = add_pdf(session, Path::new(&source), folder)
        .and_then(|rel| index_pdfs(session, now_ns).map(|_| rel))
        .map(Some);
    LibraryMsg::Added(result.map_err(|e| e.to_string()))
}

/// Perform one library job.
pub fn run(w: &mut Worker, job: LibraryJob, now_ns: i64) -> Msg {
    reply(match job {
        LibraryJob::Load => LibraryMsg::Loaded(match &w.session {
            Some(s) => pdf_library(s).map_err(|e| e.to_string()),
            None => Ok(Vec::new()),
        }),
        LibraryJob::Add { folder } => add(w, &folder, now_ns),
        LibraryJob::Rename { path, name } => LibraryMsg::Renamed(match &mut w.session {
            Some(s) => rename_pdf(s, &path, &name)
                .and_then(|r| index_pdfs(s, now_ns).map(|_| r))
                .map_err(|e| e.to_string()),
            None => Err("no vault is open".to_owned()),
        }),
    })
}
