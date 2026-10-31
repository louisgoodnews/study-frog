#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Rearranging a PDF into a new file from the viewer.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::sync::Arc;

use common::{run_to_idle, temp_dir};
use studyfrog_pdf::doc::pdf_info;
use studyfrog_pdf::text::pdf_page_text;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_arrange_types::{ArrangeMsg, Prop};
use studyfrog_ui::pdf_types::PdfMsg;

fn viewer(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::write(dir.path.join("a.pdf"), pdfmake::three()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    send(&mut model, &mut worker, PdfMsg::Open("a.pdf".into(), 0));
    send(&mut model, &mut worker, PdfMsg::Arrange(ArrangeMsg::Start));
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

fn arr(model: &mut Model, worker: &mut Worker, m: ArrangeMsg) {
    send(model, worker, PdfMsg::Arrange(m));
}

fn pages(model: &Model) -> Vec<(usize, i32)> {
    let a = model.state.pdf.as_ref().unwrap().arrange.as_ref().unwrap();
    a.plan.iter().map(|s| (s.page, s.rotate)).collect()
}

#[test]
fn pages_are_turned_moved_dropped_and_changes_can_be_undone() {
    let (_d, mut model, mut worker) = viewer("edit");
    assert_eq!(pages(&model), [(0, 0), (1, 0), (2, 0)]);
    arr(&mut model, &mut worker, ArrangeMsg::Select(0));
    arr(&mut model, &mut worker, ArrangeMsg::Rotate(90));
    arr(&mut model, &mut worker, ArrangeMsg::Move(1));
    assert_eq!(pages(&model), [(1, 0), (0, 90), (2, 0)]);
    arr(&mut model, &mut worker, ArrangeMsg::Move(-5));
    arr(&mut model, &mut worker, ArrangeMsg::Delete);
    assert_eq!(pages(&model), [(1, 0), (2, 0)]);
    arr(&mut model, &mut worker, ArrangeMsg::Undo);
    assert_eq!(pages(&model), [(1, 0), (0, 90), (2, 0)]);
    arr(&mut model, &mut worker, ArrangeMsg::Undo);
    assert_eq!(
        pages(&model),
        [(0, 90), (1, 0), (2, 0)],
        "each undo goes one change back"
    );
    arr(&mut model, &mut worker, ArrangeMsg::Reset);
    assert_eq!(pages(&model), [(0, 0), (1, 0), (2, 0)]);
    arr(&mut model, &mut worker, ArrangeMsg::Undo);
    assert_eq!(
        pages(&model),
        [(0, 90), (1, 0), (2, 0)],
        "even a reset can be undone"
    );
}

#[test]
fn the_last_page_cannot_be_dropped_and_turning_wraps_round() {
    let (_d, mut model, mut worker) = viewer("last");
    arr(&mut model, &mut worker, ArrangeMsg::Select(0));
    for _ in 0..2 {
        arr(&mut model, &mut worker, ArrangeMsg::Delete);
    }
    arr(&mut model, &mut worker, ArrangeMsg::Delete);
    assert_eq!(pages(&model).len(), 1);
    arr(&mut model, &mut worker, ArrangeMsg::Rotate(-90));
    assert_eq!(pages(&model)[0].1, 270);
}

#[test]
fn saving_writes_a_new_file_with_the_plan_and_properties_and_leaves_the_original() {
    let (dir, mut model, mut worker) = viewer("save");
    let original = fs::read(dir.path.join("a.pdf")).unwrap();
    arr(&mut model, &mut worker, ArrangeMsg::Select(2));
    arr(&mut model, &mut worker, ArrangeMsg::Move(-2));
    arr(&mut model, &mut worker, ArrangeMsg::Name("b.pdf".into()));
    arr(
        &mut model,
        &mut worker,
        ArrangeMsg::Property(Prop::Title, "Reordered".into()),
    );
    arr(&mut model, &mut worker, ArrangeMsg::Save);
    assert!(model.state.pdf.as_ref().unwrap().arrange.is_none());
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Saved 3 pages to b.pdf")
    );
    let bytes = Arc::new(fs::read(dir.path.join("b.pdf")).unwrap());
    assert_eq!(
        pdf_info(&bytes).unwrap().title.as_deref(),
        Some("Reordered")
    );
    assert_eq!(pdf_page_text(&bytes, 0).unwrap().words[0].text, "Gamma");
    assert_eq!(fs::read(dir.path.join("a.pdf")).unwrap(), original);
    arr(&mut model, &mut worker, ArrangeMsg::Start);
    arr(&mut model, &mut worker, ArrangeMsg::Name("b.pdf".into()));
    arr(&mut model, &mut worker, ArrangeMsg::Save);
    let a = model.state.pdf.as_ref().unwrap().arrange.as_ref().unwrap();
    assert!(a.message.as_deref().unwrap().contains("already exists") && !a.busy);
}

#[test]
fn splitting_checks_the_ranges_and_writes_one_file_per_range() {
    let (dir, mut model, mut worker) = viewer("split");
    arr(&mut model, &mut worker, ArrangeMsg::Ranges("1-2, 9".into()));
    arr(&mut model, &mut worker, ArrangeMsg::Split);
    let a = model.state.pdf.as_ref().unwrap().arrange.as_ref().unwrap();
    assert!(a.message.is_some());
    assert!(!dir.path.join("a-pages-1-2.pdf").exists());
    arr(&mut model, &mut worker, ArrangeMsg::Ranges("1-2, 3".into()));
    arr(&mut model, &mut worker, ArrangeMsg::Split);
    assert!(dir.path.join("a-pages-1-2.pdf").exists() && dir.path.join("a-pages-3-3.pdf").exists());
    assert!(model.state.pdf.as_ref().unwrap().arrange.is_none());
}

#[test]
fn another_pdf_is_appended_and_a_missing_file_dialog_is_reported() {
    let (dir, mut model, mut worker) = viewer("merge");
    let other = dir.path.join("other.pdf");
    fs::write(&other, pdfmake::make(&[vec![("Delta", 72.0, 700.0)]], None)).unwrap();
    arr(&mut model, &mut worker, ArrangeMsg::AddFile);
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("no file dialog")
    );
    arr(
        &mut model,
        &mut worker,
        ArrangeMsg::FileAdded(Ok(Some((other.display().to_string(), 1)))),
    );
    assert_eq!(pages(&model).len(), 4);
    arr(&mut model, &mut worker, ArrangeMsg::Name("m.pdf".into()));
    arr(&mut model, &mut worker, ArrangeMsg::Save);
    let bytes = Arc::new(fs::read(dir.path.join("m.pdf")).unwrap());
    assert_eq!(pdf_page_text(&bytes, 3).unwrap().words[0].text, "Delta");
}

#[test]
fn leaving_without_saving_writes_nothing() {
    let (dir, mut model, mut worker) = viewer("cancel");
    arr(&mut model, &mut worker, ArrangeMsg::Cancel);
    assert!(model.state.pdf.as_ref().unwrap().arrange.is_none());
    assert!(!dir.path.join("a-edited.pdf").exists());
}
