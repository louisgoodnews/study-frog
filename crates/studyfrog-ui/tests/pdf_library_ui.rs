#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDFs in the library, on the home page, in links and in the create dialog.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use common::{NOW_NS, run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_library_types::LibraryMsg;
use studyfrog_ui::pdf_types::PdfMsg;
use studyfrog_ui::state::{Screen, Selection};

static PICKED: Mutex<Option<PathBuf>> = Mutex::new(None);

fn picker(_: &[&str]) -> Option<PathBuf> {
    PICKED.lock().unwrap().clone()
}

fn vault(label: &str) -> (common::TempDir, Model, Worker) {
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
    worker.file_picker = Some(picker);
    run_to_idle(
        &mut model,
        &mut worker,
        vec![Effect::OpenVault(dir.path.display().to_string())],
    );
    studyfrog_service::pdfs::index_pdfs_fully(worker.session.as_mut().unwrap(), NOW_NS).unwrap();
    (dir, model, worker)
}

fn lib(model: &mut Model, worker: &mut Worker, m: LibraryMsg) {
    let fx = apply(model, Msg::Pdf(PdfMsg::Library(m)));
    run_to_idle(model, worker, fx);
}

#[test]
fn the_library_lists_pdfs_adds_one_by_the_file_dialog_and_renames_with_rewriting() {
    let (dir, mut model, mut worker) = vault("lib");
    lib(&mut model, &mut worker, LibraryMsg::Load);
    assert_eq!(model.state.pdf_library.rows.len(), 1);
    lib(&mut model, &mut worker, LibraryMsg::AddTo(String::new()));
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Open a stack first")
    );
    let outside = dir
        .path
        .join("..")
        .join(format!("studyfrog-pick-{}.pdf", std::process::id()));
    fs::write(&outside, pdfmake::three()).unwrap();
    *PICKED.lock().unwrap() = Some(outside.clone());
    lib(&mut model, &mut worker, LibraryMsg::AddTo("bio".into()));
    assert_eq!(model.state.pdf_library.rows.len(), 2);
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .starts_with("Added bio/studyfrog-pick-")
    );
    *PICKED.lock().unwrap() = None;
    lib(&mut model, &mut worker, LibraryMsg::AddTo("bio".into()));
    assert_eq!(
        model.state.pdf_library.rows.len(),
        2,
        "cancelling adds nothing"
    );
    lib(
        &mut model,
        &mut worker,
        LibraryMsg::StartRename("bio/three.pdf".into()),
    );
    lib(
        &mut model,
        &mut worker,
        LibraryMsg::RenameEdit("lecture.pdf".into()),
    );
    lib(&mut model, &mut worker, LibraryMsg::DoRename);
    assert!(dir.path.join("bio/lecture.pdf").is_file() && !dir.path.join("bio/three.pdf").exists());
    assert!(model.state.pdf_library.rename.is_none());
    assert!(
        model
            .state
            .pdf_library
            .rows
            .iter()
            .any(|r| r.path == "bio/lecture.pdf")
    );
    let _ = fs::remove_file(outside);
}

#[test]
fn the_stack_page_shows_its_pdfs_with_page_counts_and_the_home_page_lists_them() {
    let (_d, mut model, mut worker) = vault("views");
    lib(&mut model, &mut worker, LibraryMsg::Load);
    apply(&mut model, Msg::Navigate(Screen::Library));
    apply(&mut model, Msg::Select(Selection::Stack("bio".into())));
    model.state.app.stack_tabs = vec!["bio:list".into()];
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(h.query_by_label("Add PDF ...").is_some());
    assert!(h.query_by_label_contains("(3 pages)").is_some());
    let link = h.get_by_label_contains("(3 pages)");
    link.click();
    h.run_steps(2);
    assert_eq!(
        h.state().0.state.pdf.as_ref().unwrap().path,
        "bio/three.pdf"
    );
}

#[test]
fn the_create_dialog_offers_adding_a_pdf() {
    let (_d, mut model, _w) = vault("dialog");
    apply(
        &mut model,
        Msg::Modal(studyfrog_ui::modal::ModalMsg::OpenCreate),
    );
    let mut h = common::views::harness(model);
    h.run_steps(8);
    h.get_by_label("More").click();
    h.run_steps(8);
    assert!(h.query_by_label("Add PDF ...").is_some());
}

#[test]
fn markdown_links_to_pdf_pages_are_recognised_and_web_links_are_not() {
    use studyfrog_ui::pdf_link::pdf_target;
    assert_eq!(
        pdf_target("bio/a.pdf#page=3"),
        Some(("bio/a.pdf".to_owned(), 2))
    );
    assert_eq!(pdf_target("bio/a.PDF"), Some(("bio/a.PDF".to_owned(), 0)));
    assert_eq!(pdf_target("https://x.org/a.pdf#page=3"), None);
    assert_eq!(pdf_target("notes.md"), None);
}

#[test]
fn the_home_page_lists_recent_pdfs() {
    let (_d, mut model, mut worker) = vault("home");
    let fx = apply(&mut model, Msg::Navigate(Screen::Home));
    run_to_idle(&mut model, &mut worker, fx);
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(h.query_by_label_contains("(3 pages)").is_some());
}

#[test]
fn a_pdf_link_in_a_note_opens_the_viewer_at_that_page() {
    use common::views::{harness, info, open_model};
    use studyfrog_core::uid::Uid;
    use studyfrog_service::document::LoadedDoc;
    let mut m = open_model();
    m.state.vault = Some(info());
    let doc = studyfrog_md::parse::parse_doc(
        "---\nid: 0000000000000000000000000A\nkind: note\ntitle: N\n---\nSee [the proof](bio/three.pdf#page=2).\n",
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
    h.get_by_label_contains("the proof").click();
    h.run_steps(2);
    let view = h.state().0.state.pdf.as_ref().unwrap().clone();
    assert_eq!((view.path.as_str(), view.page), ("bio/three.pdf", 1));
}
