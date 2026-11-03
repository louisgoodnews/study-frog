#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The navigation history: places are noted as the user moves, Back, Forward and a jump return to them.

mod common;

use common::flow::{opened, send};
use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::uid::Uid;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::history::{HistoryMsg, MAX, Place, can_back, can_forward, label};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};

fn go(m: &mut Model, h: HistoryMsg) -> Vec<studyfrog_ui::effect::Effect> {
    apply(m, Msg::Chrome(ChromeMsg::History(h)))
}

fn places(m: &Model) -> Vec<Place> {
    m.state.history.entries.clone()
}

#[test]
fn places_are_noted_in_order_and_repeats_are_not() {
    let mut m = open_model(); // opens the vault (Home) and goes to the Library
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(&mut m, Msg::Navigate(Screen::Library));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    assert_eq!(
        places(&m),
        [
            Place::Screen(Screen::Home),
            Place::Screen(Screen::Library),
            Place::Screen(Screen::Calendar),
            Place::Screen(Screen::Library),
            Place::Stack("french".into()),
        ]
    );
    assert_eq!(m.state.history.at, 4);
    assert!(can_back(&m.state.history) && !can_forward(&m.state.history));
}

#[test]
fn back_and_forward_walk_the_places_without_adding_new_ones() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(&mut m, Msg::Navigate(Screen::Library));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let before = places(&m); // Home, Library, Calendar, Library, French
    go(&mut m, HistoryMsg::Back);
    assert_eq!(m.state.screen, Screen::Library);
    assert_eq!(m.state.history.at, 3);
    go(&mut m, HistoryMsg::Back);
    assert_eq!(m.state.screen, Screen::Calendar);
    assert!(can_forward(&m.state.history));
    go(&mut m, HistoryMsg::Forward);
    go(&mut m, HistoryMsg::Forward);
    assert_eq!(m.state.screen, Screen::Library);
    assert_eq!(
        m.state.browser.selection,
        Selection::Stack("french".into()),
        "the stack is selected again"
    );
    assert_eq!(places(&m), before, "travelling adds nothing");
    assert_eq!(m.state.history.at, 4);
}

#[test]
fn going_somewhere_new_after_back_drops_the_places_ahead() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(&mut m, Msg::Navigate(Screen::Board));
    go(&mut m, HistoryMsg::Back);
    go(&mut m, HistoryMsg::Back);
    apply(&mut m, Msg::Navigate(Screen::Reports));
    assert_eq!(
        places(&m),
        [
            Place::Screen(Screen::Home),
            Place::Screen(Screen::Library),
            Place::Screen(Screen::Reports)
        ]
    );
    assert!(!can_forward(&m.state.history));
}

#[test]
fn a_jump_goes_straight_to_a_place_and_only_fifty_are_kept() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    apply(&mut m, Msg::Navigate(Screen::Board));
    go(&mut m, HistoryMsg::Go(0));
    assert_eq!((m.state.screen, m.state.history.at), (Screen::Home, 0));
    go(&mut m, HistoryMsg::Go(99));
    assert_eq!(m.state.history.at, 0, "no such place: nothing happens");
    for i in 0..MAX + 10 {
        let screen = if i % 2 == 0 {
            Screen::Board
        } else {
            Screen::Calendar
        };
        apply(&mut m, Msg::Navigate(screen));
    }
    assert_eq!(m.state.history.entries.len(), MAX);
    assert_eq!(m.state.history.at, MAX - 1);
}

#[test]
fn a_new_vault_starts_a_new_history() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    assert!(m.state.history.entries.len() >= 3);
    let mut other = common::views::info();
    other.root = "/v/other".into();
    apply(&mut m, Msg::VaultOpened(Ok(other)));
    assert_eq!(places(&m), [Place::Screen(Screen::Home)]);
}

#[test]
fn items_are_remembered_with_their_titles_and_reopened_and_unsaved_changes_are_saved_before_leaving()
 {
    let (_v, mut m, mut w) = opened("history-items");
    send(&mut m, &mut w, Msg::OpenItem(Uid(12)));
    let item = Place::Item(Uid(12));
    assert_eq!(places(&m).last(), Some(&item));
    assert_eq!(label(&m.state, &item), "Item: C");
    send(&mut m, &mut w, Msg::Navigate(Screen::Calendar));
    let fx = go(&mut m, HistoryMsg::Back);
    common::run_to_idle(&mut m, &mut w, fx);
    assert_eq!(m.state.screen, Screen::Editor, "the item is open again");
    let n = places(&m).len();
    apply(
        &mut m,
        Msg::Edit(studyfrog_ui::msg::EditAction::Title("changed".into())),
    );
    let fx = go(&mut m, HistoryMsg::Go(0));
    assert!(
        fx.iter()
            .any(|e| matches!(e, studyfrog_ui::effect::Effect::SaveDocument { .. })),
        "the changes are saved first"
    );
    assert_eq!(m.state.screen, Screen::Editor, "and the jump waits for it");
    assert_eq!(places(&m).len(), n);
    common::run_to_idle(&mut m, &mut w, fx);
    assert_ne!(m.state.screen, Screen::Editor, "then it goes");
}

#[test]
fn the_bar_has_back_forward_and_a_history_list_that_jumps() {
    let mut m = open_model();
    m.state.toasts.clear();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Forward (Alt+Right)").is_some());
    h.get_by_label("History").click();
    h.run();
    // The sidebar and the list both name the screens.
    assert!(h.query_all_by_label("Calendar").count() >= 2);
    assert!(h.query_all_by_label("Home").count() >= 2);
    h.get_by_label("Back (Alt+Left)").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Library);
    h.get_by_label("History").click();
    h.run();
    h.get_all_by_role_and_label(eframe::egui::accesskit::Role::Button, "Home")
        .last()
        .unwrap()
        .click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Home);
}

#[test]
fn alt_left_and_alt_right_go_back_and_forward() {
    use eframe::egui::{Key, Modifiers};
    let mut m = open_model();
    m.state.toasts.clear();
    apply(&mut m, Msg::Navigate(Screen::Calendar));
    let mut h = harness(m);
    h.run();
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowLeft);
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Library);
    h.key_press_modifiers(Modifiers::ALT, Key::ArrowRight);
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Calendar);
}
