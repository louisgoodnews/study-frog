#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0096 through the real worker: assigning keys after a confirmation, finding items by key in
//! the library and in the top bar, the key column and the key badge, and undoing the assignment.

mod common;

use common::flow::{opened, send};
use common::views::harness;
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::item_look::{keyless_count, stack_by_key};
use studyfrog_ui::library_bulk::{BulkMsg, LibraryEffect};
use studyfrog_ui::library_ui::{Column, DEFAULT_COLUMNS, LibraryMsg, cell};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn bulk(m: BulkMsg) -> Msg {
    Msg::Library(LibraryMsg::Bulk(m))
}

fn shown(m: &Model) -> Vec<String> {
    let lib = &m.state.library;
    lib.shown
        .iter()
        .map(|i| lib.items[*i].row.title.clone())
        .collect()
}

fn key_of(m: &Model, title: &str) -> Option<String> {
    let item = m
        .state
        .library
        .items
        .iter()
        .find(|i| i.row.title == title)?;
    item.row.key.clone()
}

#[test]
fn keys_are_assigned_only_after_confirming() {
    let (vault, mut m, mut w) = opened("keys-flow");
    send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    assert_eq!(
        keyless_count(&m.state),
        5,
        "one stack and four items, none with a key"
    );
    assert!(DEFAULT_COLUMNS.contains(&Column::Key));
    let before = std::fs::read_to_string(vault.path.join("loose.md")).unwrap();
    assert!(
        apply(&mut m, bulk(BulkMsg::AskAssignKeys(true))).is_empty(),
        "asking writes nothing"
    );
    assert!(m.state.library.confirm_keys);
    let fx = apply(&mut m, bulk(BulkMsg::AskAssignKeys(false)));
    assert!(
        fx.is_empty() && !m.state.library.confirm_keys,
        "cancel writes nothing"
    );
    assert_eq!(
        std::fs::read_to_string(vault.path.join("loose.md")).unwrap(),
        before
    );
    apply(&mut m, bulk(BulkMsg::AskAssignKeys(true)));
    let fx = apply(&mut m, bulk(BulkMsg::AssignKeys));
    assert_eq!(
        fx,
        [studyfrog_ui::effect::Effect::Library(
            LibraryEffect::AssignKeys
        )]
    );
    let keys = ["FLASHCARD_1", "FLASHCARD_2", "NOTE_1", "NOTE_2"];
    let (m, _) = assigned(m, w);
    for (title, key) in ["A", "B", "C", "Loose"].into_iter().zip(keys) {
        assert_eq!(key_of(&m, title).as_deref(), Some(key));
    }
    let b = m
        .state
        .library
        .items
        .iter()
        .find(|i| i.row.title == "B")
        .unwrap();
    assert_eq!(cell(&b.row, Column::Key), "FLASHCARD_2");
}

/// The model after assigning keys through the worker, back on the library.
fn assigned(
    mut m: Model,
    mut w: studyfrog_ui::exec::Worker,
) -> (Model, studyfrog_ui::exec::Worker) {
    send(&mut m, &mut w, bulk(BulkMsg::AssignKeys));
    send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    assert_eq!(keyless_count(&m.state), 0);
    (m, w)
}

#[test]
fn items_are_found_by_key_and_assigning_can_be_undone() {
    let (vault, m, w) = opened("keys-find");
    let before = std::fs::read_to_string(vault.path.join("loose.md")).unwrap();
    let (mut m, mut w) = assigned(m, w);
    send(&mut m, &mut w, Msg::SearchEdited("flashcard_2".into()));
    assert_eq!(
        shown(&m),
        ["B"],
        "an exact key in the library search finds its item"
    );
    send(&mut m, &mut w, Msg::SearchEdited(String::new()));
    send(
        &mut m,
        &mut w,
        Msg::Chrome(ChromeMsg::SearchEdited("NOTE_2".into())),
    );
    assert_eq!(m.state.chrome.results.first().map(|r| r.uid), Some(Uid(13)));
    assert_eq!(
        stack_by_key(&m.state, "stack_1").map(|s| s.uid),
        Some(Uid(1))
    );
    send(&mut m, &mut w, bulk(BulkMsg::Undo));
    assert_eq!(
        std::fs::read_to_string(vault.path.join("loose.md")).unwrap(),
        before
    );
}

#[test]
fn the_library_offers_assign_keys_and_asks_first() {
    let (_vault, mut m, mut w) = opened("keys-view");
    send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    send(
        &mut m,
        &mut w,
        Msg::Select(studyfrog_ui::state::Selection::Inbox),
    );
    let mut h = harness(m);
    h.run_steps(2);
    h.get_by_label("Assign keys…").click();
    h.run_steps(2);
    let fx = h.state().1.clone();
    assert!(
        h.state().0.state.library.confirm_keys,
        "the button only asks"
    );
    assert!(!fx.contains(&studyfrog_ui::effect::Effect::Library(
        LibraryEffect::AssignKeys
    )));
    assert!(
        h.query_all_by_label("Assign keys").count() >= 1,
        "the confirmation"
    );
    assert!(h.query_all_by_label("Cancel").count() >= 1);
}

#[test]
fn the_item_page_shows_the_key_with_a_copy_button() {
    let (_vault, mut m, mut w) = opened("keys-page");
    send(&mut m, &mut w, Msg::Navigate(Screen::Library));
    send(&mut m, &mut w, bulk(BulkMsg::AssignKeys));
    send(&mut m, &mut w, Msg::OpenItem(Uid(11)));
    assert_eq!(m.state.screen, Screen::Editor);
    let mut h = harness(m);
    h.run_steps(2);
    assert!(
        h.query_all_by_label("FLASHCARD_2").count() >= 1,
        "the key badge"
    );
    assert!(h.query_all_by_label("Copy key").count() >= 1);
}
