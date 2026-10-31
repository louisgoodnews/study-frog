#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Following the links of a PDF page: pages at once, web addresses after a yes.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::views::harness;
use common::{run_to_idle, temp_dir};
use egui_kittest::kittest::Queryable;
use studyfrog_pdf::links::LinkTarget;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_link_types::LinkMsg;
use studyfrog_ui::pdf_link_ui::openable;
use studyfrog_ui::pdf_types::PdfMsg;

fn viewer(label: &str) -> (common::TempDir, Model, Worker) {
    let dir = temp_dir(label);
    fs::write(dir.path.join("linked.pdf"), pdfmake::linked()).unwrap();
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
        PdfMsg::Open("linked.pdf".into(), 0),
    );
    (dir, model, worker)
}

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

fn click(model: &mut Model, worker: &mut Worker, t: LinkTarget) {
    send(model, worker, PdfMsg::Link(LinkMsg::Click(t)));
}

#[test]
fn a_page_link_is_followed_at_once_and_a_web_link_waits_for_a_yes() {
    let (_d, mut model, mut worker) = viewer("links");
    let links = model
        .state
        .pdf
        .as_ref()
        .unwrap()
        .text
        .as_ref()
        .unwrap()
        .links
        .clone();
    assert_eq!(links.len(), 3);
    click(&mut model, &mut worker, LinkTarget::Page(2));
    assert_eq!(model.state.pdf.as_ref().unwrap().page, 2);
    click(
        &mut model,
        &mut worker,
        LinkTarget::Uri("https://example.org/x".into()),
    );
    assert_eq!(
        model.state.pdf.as_ref().unwrap().pending_link.as_deref(),
        Some("https://example.org/x")
    );
    send(&mut model, &mut worker, PdfMsg::Link(LinkMsg::Dismiss));
    assert!(model.state.pdf.as_ref().unwrap().pending_link.is_none());
}

#[test]
fn only_web_and_mail_addresses_are_ever_offered_and_the_question_names_the_address() {
    for ok in ["http://a.org", "HTTPS://a.org/x?y=1", "mailto:me@a.org"] {
        assert!(openable(ok), "{ok}");
    }
    for bad in [
        "file:///etc/passwd",
        "javascript:alert(1)",
        "ftp://a.org",
        "data:text/html,x",
        "https://a.org/\n",
        "",
    ] {
        assert!(!openable(bad), "{bad:?}");
    }
    let (_d, mut model, mut worker) = viewer("refuse");
    click(
        &mut model,
        &mut worker,
        LinkTarget::Uri("file:///etc/passwd".into()),
    );
    let view = model.state.pdf.as_ref().unwrap();
    assert!(view.pending_link.is_none());
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("not opened")
    );
    click(
        &mut model,
        &mut worker,
        LinkTarget::Uri("https://example.org/x".into()),
    );
    let mut h = harness(model);
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("https://example.org/x. Open it in your browser?")
            .is_some()
    );
    assert!(h.query_by_label("Open").is_some() && h.query_by_label("Do not open").is_some());
}
