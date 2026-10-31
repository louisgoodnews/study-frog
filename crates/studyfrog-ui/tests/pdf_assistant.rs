#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! PDF pages as extra reading for the assistant.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{NOW_NS, run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::agent_types::{AgentMsg, AgentTab};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::worker_new;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_extract_types::ExtractMsg;
use studyfrog_ui::pdf_types::PdfMsg;
use studyfrog_ui::state::Screen;

#[test]
fn the_viewer_hands_pages_to_the_assistant_which_reads_them_and_says_so() {
    let dir = temp_dir("assist");
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
    let fx = apply(
        &mut model,
        Msg::Pdf(PdfMsg::Open("bio/three.pdf".into(), 1)),
    );
    run_to_idle(&mut model, &mut worker, fx);
    let fx = apply(&mut model, Msg::Pdf(PdfMsg::Extract(ExtractMsg::Last(2))));
    run_to_idle(&mut model, &mut worker, fx);
    let last = model.state.pdf.as_ref().unwrap().extract.last.unwrap();
    apply(
        &mut model,
        Msg::Agent(AgentMsg::UsePdfPages("bio/three.pdf".into(), 1, last)),
    );
    assert_eq!(
        model.state.agent.pdf_pages,
        Some(("bio/three.pdf".to_owned(), 1, 2))
    );
    assert_eq!(model.state.agent.tab, AgentTab::Wiki);
    apply(&mut model, Msg::Navigate(Screen::Agent));
    apply(&mut model, Msg::Agent(AgentMsg::Folder("bio".into(), true)));
    let mut h = common::views::harness(model);
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Also reading bio/three.pdf, pages 2 to 3")
            .is_some()
    );
    let fx = std::mem::take(&mut h.state_mut().1);
    drop(fx);
    let model = &mut h.state_mut().0;
    let ctx = studyfrog_service::agent_pdf::wiki_context_with_pdf(
        worker.session.as_ref().unwrap(),
        &["bio".to_owned()],
        Some(("bio/three.pdf", 1, 2)),
    )
    .unwrap();
    assert_eq!(ctx.items.iter().filter(|i| i.kind == "pdf").count(), 2);
    apply(model, Msg::Agent(AgentMsg::ClearPdfPages));
    assert!(model.state.agent.pdf_pages.is_none());
}
