#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript reference reached from the help screen (EPIC-058): the help tree offers it, and
//! choosing it opens the same reference in ScriptStudio with its categories.
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[test]
fn the_help_screen_offers_the_reference_and_opens_its_categories() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Help)));
    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("FrogScript reference").click();
    h.run_steps(4);
    let state = &h.state().0.state;
    assert_eq!(
        state.screen,
        Screen::Console,
        "the reference is in ScriptStudio"
    );
    assert!(
        state.console.studio.wiki.open,
        "the reference panel is shown"
    );
    assert!(
        state.console.studio.expanded,
        "ScriptStudio is expanded to hold it"
    );
    for category in ["Syntax", "Built-in Functions", "Lifecycle Events"] {
        assert!(
            h.query_by_label(category).is_some(),
            "the category {category} is listed"
        );
    }
}

#[test]
fn without_a_vault_the_help_screen_says_why_the_reference_is_not_offered() {
    let mut m = Model::default();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Help)));
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("FrogScript reference").is_none());
    assert!(
        h.query_by_label_contains("Open a vault to use the FrogScript reference")
            .is_some()
    );
}
