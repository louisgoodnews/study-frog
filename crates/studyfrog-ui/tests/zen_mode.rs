#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! STORY-0116: Zen mode hides the sidebar and the status bar (shortcut, button, automatically in runs), rehearsal
//! targets are at least 36 pt tall, and showing the answer does not move the card's header.

mod common;

use common::views::{harness, open_model, running_model};
use eframe::egui;
use egui_kittest::kittest::Queryable;
use studyfrog_core::run::RunMode;
use studyfrog_ui::appearance_ui::AppearanceMsg;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

const SIDEBAR_ENTRY: &str = "Achievements";
const ZEN: &str = "Zen mode (Ctrl+Shift+F)";

#[test]
fn the_shortcut_and_the_button_hide_and_bring_back_the_chrome() {
    let mut m = open_model();
    m.state.toasts.clear();
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label(SIDEBAR_ENTRY).is_some(), "sidebar shown");
    assert!(
        h.query_by_label_contains("stacks ·").is_some(),
        "status bar shown"
    );
    h.key_press_modifiers(
        egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
        egui::Key::F,
    );
    h.run();
    assert!(studyfrog_ui::zen::active(&h.state().0.state));
    assert!(h.query_by_label(SIDEBAR_ENTRY).is_none(), "sidebar hidden");
    assert!(
        h.query_by_label_contains("stacks ·").is_none(),
        "status bar hidden"
    );
    h.get_by_label(ZEN).click();
    h.run();
    assert!(!studyfrog_ui::zen::active(&h.state().0.state));
    assert!(h.query_by_label(SIDEBAR_ENTRY).is_some(), "sidebar back");
    h.key_press(egui::Key::F11);
    h.run();
    assert!(studyfrog_ui::zen::active(&h.state().0.state), "F11 too");
}

#[test]
fn zen_comes_on_by_itself_in_a_run_when_chosen_and_can_be_left() {
    let mut m = running_model(RunMode::Flip);
    assert!(!studyfrog_ui::zen::active(&m.state), "off unless chosen");
    apply(&mut m, Msg::Appearance(AppearanceMsg::ZenInRuns(true)));
    assert!(studyfrog_ui::zen::active(&m.state), "on during the run");
    apply(&mut m, Msg::Chrome(ChromeMsg::ToggleZen));
    assert!(!studyfrog_ui::zen::active(&m.state), "left by hand");
    apply(&mut m, Msg::Navigate(studyfrog_ui::state::Screen::Home));
    apply(&mut m, Msg::Chrome(ChromeMsg::ToggleZen));
    assert!(studyfrog_ui::zen::active(&m.state), "on by hand anywhere");
}

#[test]
fn rehearsal_targets_are_tall_and_showing_the_answer_does_not_move_the_header() {
    let mut h = harness(running_model(RunMode::Flip));
    h.set_size(egui::vec2(1400.0, 900.0));
    h.run();
    let prompt = h.get_by_label("Capital of France?").rect();
    let count = h.get_by_label("1 / 2").rect();
    let show = h.get_by_label("Show answer (Space)").rect();
    assert!(show.height() >= 36.0, "Show answer {}", show.height());
    h.get_by_label("Show answer (Space)").click();
    h.run();
    assert_eq!(
        h.get_by_label("Capital of France?").rect().top(),
        prompt.top(),
        "prompt moved"
    );
    assert_eq!(h.get_by_label("1 / 2").rect(), count, "header moved");
    for label in ["Again (1)", "Hard (2)", "Good (3)", "Easy (4)"] {
        let r = h.get_by_label(label).rect();
        assert!(r.height() >= 36.0, "{label} {}", r.height());
    }
    let mut t = harness(running_model(RunMode::Answer));
    t.run();
    let submit = t.get_by_label("Submit answer").rect();
    assert!(submit.height() >= 36.0, "Submit {}", submit.height());
}
