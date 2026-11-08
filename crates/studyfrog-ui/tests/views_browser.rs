#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The browser's real widgets, headless.

mod common;

use common::views::{harness, lib_item, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{ItemRow, Selection};

fn item(n: u128, title: &str) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("french/{n}.md"),
        due: None,
        status: studyfrog_core::status::Status::New,
    }
}

fn stack_open() -> studyfrog_ui::model::Model {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    m.state.app.stack_tabs = vec!["french:list".into()];
    apply(
        &mut m,
        Msg::ItemsLoaded(Ok(vec![item(1, "Bonjour"), item(2, "Au revoir")])),
    );
    let rows = vec![lib_item(1, "Bonjour"), lib_item(2, "Au revoir")];
    apply(
        &mut m,
        Msg::Library(studyfrog_ui::library_ui::LibraryMsg::Loaded(Ok(rows))),
    );
    m
}

#[test]
fn clicking_a_stack_selects_it_and_asks_for_its_items() {
    let mut h = harness(open_model());
    h.run();
    h.get_all_by_label("Maths").next().unwrap().click();
    h.run();
    assert!(
        h.state()
            .1
            .contains(&Effect::LoadItems(Selection::Stack("maths".into())))
    );
    assert_eq!(
        h.state().0.state.browser.selection,
        Selection::Stack("maths".into())
    );
}

#[test]
fn the_title_filter_narrows_the_list_as_you_type() {
    let mut h = harness(stack_open());
    h.run();
    assert!(h.query_by_label("2 of 2 items").is_some() && h.query_by_label("Bonjour").is_some());
    h.get_by_label("Search titles").focus();
    h.get_by_label("Search titles").type_text("revoir");
    h.run();
    assert!(h.query_by_label("1 of 2 items").is_some());
    assert!(h.query_by_label("Bonjour").is_none() && h.query_by_label("Au revoir").is_some());
}

#[test]
fn the_page_holds_no_creation_forms_and_a_card_is_made_in_the_create_dialog() {
    let mut h = harness(stack_open());
    h.run();
    for gone in ["Card front", "Card back", "Add card", "New card"] {
        assert!(
            h.query_by_label(gone).is_none(),
            "{gone} moved to the Create dialog"
        );
    }
    apply(
        &mut h.state_mut().0,
        Msg::Modal(studyfrog_ui::modal::ModalMsg::OpenCreate),
    );
    h.run();
    assert!(
        h.query_by_label("Type of item").is_some() && h.query_by_label("Parent stack").is_some()
    );
}

#[test]
fn the_new_stack_button_opens_the_dialog_for_a_stack() {
    let mut h = harness(stack_open());
    h.run();
    h.get_by_label("New stack…").click();
    h.run();
    assert!(
        matches!(&h.state().0.state.modal, Some(studyfrog_ui::modal::Modal::Create(f)) if f.kind == studyfrog_service::create_more::NewKind::Stack),
        "the button opens the dialog for a stack"
    );
    apply(
        &mut h.state_mut().0,
        Msg::Modal(studyfrog_ui::modal::ModalMsg::Close),
    );
    h.run();
}

#[test]
fn the_inbox_keeps_the_stack_list_available() {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Inbox));
    let mut h = harness(m);
    h.run();
    assert!(
        h.query_by_label("New stack…").is_some(),
        "the stack list stays available"
    );
}

#[test]
fn the_item_list_says_when_each_item_is_next_due() {
    let mut m = stack_open();
    m.state.library.items[0].row.due_day = Some(1_800_000_000_000 / 86_400_000); // 2027-01-15 UTC
    m.state.library.rows[0].due_day = Some(1_800_000_000_000 / 86_400_000);
    for (n, item) in m.state.library.items.iter_mut().enumerate() {
        item.row.key = Some(format!("FLASHCARD_{}", n + 1)); // no empty Key cell to miscount
    }
    let mut h = harness(m);
    h.run_steps(2);
    assert!(
        h.query_by_label("2027-01-15").is_some(),
        "a reviewed item shows its date"
    );
    assert_eq!(
        h.query_all_by_label("–").count(),
        1,
        "the other was never reviewed, so it has no due date"
    );
}
