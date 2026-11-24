#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript reference beside ScriptStudio (STORY-0171): its state, and the console screen with
//! the panel open.
use studyfrog_ui::msg::MsgVault;

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_script::wiki::{Category, label, topics};
use studyfrog_ui::console_ui::ConsoleMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::script_events::EVENT_KINDS;
use studyfrog_ui::script_studio::wiki_panel::WikiMsg;
use studyfrog_ui::state::Screen;

fn wiki(m: &mut studyfrog_ui::model::Model, w: WikiMsg) {
    apply(m, Msg::Console(ConsoleMsg::Wiki(w)));
}

#[test]
fn the_panel_opens_searches_and_remembers_the_topic_by_category() {
    let mut m = open_model();
    assert!(!m.state.console.studio.wiki.open);
    wiki(&mut m, WikiMsg::Toggle);
    assert!(m.state.console.studio.wiki.open);
    wiki(&mut m, WikiMsg::Query("len".into()));
    assert_eq!(m.state.console.studio.wiki.query, "len");
    // `tag` is both a keyword and a function: the key says which one was picked.
    wiki(&mut m, WikiMsg::Pick("Built-in Functions:tag".into()));
    assert_eq!(
        m.state.console.studio.wiki.topic.as_deref(),
        Some("Built-in Functions:tag")
    );
    wiki(&mut m, WikiMsg::Toggle);
    assert!(!m.state.console.studio.wiki.open);
}

#[test]
fn each_topic_has_its_own_key_in_its_category() {
    let tag_keys: Vec<&str> = topics()
        .iter()
        .filter(|t| t.name == "tag")
        .map(|t| label(t.category))
        .collect();
    assert_eq!(tag_keys.len(), 2, "tag is a keyword and a function");
    assert!(tag_keys.contains(&label(Category::Syntax)));
    assert!(tag_keys.contains(&label(Category::BuiltinFunctions)));
}

#[test]
fn every_lifecycle_event_kind_has_an_entry_in_its_category() {
    for kind in EVENT_KINDS {
        assert!(
            topics()
                .iter()
                .any(|t| t.name == *kind && t.category == Category::LifecycleEvents),
            "no lifecycle entry for the event {kind}"
        );
    }
    let listed = topics()
        .iter()
        .filter(|t| t.category == Category::LifecycleEvents)
        .count();
    assert_eq!(listed, EVENT_KINDS.len(), "no entry for an unknown event");
}

#[test]
fn the_console_screen_shows_the_reference_when_it_is_open() {
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Console)));
    apply(&mut m, Msg::Console(ConsoleMsg::StudioToggleExpanded));
    wiki(&mut m, WikiMsg::Toggle);
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.query_by_label("Hide reference").is_some(),
        "the toggle says what it does"
    );
    assert!(h.query_by_label("FrogScript reference").is_some());
    assert!(
        h.query_by_label("Built-in Functions").is_some(),
        "the categories are listed"
    );
}

#[test]
fn the_cookbook_offers_every_recipe_and_runs_the_read_only_ones() {
    use studyfrog_script::cookbook::RECIPES;
    let mut m = open_model();
    apply(&mut m, Msg::Vault(MsgVault::Navigate(Screen::Console)));
    apply(&mut m, Msg::Console(ConsoleMsg::StudioToggleExpanded));
    wiki(&mut m, WikiMsg::Toggle);
    let mut h = harness(m);
    h.run_steps(4);
    let inserts = h.query_all_by_label("Insert into editor").count();
    assert_eq!(inserts, RECIPES.len(), "every recipe can be inserted");
    let runs = h.query_all_by_label("Run in console").count();
    assert_eq!(runs, RECIPES.iter().filter(|r| r.read_only).count());
}

#[test]
fn running_a_recipe_opens_a_new_tab_and_keeps_the_users_tab() {
    use studyfrog_script::cookbook::recipe;
    use studyfrog_ui::script_studio::wiki_panel::run_in_console;
    let mut m = open_model();
    let before = m.state.console.studio.tabs.len();
    let user_code = m.state.console.studio.tabs[m.state.console.studio.active_tab]
        .content
        .clone();
    let due = recipe("Due today").unwrap();
    for msg in run_in_console(due.code) {
        apply(&mut m, Msg::Console(msg));
    }
    let studio = &m.state.console.studio;
    assert_eq!(studio.tabs.len(), before + 1, "a new tab");
    assert_eq!(studio.tabs[studio.active_tab].content, due.code);
    assert_eq!(
        studio.tabs[before - 1].content,
        user_code,
        "the user's tab is untouched"
    );
    assert!(
        m.state
            .console
            .entries
            .iter()
            .any(|e| e.text.contains("due today")),
        "the recipe's output reaches the console"
    );
}

#[test]
fn appending_code_puts_it_on_its_own_line() {
    use studyfrog_ui::script_studio::wiki_panel::append_code;
    assert_eq!(append_code("a", "b"), "a\nb\n");
    assert_eq!(append_code("", "x"), "x\n");
    assert_eq!(append_code("a\n", "b\n"), "a\nb\n");
}
