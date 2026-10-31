//! Writing a card or a question from a highlighted passage. Pure.

use crate::effect::Effect;
use crate::pdf_draft_types::{Draft, DraftKind, DraftMsg, ItemJob, ItemMade};
use crate::pdf_select::selected_text;
use crate::pdf_types::{PdfEffect, PdfView};
use crate::pdf_ui::{fail, with};
use crate::state::{Notice, NoticeKind, State};

fn passage(view: &PdfView, mark: Option<u32>) -> Option<(String, String, usize)> {
    match mark {
        Some(id) => {
            let m = view.marks.iter().find(|m| m.id == id)?;
            Some((m.note.clone(), m.covered.clone(), m.page))
        }
        None => {
            let text = view.text.as_ref().filter(|_| !view.selection.is_empty())?;
            Some((
                String::new(),
                selected_text(&text.words, &view.selection).replace('\n', " "),
                view.page,
            ))
        }
    }
}

fn start(state: State, kind: DraftKind, mark: Option<u32>) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let Some((front, back, page)) = passage(&view, mark) else {
        return fail(
            state,
            "Select some words or highlight a passage first.".to_owned(),
        );
    };
    let draft = Some(Draft {
        kind,
        front,
        back,
        page,
        busy: false,
    });
    let side = crate::pdf_types::Side::Extract;
    (
        with(
            state,
            PdfView {
                draft,
                side,
                ..view
            },
        ),
        Vec::new(),
    )
}

fn edit(state: State, f: impl FnOnce(Draft) -> Draft) -> (State, Vec<Effect>) {
    let view = state.pdf.clone().map(|v| PdfView {
        draft: v.draft.clone().map(f),
        ..v
    });
    (State { pdf: view, ..state }, Vec::new())
}

fn create(state: State) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let Some(draft) = view.draft.clone() else {
        return (state, Vec::new());
    };
    if view.extract.folder.is_empty() {
        return fail(state, "Choose the stack it goes into.".to_owned());
    }
    let job = ItemJob {
        path: view.path.clone(),
        kind: draft.kind,
        page: draft.page,
        front: draft.front.clone(),
        back: draft.back.clone(),
        folder: view.extract.folder.clone(),
    };
    let draft = Some(Draft {
        busy: true,
        ..draft
    });
    (
        with(state, PdfView { draft, ..view }),
        vec![Effect::Pdf(PdfEffect::MakeItem(job))],
    )
}

fn made(state: State, result: ItemMade) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    match result {
        Ok(done) => {
            let (info, path) = *done;
            let notice = Some(Notice {
                kind: NoticeKind::Info,
                text: format!("Made {path}."),
            });
            let ask = crate::pdf_extract_ui::ask_sourced(&view.path);
            (
                State {
                    vault: Some(info),
                    notice,
                    ..with(
                        state,
                        PdfView {
                            draft: None,
                            ..view
                        },
                    )
                },
                vec![ask],
            )
        }
        Err(e) => {
            let draft = view.draft.clone().map(|d| Draft { busy: false, ..d });
            let (state, effects) = fail(state, format!("Could not make it: {e}"));
            (with(state, PdfView { draft, ..view }), effects)
        }
    }
}

/// Handle the messages about the draft.
pub fn update(state: State, msg: DraftMsg) -> (State, Vec<Effect>) {
    match msg {
        DraftMsg::Start(kind, mark) => start(state, kind, mark),
        DraftMsg::Front(t) => edit(state, |d| Draft { front: t, ..d }),
        DraftMsg::Back(t) => edit(state, |d| Draft { back: t, ..d }),
        DraftMsg::Create => create(state),
        DraftMsg::Cancel => (
            State {
                pdf: state.pdf.clone().map(|v| PdfView { draft: None, ..v }),
                ..state
            },
            Vec::new(),
        ),
        DraftMsg::Made(r) => made(state, r),
    }
}
