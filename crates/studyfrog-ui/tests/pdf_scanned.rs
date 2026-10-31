#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Pages that are only pictures are said to have no text, in the viewer and in notes.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{NOW_NS, run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_extract_types::ExtractMsg;
use studyfrog_ui::pdf_types::PdfMsg;

#[test]
fn a_scanned_pdf_is_flagged_in_the_viewer_and_its_note_says_why_there_is_no_text() {
    let dir = temp_dir("scan");
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(
        dir.path.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(dir.path.join("bio/scan.pdf"), pdfmake::make_scan()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    studyfrog_service::pdfs::index_pdfs_fully(worker.session.as_mut().unwrap(), NOW_NS).unwrap();
    let fx = apply(&mut model, Msg::Pdf(PdfMsg::Open("bio/scan.pdf".into(), 0)));
    run_to_idle(&mut model, &mut worker, fx);
    assert_eq!(model.state.pdf.as_ref().unwrap().scanned, [0]);

    let fx = apply(
        &mut model,
        Msg::Pdf(PdfMsg::Extract(ExtractMsg::Folder("bio".into()))),
    );
    run_to_idle(&mut model, &mut worker, fx);
    let fx = apply(&mut model, Msg::Pdf(PdfMsg::Extract(ExtractMsg::Make)));
    run_to_idle(&mut model, &mut worker, fx);
    let view = model.state.pdf.as_ref().unwrap();
    let text = fs::read_to_string(dir.path.join(&view.sourced[0].path)).unwrap();
    assert!(text.contains("picture without text"), "{text}");
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(h.query_by_label_contains("only holds pictures").is_some());
}

#[test]
fn pictures_and_snapshots_are_saved_and_can_be_put_into_the_note() {
    let dir = temp_dir("pics");
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(
        dir.path.join("bio/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: Bio\n---\n",
    )
    .unwrap();
    fs::write(dir.path.join("bio/scan.pdf"), pdfmake::make_scan()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    let mut send = |model: &mut Model, m: PdfMsg| {
        let fx = apply(model, Msg::Pdf(m));
        run_to_idle(model, &mut worker, fx);
    };
    send(&mut model, PdfMsg::Open("bio/scan.pdf".into(), 0));
    send(&mut model, PdfMsg::Extract(ExtractMsg::SavePictures));
    assert_eq!(model.state.pdf.as_ref().unwrap().extract.pictures.len(), 1);
    send(&mut model, PdfMsg::Extract(ExtractMsg::ToggleSnapshot));
    let rect = studyfrog_pdf::render::PageRect {
        x: 10.0,
        y: 10.0,
        width: 40.0,
        height: 30.0,
    };
    send(&mut model, PdfMsg::Extract(ExtractMsg::Snapshot(rect)));
    let extract = model.state.pdf.as_ref().unwrap().extract.clone();
    assert_eq!((extract.pictures.len(), extract.snapshot_mode), (2, false));
    assert!(dir.path.join(&extract.pictures[1]).exists());
    send(
        &mut model,
        PdfMsg::Extract(ExtractMsg::IncludePictures(true)),
    );
    send(
        &mut model,
        PdfMsg::Extract(ExtractMsg::Folder("bio".into())),
    );
    send(&mut model, PdfMsg::Extract(ExtractMsg::Make));
    let view = model.state.pdf.as_ref().unwrap();
    let text = fs::read_to_string(dir.path.join(&view.sourced[0].path)).unwrap();
    assert!(
        text.contains("![scan-p1-1-") && text.contains("snapshot-"),
        "{text}"
    );
}
