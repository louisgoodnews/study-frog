//! Adding, listing and renaming PDFs from the library. Pure.

use crate::effect::Effect;
use crate::pdf_library_types::{LibraryJob, LibraryMsg, PdfLibrary};
use crate::pdf_types::PdfEffect;
use crate::state::{Notice, NoticeKind, State};

/// The effect that lists the PDFs.
pub fn load() -> Effect {
    Effect::Pdf(PdfEffect::Library(LibraryJob::Load))
}

fn say(state: State, kind: NoticeKind, text: String) -> State {
    State {
        notice: Some(Notice { kind, text }),
        ..state
    }
}

fn with(state: State, f: impl FnOnce(PdfLibrary) -> PdfLibrary) -> State {
    let pdf_library = f(state.pdf_library.clone());
    State {
        pdf_library,
        ..state
    }
}

fn rename_to(state: &State) -> Option<(String, String)> {
    state
        .pdf_library
        .rename
        .clone()
        .filter(|(_, n)| !n.trim().is_empty())
}

/// Handle the messages about the library's PDFs.
pub fn update(state: State, msg: LibraryMsg) -> (State, Vec<Effect>) {
    match msg {
        LibraryMsg::Load => (state, vec![load()]),
        LibraryMsg::Loaded(Ok(rows)) => (with(state, |l| PdfLibrary { rows, ..l }), Vec::new()),
        LibraryMsg::Loaded(Err(e)) => (
            say(
                state,
                NoticeKind::Error,
                format!("Could not list the PDFs: {e}"),
            ),
            Vec::new(),
        ),
        LibraryMsg::AddTo(folder) if folder.is_empty() => (
            say(
                state,
                NoticeKind::Error,
                "Open a stack first, then add the PDF to it.".to_owned(),
            ),
            Vec::new(),
        ),
        LibraryMsg::AddTo(folder) => (
            state,
            vec![Effect::Pdf(PdfEffect::Library(LibraryJob::Add { folder }))],
        ),
        LibraryMsg::Added(Ok(Some(path))) => (
            say(
                state,
                NoticeKind::Info,
                format!("Added {path}. It is indexed in a moment."),
            ),
            vec![load()],
        ),
        LibraryMsg::Added(Ok(None)) => (state, Vec::new()),
        LibraryMsg::Added(Err(e)) => (
            say(
                state,
                NoticeKind::Error,
                format!("Could not add the PDF: {e}"),
            ),
            Vec::new(),
        ),
        LibraryMsg::StartRename(path) => {
            let name = path.rsplit('/').next().unwrap_or(&path).to_owned();
            (
                with(state, |l| PdfLibrary {
                    rename: Some((path, name)),
                    ..l
                }),
                Vec::new(),
            )
        }
        LibraryMsg::RenameEdit(name) => (
            with(state, |l| PdfLibrary {
                rename: l.rename.map(|(p, _)| (p, name)),
                ..l
            }),
            Vec::new(),
        ),
        LibraryMsg::CancelRename => (
            with(state, |l| PdfLibrary { rename: None, ..l }),
            Vec::new(),
        ),
        LibraryMsg::DoRename => match rename_to(&state) {
            Some((path, name)) => (
                state,
                vec![Effect::Pdf(PdfEffect::Library(LibraryJob::Rename {
                    path,
                    name: name.trim().to_owned(),
                }))],
            ),
            None => (state, Vec::new()),
        },
        LibraryMsg::Renamed(Ok((path, n))) => {
            let note = format!("Renamed to {path}; {n} files that mentioned it were updated.");
            let state = with(say(state, NoticeKind::Info, note), |l| PdfLibrary {
                rename: None,
                ..l
            });
            let state = State {
                pdf: state
                    .pdf
                    .clone()
                    .filter(|v| !v.path.is_empty() && v.path != path),
                ..state
            };
            (state, vec![load()])
        }
        LibraryMsg::Renamed(Err(e)) => (
            say(state, NoticeKind::Error, format!("Could not rename: {e}")),
            Vec::new(),
        ),
    }
}
