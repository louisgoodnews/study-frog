#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The top bar's Import button opens the import dialog on the Import tab.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::modal::Modal;
use studyfrog_ui::transfer_ui::Tab;

#[test]
fn the_import_button_in_the_top_bar_opens_the_import_dialog() {
    let mut h = harness(open_model());
    h.run();
    h.get_by_label("Import").click();
    h.run_steps(8);
    assert!(matches!(h.state().0.state.modal, Some(Modal::Transfer)));
    let s = &h.state().0.state.transfer;
    assert_eq!((s.tab, s.scope.as_deref()), (Tab::Import, None));
    assert!(
        h.query_by_label("Import into").is_some(),
        "the import dialog is on screen"
    );
}
