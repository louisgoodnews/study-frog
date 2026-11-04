#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Headless UI acceptance tests for PDF export execution, outcomes, and notifications.

mod common;

use std::path::PathBuf;

use common::flow::opened;
use studyfrog_service::pdf_export::{ExportTarget, PdfExportOutcome};
use studyfrog_ui::modal::{Modal, ModalMsg, ToastAction};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::notify_types::NotifyTarget;
use studyfrog_ui::pdf_export_types::PdfExportMsg;
use studyfrog_ui::pdf_types::PdfMsg;
use studyfrog_ui::state::Screen;

fn export_msg(m: PdfExportMsg) -> Msg {
    Msg::Pdf(PdfMsg::Export(m))
}

#[test]
fn pdf_export_execution_outcome_status_and_notification() {
    let (_v, mut model, _worker) = opened("pdf-export-exec");
    apply(
        &mut model,
        export_msg(PdfExportMsg::Open(ExportTarget::Stack("french".into()))),
    );

    let dest = PathBuf::from("/tmp/test_french_export.pdf");
    apply(
        &mut model,
        export_msg(PdfExportMsg::EditDestination(dest.display().to_string())),
    );

    let outcome = PdfExportOutcome {
        destination: dest.clone(),
        bytes_written: 12345,
        pages: 3,
        item_count: 5,
        warnings: Vec::new(),
    };
    apply(&mut model, export_msg(PdfExportMsg::Exported(Ok(outcome))));

    if let Some(Modal::PdfExport(f)) = &model.state.modal {
        assert!(f.status.as_ref().unwrap().contains("Exported 3 pages"));
        assert!(
            f.status
                .as_ref()
                .unwrap()
                .contains("/tmp/test_french_export.pdf")
        );
    } else {
        panic!("expected modal still open with status");
    }

    let notifs = &model.state.chrome.notifications;
    let n = notifs.last().expect("expected notification");
    assert!(n.text.contains("Exported 3 pages"));
    assert_eq!(n.target, Some(NotifyTarget::Screen(Screen::Pdf)));

    let toast = model.state.toasts.last().expect("expected toast");
    assert_eq!(toast.title, "PDF exported");
    assert_eq!(
        toast.action,
        ToastAction::OpenPdf(dest.display().to_string())
    );

    let (next_state, _) = studyfrog_ui::modal::update(model.state, ModalMsg::ToastClicked(0));
    assert_eq!(next_state.screen, Screen::Pdf);
    assert!(next_state.pdf.is_some());
    assert_eq!(
        next_state.pdf.as_ref().unwrap().path,
        dest.display().to_string()
    );
}
