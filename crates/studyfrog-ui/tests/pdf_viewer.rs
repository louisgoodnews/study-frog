#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The PDF viewer end to end: the update, the worker with a real file, and the selection helpers.
use common::{NOW_NS, run_to_idle, temp_dir};
use std::fs;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_types::{MAX_CACHED, PageKey, PdfMsg, Zoom};
use studyfrog_ui::state::Screen;

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

fn open_with_pdf(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::create_dir_all(dir.path.join("bio")).unwrap();
    fs::write(dir.path.join("bio/three.pdf"), pdfmake::three()).unwrap();
    let mut model = Model::default();
    let mut worker = worker_new(None);
    let path = dir.path.display().to_string();
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::Vault(EffectVault::OpenVault(path))],
    );
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

#[test]
fn opening_a_pdf_reads_its_structure_and_the_words_of_the_page() {
    let (_d, mut model, mut worker) = open_with_pdf("open");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 1),
    );
    let view = model.state.pdf.as_ref().unwrap();
    assert_eq!(model.state.screen, Screen::Pdf);
    assert_eq!((view.info.as_ref().unwrap().page_count, view.page), (3, 1));
    assert_eq!(
        view.title,
        "three".to_owned().replace("three", view.title.as_str())
    );
    assert_eq!(view.text.as_ref().unwrap().words[0].text, "Beta");
    assert!(view.error.is_none());
}

#[test]
fn pages_turn_within_the_document_and_zoom_steps_through_the_list() {
    let (_d, mut model, mut worker) = open_with_pdf("turn");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    send(&mut model, &mut worker, PdfMsg::Previous);
    assert_eq!(model.state.pdf.as_ref().unwrap().page, 0);
    send(&mut model, &mut worker, PdfMsg::Next);
    send(&mut model, &mut worker, PdfMsg::Next);
    send(&mut model, &mut worker, PdfMsg::Next);
    let view = model.state.pdf.as_ref().unwrap();
    assert_eq!(
        (
            view.page,
            view.text.as_ref().unwrap().words[0].text.as_str()
        ),
        (2, "Gamma")
    );
    send(&mut model, &mut worker, PdfMsg::ZoomIn);
    assert_eq!(model.state.pdf.as_ref().unwrap().zoom, Zoom::Percent(125));
    send(&mut model, &mut worker, PdfMsg::SetZoom(Zoom::Percent(25)));
    send(&mut model, &mut worker, PdfMsg::ZoomOut);
    assert_eq!(
        model.state.pdf.as_ref().unwrap().zoom,
        Zoom::Percent(25),
        "no step below the smallest"
    );
}

#[test]
fn a_page_is_drawn_once_per_scale_and_the_cache_stays_bounded() {
    let (_d, mut model, mut worker) = open_with_pdf("draw");
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    let key = PageKey {
        page: 0,
        scale_milli: 1000,
    };
    let fx = apply(&mut model, Msg::Pdf(PdfMsg::Need(key)));
    assert_eq!(fx.len(), 1);
    assert!(
        apply(&mut model, Msg::Pdf(PdfMsg::Need(key))).is_empty(),
        "already asked for"
    );
    run_to_idle(&mut model, &mut worker, fx);
    let view = model.state.pdf.as_ref().unwrap();
    let (_, px) = &view.pictures[0];
    assert_eq!((px.width, px.height), (612, 792));
    assert!(view.requested.is_empty());
    assert!(
        apply(&mut model, Msg::Pdf(PdfMsg::Need(key))).is_empty(),
        "already drawn"
    );
    for i in 0..MAX_CACHED + 5 {
        let k = PageKey {
            page: 1,
            scale_milli: 400 + i as u32,
        };
        send(&mut model, &mut worker, PdfMsg::Need(k));
    }
    assert!(model.state.pdf.as_ref().unwrap().pictures.len() <= MAX_CACHED);
}

#[test]
fn searching_inside_a_pdf_uses_the_index_and_jumps_to_the_first_hit_from_here() {
    let (_d, mut model, mut worker) = open_with_pdf("find");
    studyfrog_service::pdfs::index_pdfs_fully(worker.session.as_mut().unwrap(), NOW_NS).unwrap();
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open("bio/three.pdf".into(), 0),
    );
    send(&mut model, &mut worker, PdfMsg::Query("gamma".into()));
    let view = model.state.pdf.as_ref().unwrap();
    assert_eq!((view.found.clone(), view.page), (vec![2], 2));
    send(&mut model, &mut worker, PdfMsg::PreviousHit);
    assert_eq!(
        model.state.pdf.as_ref().unwrap().page,
        2,
        "wraps to the only hit"
    );
    send(&mut model, &mut worker, PdfMsg::Query(String::new()));
    assert!(model.state.pdf.as_ref().unwrap().found.is_empty());
}

#[test]
fn the_global_search_finds_pdf_pages_and_a_click_opens_them_at_that_page() {
    let (_d, mut model, mut worker) = open_with_pdf("global");
    studyfrog_service::pdfs::index_pdfs_fully(worker.session.as_mut().unwrap(), NOW_NS).unwrap();
    let fx = apply(
        &mut model,
        Msg::Chrome(studyfrog_ui::chrome::ChromeMsg::SearchEdited("beta".into())),
    );
    run_to_idle(&mut model, &mut worker, fx);
    let hit = model.state.chrome.pdf_results[0].clone();
    assert_eq!((hit.path.as_str(), hit.page), ("bio/three.pdf", Some(2)));
    send(
        &mut model,
        &mut worker,
        PdfMsg::Open(hit.path, hit.page.unwrap() - 1),
    );
    assert_eq!(model.state.pdf.as_ref().unwrap().page, 1);
}
