#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Highlights and notes in the viewer, written to the file next to the PDF.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{NOW_NS, run_to_idle, temp_dir};
use studyfrog_config::pdf_marks::{MarkKind, parse_marks};
use studyfrog_pdf::render::PageRect;
use studyfrog_pdf::text::WordBox;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_mark_types::MarkMsg;
use studyfrog_ui::pdf_select::{line_rects, words_in_rect};
use studyfrog_ui::pdf_types::{PdfMsg, Side};

fn viewer(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::write(dir.path.join("a.pdf"), pdfmake::three()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    let path = dir.path.display().to_string();
    run_to_idle(&mut model, &mut worker, vec![Effect::OpenVault(path)]);
    send(&mut model, &mut worker, PdfMsg::Open("a.pdf".into(), 1));
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

fn mark(model: &mut Model, worker: &mut Worker, m: MarkMsg) {
    send(model, worker, PdfMsg::Marks(m));
}

fn on_disk(dir: &common::TempDir) -> Vec<studyfrog_config::pdf_marks::PdfMark> {
    parse_marks(&fs::read_to_string(dir.path.join("a.pdf.notes.toml")).unwrap()).unwrap()
}

#[test]
fn a_selection_becomes_a_highlight_a_note_is_written_and_both_can_be_removed() {
    let (dir, mut model, mut worker) = viewer("marks");
    let words = model
        .state
        .pdf
        .as_ref()
        .unwrap()
        .text
        .as_ref()
        .unwrap()
        .words
        .clone();
    let everything = PageRect {
        x: 0.0,
        y: 0.0,
        width: 612.0,
        height: 792.0,
    };
    send(
        &mut model,
        &mut worker,
        PdfMsg::Select(words_in_rect(&words, &everything)),
    );
    mark(&mut model, &mut worker, MarkMsg::Color("green".into()));
    mark(&mut model, &mut worker, MarkMsg::Highlight);
    let saved = on_disk(&dir);
    assert_eq!(
        (saved.len(), saved[0].page, saved[0].kind),
        (1, 1, MarkKind::Highlight)
    );
    assert_eq!(
        (saved[0].color.as_str(), saved[0].covered.as_str()),
        ("green", "Beta second page")
    );
    let view = model.state.pdf.as_ref().unwrap();
    assert!(view.selection.is_empty());
    assert_eq!(view.side, Side::Notes);
    mark(
        &mut model,
        &mut worker,
        MarkMsg::EditNote(1, "check the proof".into()),
    );
    mark(&mut model, &mut worker, MarkMsg::ToggleNoteMode);
    assert!(model.state.pdf.as_ref().unwrap().placing_note);
    mark(&mut model, &mut worker, MarkMsg::PlaceNote(100.0, 200.0));
    let saved = on_disk(&dir);
    assert_eq!(
        (saved.len(), saved[0].note.as_str(), saved[1].kind),
        (2, "check the proof", MarkKind::Note)
    );
    assert!(!model.state.pdf.as_ref().unwrap().placing_note);
    mark(&mut model, &mut worker, MarkMsg::Delete(1));
    mark(&mut model, &mut worker, MarkMsg::Delete(2));
    assert!(!dir.path.join("a.pdf.notes.toml").exists());
}

#[test]
fn marks_come_back_when_the_pdf_is_opened_again_and_a_jump_goes_to_the_page() {
    let (dir, mut model, mut worker) = viewer("again");
    mark(&mut model, &mut worker, MarkMsg::PlaceNote(10.0, 10.0));
    send(&mut model, &mut worker, PdfMsg::Close);
    send(&mut model, &mut worker, PdfMsg::Open("a.pdf".into(), 0));
    let view = model.state.pdf.as_ref().unwrap();
    assert_eq!((view.marks.len(), view.page), (1, 0));
    mark(&mut model, &mut worker, MarkMsg::Jump(1));
    assert_eq!(model.state.pdf.as_ref().unwrap().page, 1);
    assert!(dir.path.join("a.pdf.notes.toml").exists());
    let _ = NOW_NS;
}

#[test]
fn a_damaged_notes_file_gives_a_notice_and_does_not_stop_the_viewer() {
    let dir = temp_dir("damaged");
    fs::write(dir.path.join("a.pdf"), pdfmake::three()).unwrap();
    fs::write(dir.path.join("a.pdf.notes.toml"), "[[mark]]\nid = 1\n").unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    send(&mut model, &mut worker, PdfMsg::Open("a.pdf".into(), 0));
    let notice = model.state.notice.as_ref().unwrap();
    assert!(notice.text.contains("a.pdf.notes.toml"), "{}", notice.text);
    let view = model.state.pdf.as_ref().unwrap();
    assert!(view.info.is_some() && view.marks.is_empty());
}

#[test]
fn selected_words_make_one_rectangle_per_line() {
    let w = |t: &str, x: f32, y: f32| WordBox {
        text: t.into(),
        x,
        y,
        width: 30.0,
        height: 10.0,
    };
    let words = [w("a", 10.0, 10.0), w("b", 50.0, 11.0), w("c", 10.0, 40.0)];
    let rects = line_rects(&words, &[0, 1, 2]);
    assert_eq!(rects.len(), 2);
    assert_eq!((rects[0].x, rects[0].width), (10.0, 70.0));
    assert!(line_rects(&words, &[]).is_empty());
}
