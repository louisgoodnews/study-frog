#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Selecting rows, bulk edits through the real worker, undo, and saved filters.

mod common;

use common::flow::{opened, send};
use common::views::{assert_all_named, harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::status::Status;
use studyfrog_service::bulk::BulkEdit;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::library_bulk::{BulkMsg, LibraryEffect};
use studyfrog_ui::library_search::{SearchMode, SearchMsg};
use studyfrog_ui::library_ui::LibraryMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn bulk(m: BulkMsg) -> Msg {
    Msg::Library(LibraryMsg::Bulk(m))
}

fn titles(m: &Model) -> Vec<String> {
    let l = &m.state.library;
    l.shown
        .iter()
        .map(|i| l.items[*i].row.title.clone())
        .collect()
}

#[test]
fn selecting_deleting_with_a_question_and_pruning_after_a_load() {
    let mut m = open_model();
    let rows: Vec<_> = (1..=4).map(|i| lib_item(i, &format!("Item {i}"))).collect();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows.clone()))));
    apply(&mut m, bulk(BulkMsg::Select(rows[0].uid, true)));
    apply(&mut m, bulk(BulkMsg::Select(rows[1].uid, true)));
    assert_eq!(m.state.library.selected.len(), 2);
    apply(&mut m, bulk(BulkMsg::Select(rows[1].uid, false)));
    apply(&mut m, bulk(BulkMsg::SelectAll(true)));
    assert_eq!(m.state.library.selected.len(), 4);
    apply(&mut m, Msg::SearchEdited("Item 3".into()));
    apply(&mut m, bulk(BulkMsg::SelectAll(true)));
    assert_eq!(
        m.state.library.selected.len(),
        1,
        "only the rows the filter lets through"
    );

    assert!(apply(&mut m, bulk(BulkMsg::Apply(BulkEdit::Delete))).is_empty());
    assert!(m.state.library.confirm_delete, "the first click only asks");
    let fx = apply(&mut m, bulk(BulkMsg::Apply(BulkEdit::Delete)));
    assert_eq!(
        fx,
        [Effect::Library(LibraryEffect::Bulk(
            vec![rows[2].uid],
            BulkEdit::Delete
        ))]
    );
    apply(&mut m, bulk(BulkMsg::AskDelete(false)));
    assert!(!m.state.library.confirm_delete);

    apply(
        &mut m,
        Msg::Library(LibraryMsg::Loaded(Ok(rows[..2].to_vec()))),
    );
    assert!(
        m.state.library.selected.is_empty(),
        "a row that is gone is not selected"
    );
}

#[test]
fn the_bar_appears_with_a_selection_and_every_control_is_named() {
    let mut m = open_model();
    let rows: Vec<_> = (1..=3).map(|i| lib_item(i, &format!("Item {i}"))).collect();
    apply(&mut m, Msg::Library(LibraryMsg::Loaded(Ok(rows.clone()))));
    let mut h = harness(m);
    h.run_steps(3);
    assert!(h.query_by_label("Select all rows").is_some());
    assert!(h.query_by_label("Select Item 2").is_some());
    assert!(
        h.query_by_label("Set status").is_none(),
        "no bar without a selection"
    );
    assert!(h.query_by_label("Filters").is_some());
    h.get_by_label("Select Item 2").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.library.selected.len(), 1);
    for name in [
        "1 selected",
        "Set status",
        "Set priority",
        "Set difficulty",
        "Move to stack",
        "Add tag",
        "Delete",
        "Clear selection",
    ] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert_all_named(&h, "bulk bar");
}

#[test]
fn a_bulk_edit_through_the_worker_changes_files_and_can_be_undone() {
    let (_vault, mut model, mut worker) = opened("bulk");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    let all = titles(&model);
    assert!(all.len() >= 3, "{all:?}");
    send(&mut model, &mut worker, bulk(BulkMsg::SelectAll(true)));
    send(
        &mut model,
        &mut worker,
        bulk(BulkMsg::Apply(BulkEdit::Status(Status::Paused))),
    );
    let l = &model.state.library;
    assert!(l.selected.is_empty() && l.undo.is_some());
    assert!(
        l.items.iter().all(|i| i.row.status == Status::Paused),
        "rows reloaded with the new status"
    );
    assert!(
        model
            .state
            .notice
            .as_ref()
            .unwrap()
            .text
            .starts_with("Changed ")
    );

    send(&mut model, &mut worker, bulk(BulkMsg::Undo));
    assert!(model.state.library.undo.is_none());
    assert!(
        model
            .state
            .library
            .items
            .iter()
            .all(|i| i.row.status != Status::Paused)
    );
}

#[test]
fn saved_filters_are_saved_listed_reopened_and_removed() {
    let (vault, mut model, mut worker) = opened("filters");
    send(&mut model, &mut worker, Msg::Navigate(Screen::Library));
    assert!(model.state.library.filters.is_empty());

    send(
        &mut model,
        &mut worker,
        Msg::Library(LibraryMsg::Search(SearchMsg::Mode(true))),
    );
    send(
        &mut model,
        &mut worker,
        Msg::Library(LibraryMsg::Search(SearchMsg::Query("type = card".into()))),
    );
    send(&mut model, &mut worker, bulk(BulkMsg::SaveFilter));
    assert!(
        model.state.notice.as_ref().unwrap().text.contains("name"),
        "a name is needed"
    );
    send(
        &mut model,
        &mut worker,
        bulk(BulkMsg::FilterName("Cards".into())),
    );
    send(&mut model, &mut worker, bulk(BulkMsg::SaveFilter));
    assert_eq!(model.state.library.filters.len(), 1);
    assert!(model.state.library.filter_name.is_empty());
    assert!(
        std::fs::read_to_string(vault.path.join("filters.toml"))
            .unwrap()
            .contains("type = card")
    );

    send(
        &mut model,
        &mut worker,
        Msg::Library(LibraryMsg::Search(SearchMsg::Query("".into()))),
    );
    let everything = titles(&model).len();
    send(
        &mut model,
        &mut worker,
        bulk(BulkMsg::OpenFilter("Cards".into())),
    );
    assert_eq!(model.state.library.search.mode, SearchMode::Query);
    assert_eq!(model.state.library.search.query, "type = card");
    assert!(titles(&model).len() < everything);

    send(
        &mut model,
        &mut worker,
        bulk(BulkMsg::DeleteFilter("Cards".into())),
    );
    assert!(model.state.library.filters.is_empty());
}
