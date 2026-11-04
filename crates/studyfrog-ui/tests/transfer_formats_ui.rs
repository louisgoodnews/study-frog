#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! TASK-0209: the import dialog tells which files it reads, with examples that can be saved.

mod common;

use common::views::{assert_all_named, open_model};
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

fn harness(
    model: studyfrog_ui::model::Model,
) -> egui_kittest::Harness<'static, common::views::App> {
    egui_kittest::Harness::builder()
        .with_size(eframe::egui::vec2(1000.0, 1600.0))
        .build_ui_state(
            |ui, (model, effects): &mut common::views::App| {
                effects.extend(studyfrog_ui::model::frame(model, ui))
            },
            (model, Vec::new()),
        )
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

#[test]
fn toml_and_ron_examples_can_be_saved() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    let mut h = harness(m);
    h.run_steps(6);
    h.get_by_label("Which files can I import?").click();
    h.run_steps(6);
    h.get_by_label("Save example (studyfrog-example.toml)")
        .click();
    h.run_steps(4);
    let wrote_toml = h.state().1.iter().any(|e| {
        matches!(
            e,
            Effect::Library(LibraryEffect::Transfer(TransferJob::SaveExample { file, .. }))
                if file == "studyfrog-example.toml"
        )
    });
    assert!(wrote_toml);

    h.get_by_label("Save example (studyfrog-example.ron)")
        .click();
    h.run_steps(4);
    let wrote_ron = h.state().1.iter().any(|e| {
        matches!(
            e,
            Effect::Library(LibraryEffect::Transfer(TransferJob::SaveExample { file, .. }))
                if file == "studyfrog-example.ron"
        )
    });
    assert!(wrote_ron);
}

#[test]
fn the_csv_entry_offers_a_prompt_and_the_way_to_all_of_them() {
    let mut m = open_model();
    apply(&mut m, t(TransferMsg::Open(Tab::Import, None)));
    let mut h = harness(m);
    h.run_steps(6);
    h.get_by_label("Which files can I import?").click();
    h.run_steps(6);
    assert!(
        h.query_by_label("Copy a prompt that makes this file")
            .is_some()
    );
    h.get_by_label("More prompts…").click();
    h.run_steps(6);
    let s = &h.state().0.state;
    assert_eq!(s.screen, studyfrog_ui::state::Screen::Agent);
    assert_eq!(s.agent.tab, studyfrog_ui::agent_types::AgentTab::Prompts);
    assert!(s.modal.is_none());
}
