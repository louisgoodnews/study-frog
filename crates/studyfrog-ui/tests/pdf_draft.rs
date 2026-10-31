#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Cards and questions from highlighted passages.

mod common;
#[path = "../../studyfrog-pdf/tests/common/mod.rs"]
mod pdfmake;

use std::fs;

use common::{run_to_idle, temp_dir};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{Worker, worker_new};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_draft_types::{DraftKind, DraftMsg};
use studyfrog_ui::pdf_extract_types::ExtractMsg;
use studyfrog_ui::pdf_mark_types::MarkMsg;
use studyfrog_ui::pdf_types::PdfMsg;

fn send(model: &mut Model, worker: &mut Worker, m: PdfMsg) {
    let fx = apply(model, Msg::Pdf(m));
    run_to_idle(model, worker, fx);
}

#[test]
fn a_highlight_becomes_a_card_and_a_selection_a_question_each_with_the_page_as_source() {
    let dir = temp_dir("draft");
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
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Start(DraftKind::Card, None)),
    );
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Select some words")
    );
    send(&mut model, &mut worker, PdfMsg::Select(vec![0, 1]));
    send(&mut model, &mut worker, PdfMsg::Marks(MarkMsg::Highlight));
    send(
        &mut model,
        &mut worker,
        PdfMsg::Marks(MarkMsg::EditNote(1, "Which letter is second?".into())),
    );
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Start(DraftKind::Card, Some(1))),
    );
    let d = model.state.pdf.as_ref().unwrap().draft.clone().unwrap();
    assert_eq!(
        (d.front.as_str(), d.back.as_str(), d.page),
        ("Which letter is second?", "Beta second", 1)
    );
    send(&mut model, &mut worker, PdfMsg::Draft(DraftMsg::Create));
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Choose the stack")
    );
    send(
        &mut model,
        &mut worker,
        PdfMsg::Extract(ExtractMsg::Folder("bio".into())),
    );
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Back("Beta".into())),
    );
    send(&mut model, &mut worker, PdfMsg::Draft(DraftMsg::Create));
    let view = model.state.pdf.as_ref().unwrap();
    assert!(view.draft.is_none());
    assert_eq!(
        (view.sourced.len(), view.sourced[0].kind.as_str()),
        (1, "card")
    );
    let text = fs::read_to_string(dir.path.join(&view.sourced[0].path)).unwrap();
    assert!(
        text.contains("page=2") && text.contains("Which letter is second?"),
        "{text}"
    );
    send(&mut model, &mut worker, PdfMsg::Select(vec![0]));
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Start(DraftKind::Question, None)),
    );
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Front("Name it".into())),
    );
    send(&mut model, &mut worker, PdfMsg::Draft(DraftMsg::Create));
    assert_eq!(model.state.pdf.as_ref().unwrap().sourced.len(), 2);
    send(
        &mut model,
        &mut worker,
        PdfMsg::Draft(DraftMsg::Start(DraftKind::Card, None)),
    );
    send(&mut model, &mut worker, PdfMsg::Draft(DraftMsg::Cancel));
    assert!(model.state.pdf.as_ref().unwrap().draft.is_none());
}
