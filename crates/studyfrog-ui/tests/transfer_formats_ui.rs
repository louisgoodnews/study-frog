#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0209: the import dialog tells which files it reads, with examples that can be saved.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_bulk::LibraryEffect;
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::transfer_ui::{Tab, TransferJob, TransferMsg};

fn t(m: TransferMsg) -> Msg {
    Msg::Library(LibraryMsg::Transfer(m))
}

#[test]
fn the_list_of_formats_opens_and_saving_an_example_asks_for_a_file_write() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    let mut h = harness(m);
    h.run_steps(6);
    h.get_by_label("Which files can I import?").click();
    h.run_steps(6);
    assert!(
        h.query_by_label_contains("Without a header the first two columns")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Obsidian vault (folder)")
            .is_some()
    );
    assert_all_named(&h, "import formats");
    h.get_by_label("Save example (studyfrog-example.csv)")
        .click();
    h.run_steps(4);
    let wrote = h.state().1.iter().any(|e| {
        matches!(
            e,
            Effect::Library(LibraryEffect::Transfer(TransferJob::SaveExample { file, .. }))
                if file == "studyfrog-example.csv"
        )
    });
    assert!(wrote, "{:?}", h.state().1);
}

#[test]
fn the_answer_of_the_write_is_told_in_a_notice() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    apply(&mut m, t(TransferMsg::ExampleSaved(Ok("/h/x.csv".into()))));
    assert!(m.state.notice.as_ref().unwrap().text.contains("/h/x.csv"));
    apply(&mut m, t(TransferMsg::ExampleSaved(Err("no space".into()))));
    assert!(m.state.notice.as_ref().unwrap().text.contains("no space"));
}
