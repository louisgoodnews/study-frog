#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Notes made from PDF text, and the way back from the note to the page.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_service::document::LoadedDoc;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_extract_types::ExtractMsg;
use studyfrog_ui::pdf_types::PdfMsg;

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

fn extract(model: &mut Model, worker: &mut Worker, m: ExtractMsg) {
    send(model, worker, PdfMsg::Extract(m));
}

#[test]
fn pages_become_a_note_in_the_chosen_stack_and_the_pdf_lists_it() {
    let (dir, mut model, mut worker) = viewer("note");
    extract(&mut model, &mut worker, ExtractMsg::Make);
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Choose the stack")
    );
    extract(&mut model, &mut worker, ExtractMsg::Folder("bio".into()));
    extract(&mut model, &mut worker, ExtractMsg::Last(2));
    extract(&mut model, &mut worker, ExtractMsg::Make);
    let view = model.state.pdf.as_ref().unwrap();
    assert!(!view.extract.busy);
    assert_eq!((view.sourced.len(), view.sourced[0].page), (1, Some(1)));
    let path = dir.path.join(&view.sourced[0].path);
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains("Beta") && text.contains("Gamma"), "{text}");
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .starts_with("Made the note")
    );
}

#[test]
fn a_selection_is_required_when_asked_for_and_then_used() {
    let (dir, mut model, mut worker) = viewer("selection");
    extract(&mut model, &mut worker, ExtractMsg::Folder("bio".into()));
    extract(&mut model, &mut worker, ExtractMsg::UseSelection(true));
    extract(&mut model, &mut worker, ExtractMsg::Make);
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Select some words")
    );
    send(&mut model, &mut worker, PdfMsg::Select(vec![0]));
    extract(
        &mut model,
        &mut worker,
        ExtractMsg::Title("Just one word".into()),
    );
    extract(&mut model, &mut worker, ExtractMsg::Make);
    let view = model.state.pdf.as_ref().unwrap();
    let text = fs::read_to_string(dir.path.join(&view.sourced[0].path)).unwrap();
    assert!(
        text.contains("Beta") && !text.contains("second page"),
        "{text}"
    );
}

#[test]
fn an_item_made_from_a_pdf_page_links_back_to_it() {
    use common::views::{harness, info, open_model};
    let mut m = open_model();
    m.state.vault = Some(info());
    let doc = studyfrog_md::parse::parse_doc(
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: N\nsource: \"bio/three.pdf#page=3\"\n---\nbody\n",
    )
    .unwrap();
    apply(
        &mut m,
        Msg::DocumentLoaded(
            Uid(0xA),
            Ok(LoadedDoc {
                path: "s/n.md".into(),
                doc,
                hash: 1,
            }),
        ),
    );
    let mut h = harness(m);
    h.run_steps(2);
    let link = h.get_by_label_contains("Open source");
    link.click();
    h.run_steps(2);
    let model = &h.state().0;
    assert_eq!(model.state.pdf.as_ref().unwrap().page, 2);
    assert_eq!(model.state.pdf.as_ref().unwrap().path, "bio/three.pdf");
}
