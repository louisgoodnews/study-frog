//! The "make a note from this PDF" form in the viewer. Pure.

use crate::effect::Effect;
use crate::pdf_extract_types::{ExtractForm, ExtractMsg, Made};
use crate::pdf_select::selected_text;
use crate::pdf_types::{PdfEffect, PdfView};
use crate::pdf_ui::{fail, with};
use crate::state::{Notice, NoticeKind, State};
use studyfrog_pdf::render::PageRect;
use studyfrog_store::pdf_notes::SourcedItem;

/// The effect that lists what was made from the PDF.
pub fn ask_sourced(path: &str) -> Effect {
    Effect::Pdf(PdfEffect::LoadSourced(path.to_owned()))
}

fn edit(state: State, f: impl FnOnce(ExtractForm) -> ExtractForm) -> (State, Vec<Effect>) {
    let view = state.pdf.clone().map(|v| PdfView {
        extract: f(v.extract.clone()),
        ..v
    });
    (State { pdf: view, ..state }, Vec::new())
}

fn make(state: State) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let form = view.extract.clone();
    if form.folder.is_empty() {
        return fail(state, "Choose the stack the note goes into.".to_owned());
    }
    let selection = view
        .text
        .as_ref()
        .filter(|_| form.use_selection && !view.selection.is_empty())
        .map(|t| selected_text(&t.words, &view.selection));
    if form.use_selection && selection.is_none() {
        return fail(state, "Select some words on the page first.".to_owned());
    }
    let ask = Effect::Pdf(PdfEffect::MakeNote {
        path: view.path.clone(),
        first: view.page,
        last: form.last.unwrap_or(view.page).max(view.page),
        selection,
        folder: form.folder.clone(),
        title: form.title.clone(),
        pictures: if form.include_pictures {
            form.pictures.clone()
        } else {
            Vec::new()
        },
    });
    let extract = ExtractForm { busy: true, ..form };
    (with(state, PdfView { extract, ..view }), vec![ask])
}

fn made(state: State, result: Made) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let extract = ExtractForm {
        busy: false,
        ..view.extract.clone()
    };
    match result {
        Ok(done) => {
            let (info, path) = *done;
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: format!("Made the note {path}."),
            });
            let ask = ask_sourced(&view.path);
            let state = State {
                vault: Some(info),
                notice,
                ..with(
                    state,
                    PdfView {
                        extract: ExtractForm {
                            title: String::new(),
                            ..extract
                        },
                        ..view
                    },
                )
            };
            (state, vec![ask])
        }
        Err(e) => {
            let (state, effects) = fail(state, format!("Could not make the note: {e}"));
            (with(state, PdfView { extract, ..view }), effects)
        }
    }
}

fn sourced(
    state: State,
    path: String,
    result: Result<Vec<SourcedItem>, String>,
) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone().filter(|v| v.path == path) else {
        return (state, Vec::new());
    };
    match result {
        Ok(items) => (
            with(
                state,
                PdfView {
                    sourced: items,
                    ..view
                },
            ),
            Vec::new(),
        ),
        Err(e) => fail(state, e),
    }
}

/// Handle the messages about making notes.
pub fn update(state: State, msg: ExtractMsg) -> (State, Vec<Effect>) {
    match msg {
        ExtractMsg::Folder(f) => edit(state, |x| ExtractForm { folder: f, ..x }),
        ExtractMsg::Last(p) => edit(state, |x| ExtractForm { last: Some(p), ..x }),
        ExtractMsg::UseSelection(on) => edit(state, |x| ExtractForm {
            use_selection: on,
            ..x
        }),
        ExtractMsg::Title(t) => edit(state, |x| ExtractForm { title: t, ..x }),
        ExtractMsg::SavePictures => ask_pictures(state),
        ExtractMsg::PicturesSaved(r) => pictures_saved(state, r),
        ExtractMsg::ToggleSnapshot => edit(state, |x| ExtractForm {
            snapshot_mode: !x.snapshot_mode,
            ..x
        }),
        ExtractMsg::Snapshot(rect) => snapshot(state, rect),
        ExtractMsg::SnapshotSaved(r) => snapshot_saved(state, r),
        ExtractMsg::IncludePictures(on) => edit(state, |x| ExtractForm {
            include_pictures: on,
            ..x
        }),
        ExtractMsg::Make => make(state),
        ExtractMsg::Made(r) => made(state, r),
        ExtractMsg::Sourced(path, r) => sourced(state, path, r),
    }
}

fn info(state: State, text: String) -> State {
    State {
        notice: Some(Notice {
            kind: NoticeKind::Info,
            text,
        }),
        ..state
    }
}

fn ask_pictures(state: State) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.as_ref() else {
        return (state, Vec::new());
    };
    let ask = Effect::Pdf(PdfEffect::SavePictures {
        path: view.path.clone(),
        page: view.page,
    });
    (state, vec![ask])
}

fn add_pictures(state: State, mut paths: Vec<String>) -> State {
    let view = state.pdf.clone().map(|v| {
        let mut all = v.extract.pictures.clone();
        paths.retain(|p| !all.contains(p));
        all.extend(paths);
        PdfView {
            extract: ExtractForm {
                pictures: all,
                ..v.extract.clone()
            },
            ..v
        }
    });
    State { pdf: view, ..state }
}

fn pictures_saved(state: State, result: Result<Vec<String>, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(paths) if paths.is_empty() => (
            info(state, "This page has no pictures to save.".to_owned()),
            Vec::new(),
        ),
        Ok(paths) => {
            let text = format!("Saved {} pictures into assets.", paths.len());
            (info(add_pictures(state, paths), text), Vec::new())
        }
        Err(e) => fail(state, format!("Could not save the pictures: {e}")),
    }
}

fn snapshot(state: State, rect: PageRect) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let extract = ExtractForm {
        snapshot_mode: false,
        ..view.extract.clone()
    };
    let ask = Effect::Pdf(PdfEffect::SaveRegion {
        path: view.path.clone(),
        page: view.page,
        rect,
    });
    (with(state, PdfView { extract, ..view }), vec![ask])
}

fn snapshot_saved(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(path) => (
            info(
                add_pictures(state, vec![path.clone()]),
                format!("Saved the snapshot as {path}."),
            ),
            Vec::new(),
        ),
        Err(e) => fail(state, format!("Could not save the snapshot: {e}")),
    }
}
