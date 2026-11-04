#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cognitive_complexity)] // tests may panic on failure
//! Headless UI acceptance tests for TASK-0231: Export to PDF in the application.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness};
use egui_kittest::kittest::Queryable;
use studyfrog_pdf::write::PageSize;
use studyfrog_service::pdf_export::ExportTarget;
use studyfrog_ui::modal::Modal;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::pdf_export_types::{PdfExportMsg, PdfExportPreview, PreviewPage, margins_narrow};
use studyfrog_ui::pdf_types::PdfMsg;
use studyfrog_ui::state::{Screen, Selection};

fn export_msg(m: PdfExportMsg) -> Msg {
    Msg::Pdf(PdfMsg::Export(m))
}

#[test]
fn pdf_export_offered_on_stack_page_and_export_screen() {
    let (_v, mut model, mut worker) = opened("pdf-export-open");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(
        &mut model,
        &mut worker,
        Msg::Select(Selection::Stack("french".into())),
    );

    // 1. Offered on the stack page under "More actions"
    let mut h = harness(model);
    h.run_steps(4);
    h.get_by_label("More actions").click();
    h.run_steps(4);
    assert!(h.query_by_label("Export to PDF…").is_some());
    h.get_by_label("Export to PDF…").click();
    h.run_steps(4);

    let state = &h.state().0.state;
    assert!(matches!(state.modal, Some(Modal::PdfExport(_))));
    if let Some(Modal::PdfExport(form)) = &state.modal {
        assert_eq!(form.target, ExportTarget::Stack("french".into()));
    }
    assert_all_named(&h, "pdf export dialog from stack");

    // 2. Offered in the Export screen (Transfer dialog)
    let (_v2, mut m2, mut w2) = opened("pdf-export-screen");
    let open_xfer = studyfrog_ui::transfer_ui::TransferMsg::Open(
        studyfrog_ui::transfer_ui::Tab::Export,
        Some("french".into()),
    );
    send(
        &mut m2,
        &mut w2,
        Msg::Library(studyfrog_ui::library_ui::LibraryMsg::Transfer(open_xfer)),
    );
    let mut h2 = harness(m2);
    h2.run_steps(4);
    assert!(h2.query_by_label("Export to PDF…").is_some());
    h2.get_by_label("Export to PDF…").click();
    h2.run_steps(4);

    assert!(matches!(
        h2.state().0.state.modal,
        Some(Modal::PdfExport(_))
    ));
}

#[test]
fn pdf_export_offered_on_bulk_bar_and_item_page() {
    let (_v, mut model, mut worker) = opened("pdf-export-bulk");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    send(&mut model, &mut worker, Msg::Select(Selection::Nothing));

    // 1. Offered on bulk bar when items are selected
    let uids = model
        .state
        .library
        .items
        .iter()
        .map(|i| i.uid)
        .collect::<Vec<_>>();
    model.state.library.selected = uids.iter().copied().collect();
    let mut h = harness(model);
    h.run_steps(4);
    assert!(h.query_by_label("Export to PDF…").is_some());
    h.get_by_label("Export to PDF…").click();
    h.run_steps(4);

    if let Some(Modal::PdfExport(form)) = &h.state().0.state.modal {
        assert_eq!(form.target, ExportTarget::Selection(uids));
    } else {
        panic!("expected PdfExport modal");
    }

    // 2. Offered on item page toolbar under "More actions"
    let (_v2, mut m2, mut w2) = opened("pdf-export-item");
    send(&mut m2, &mut w2, Msg::Navigate(Screen::Library));
    let item_uid = m2.state.library.items[0].uid;
    send(&mut m2, &mut w2, Msg::OpenItem(item_uid));
    let mut h2 = harness(m2);
    h2.run_steps(4);
    h2.get_by_label("More actions").click();
    h2.run_steps(4);
    assert!(h2.query_by_label("Export to PDF…").is_some());
    h2.get_by_label("Export to PDF…").click();
    h2.run_steps(4);

    if let Some(Modal::PdfExport(form)) = &h2.state().0.state.modal {
        assert_eq!(form.target, ExportTarget::SingleItem(item_uid));
    } else {
        panic!("expected PdfExport modal on item page");
    }
}

#[test]
fn pdf_export_dialog_options_preview_and_accessible_names() {
    let (_v, mut model, mut worker) = opened("pdf-export-options");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    apply(
        &mut model,
        export_msg(PdfExportMsg::Open(ExportTarget::Stack("french".into()))),
    );

    let mut h = harness(model);
    h.run_steps(4);
    assert_all_named(&h, "pdf export options dialog");

    // Check all options widgets
    assert!(h.query_by_label("Page size: A4").is_some());
    assert!(h.query_by_label("Page size: US Letter").is_some());
    assert!(h.query_by_label("Margins: Normal 54 points").is_some());
    assert!(h.query_by_label("Margins: Narrow 36 points").is_some());
    assert!(h.query_by_label("Margins: Wide 72 points").is_some());
    assert!(h.query_by_label("Font size: 10 points").is_some());
    assert!(h.query_by_label("Include title page").is_some());
    assert!(h.query_by_label("Include table of contents").is_some());
    assert!(h.query_by_label("Include page numbers").is_some());
    assert!(
        h.query_by_label("Include card and question answers")
            .is_some()
    );
    assert!(h.query_by_label("Browse for destination file").is_some());
    assert!(h.query_by_label("Export PDF").is_some());
    assert!(h.query_by_label("Cancel PDF export").is_some());

    // Switch options
    h.get_by_label("Page size: US Letter").click();
    h.run_steps(2);
    if let Some(Modal::PdfExport(f)) = &h.state().0.state.modal {
        assert_eq!(f.options.page_size, PageSize::Letter);
    }

    h.get_by_label("Margins: Narrow 36 points").click();
    h.run_steps(2);
    if let Some(Modal::PdfExport(f)) = &h.state().0.state.modal {
        assert_eq!(f.options.margins, margins_narrow());
    }

    // Deliver preview
    let prev = PdfExportPreview {
        total_pages: 2,
        item_count: 5,
        title: "French Grammar".into(),
        warnings: Vec::new(),
        pages: vec![
            PreviewPage {
                page_number: 1,
                headings: vec!["French Grammar".into()],
                snippet: "Introduction to verbs".into(),
                pixels: None,
            },
            PreviewPage {
                page_number: 2,
                headings: vec!["Irregular Verbs".into()],
                snippet: "Avoir and Etre".into(),
                pixels: None,
            },
        ],
    };
    apply(
        &mut h.state_mut().0,
        export_msg(PdfExportMsg::PreviewReady(Ok(prev))),
    );
    h.run_steps(4);
    assert_all_named(&h, "pdf export dialog with preview");
    assert!(h.query_by_label("2 pages, 5 items").is_some());
}
