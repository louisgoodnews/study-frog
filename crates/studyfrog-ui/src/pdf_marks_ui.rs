//! Highlights and notes in the viewer: making, editing and removing them. Pure; every change asks
//! the worker to write the notes file.

use studyfrog_config::pdf_marks::{MARK_COLORS, MarkKind, MarkRect, PdfMark, next_mark_id};

use crate::effect::Effect;
use crate::pdf_mark_types::{MarkMsg, MarkSet};
use crate::pdf_select::{line_rects, selected_text};
use crate::pdf_types::{PdfEffect, PdfView};
use crate::pdf_ui::{fail, go_to, with};
use crate::state::State;

/// Size of the square a note occupies, in points.
pub const NOTE_SIZE: f32 = 16.0;

fn save(view: &PdfView) -> Vec<Effect> {
    vec![Effect::Pdf(PdfEffect::SaveMarks {
        path: view.path.clone(),
        marks: view.marks.clone(),
    })]
}

fn changed(state: State, view: PdfView) -> (State, Vec<Effect>) {
    let effects = save(&view);
    (with(state, view), effects)
}

fn loaded(
    state: State,
    path: String,
    result: Result<Box<MarkSet>, String>,
) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.clone().filter(|v| v.path == path) else {
        return (state, Vec::new());
    };
    match result {
        Ok(set) => {
            let (marks, marks_now) = *set;
            (
                with(
                    state,
                    PdfView {
                        marks,
                        marks_now,
                        ..view
                    },
                ),
                Vec::new(),
            )
        }
        Err(e) => fail(state, e),
    }
}

fn highlight(state: State) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let Some(text) = view.text.clone().filter(|_| !view.selection.is_empty()) else {
        return (state, Vec::new());
    };
    let covered = selected_text(&text.words, &view.selection).replace('\n', " ");
    let mark = PdfMark {
        id: next_mark_id(&view.marks),
        page: view.page,
        kind: MarkKind::Highlight,
        color: view.color.clone(),
        rects: line_rects(&text.words, &view.selection),
        covered,
        note: String::new(),
    };
    view.marks_now.push(Some(mark.page));
    view.marks.push(mark);
    view.selection.clear();
    view.side = crate::pdf_types::Side::Notes;
    changed(state, view)
}

fn place_note(state: State, x: f32, y: f32) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let mark = PdfMark {
        id: next_mark_id(&view.marks),
        page: view.page,
        kind: MarkKind::Note,
        color: view.color.clone(),
        rects: vec![MarkRect {
            x,
            y,
            width: NOTE_SIZE,
            height: NOTE_SIZE,
        }],
        covered: String::new(),
        note: String::new(),
    };
    view.marks_now.push(Some(mark.page));
    view.marks.push(mark);
    view.placing_note = false;
    view.side = crate::pdf_types::Side::Notes;
    changed(state, view)
}

fn edit(state: State, id: u32, note: String) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    match view.marks.iter_mut().find(|m| m.id == id) {
        Some(m) if m.note != note => m.note = note,
        _ => return (state, Vec::new()),
    }
    changed(state, view)
}

fn delete(state: State, id: u32) -> (State, Vec<Effect>) {
    let Some(mut view) = state.pdf.clone() else {
        return (state, Vec::new());
    };
    let Some(at) = view.marks.iter().position(|m| m.id == id) else {
        return (state, Vec::new());
    };
    view.marks.remove(at);
    if at < view.marks_now.len() {
        view.marks_now.remove(at);
    }
    changed(state, view)
}

fn jump(state: State, id: u32) -> (State, Vec<Effect>) {
    let Some(view) = state.pdf.as_ref() else {
        return (state, Vec::new());
    };
    let at = view.marks.iter().position(|m| m.id == id);
    let page = at.and_then(|i| {
        view.marks_now
            .get(i)
            .copied()
            .flatten()
            .or(Some(view.marks[i].page))
    });
    page.map_or((state.clone(), Vec::new()), |p| go_to(state, p))
}

/// Handle the messages about marks.
pub fn update(state: State, msg: MarkMsg) -> (State, Vec<Effect>) {
    match msg {
        MarkMsg::Loaded(path, result) => loaded(state, path, result),
        MarkMsg::Saved(Ok(())) => (state, Vec::new()),
        MarkMsg::Saved(Err(e)) => fail(state, format!("Could not save the notes: {e}")),
        MarkMsg::Highlight => highlight(state),
        MarkMsg::Color(name) => recolor(state, name),
        MarkMsg::ToggleNoteMode => toggle_note_mode(state),
        MarkMsg::PlaceNote(x, y) => place_note(state, x, y),
        MarkMsg::EditNote(id, note) => edit(state, id, note),
        MarkMsg::Delete(id) => delete(state, id),
        MarkMsg::Jump(id) => jump(state, id),
    }
}

fn recolor(state: State, name: String) -> (State, Vec<Effect>) {
    if !MARK_COLORS.iter().any(|(n, _)| *n == name) {
        return (state, Vec::new());
    }
    let view = state.pdf.clone().map(|v| PdfView { color: name, ..v });
    (State { pdf: view, ..state }, Vec::new())
}

fn toggle_note_mode(state: State) -> (State, Vec<Effect>) {
    let view = state.pdf.clone().map(|v| PdfView {
        placing_note: !v.placing_note,
        ..v
    });
    (State { pdf: view, ..state }, Vec::new())
}
