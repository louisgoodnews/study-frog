#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Accessibility audit of the PDF screens: every control is named, the page is described and its
//! text is available.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::views::{assert_all_named, harness};
use common::{NOW_NS, run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_arrange_types::ArrangeMsg;
use studyfrog_ui::pdf_draft_types::{DraftKind, DraftMsg};
use studyfrog_ui::pdf_mark_types::MarkMsg;
use studyfrog_ui::pdf_types::{PdfMsg, Side};

fn viewer(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(
        dir.path.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(dir.path.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    studyfrog_service::pdfs::index_pdfs_fully(worker.session.as_mut().unwrap(), NOW_NS).unwrap();
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 1),
    );
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

fn audit(model: Model, what: &str) {
    let mut h = harness(model);
    h.run_steps(3);
    assert_all_named(&h, what);
}

#[test]
fn every_control_of_the_viewer_and_its_panels_has_a_name() {
    for side in [
        Side::Outline,
        Side::Pages,
        Side::Notes,
        Side::Extract,
        Side::Closed,
    ] {
        let (_d, mut model, mut worker) = viewer("a11y-side");
        send(&mut model, &mut worker, PdfMsg::Select(vec![0]));
        send(&mut model, &mut worker, PdfMsg::Marks(MarkMsg::Highlight));
        send(&mut model, &mut worker, PdfMsg::Query("beta".into()));
        send(&mut model, &mut worker, PdfMsg::ShowSide(side));
        audit(model, &format!("viewer {side:?}"));
    }
}

#[test]
fn the_draft_form_and_the_arrange_screen_are_named_too() {
    let (_d, mut model, mut worker) = viewer("a11y-draft");
    send(&mut model, &mut worker, PdfMsg::Select(vec![0]));
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Start(DraftKind::Card, None)),
    );
    audit(model, "draft form");
    let (_d, mut model, mut worker) = viewer("a11y-arrange");
    send(&mut model, &mut worker, PdfMsg::Arrange(ArrangeMsg::Start));
    audit(model, "arrange pages");
}

#[test]
fn the_page_is_an_image_with_a_name_and_its_text_is_there_for_screen_readers() {
    let (_d, model, _w) = viewer("a11y-page");
    let mut h = harness(model);
    h.run_steps(3);
    assert!(
        h.query_all_by_label("Page 2").count() >= 1,
        "the page picture is named"
    );
    assert!(h.query_by_label("Text of this page").is_some());
    assert!(h.query_by_label("Find in this PDF").is_some());
    assert!(h.query_by_label("Page number").is_some());
}
