#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The FrogScript reference beside ScriptStudio (STORY-0171): its state, and the console screen with
//! the panel open.

mod common;

use common::views::{harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_script::wiki::{Category, label, topics};
use studyfrog_ui::console_ui::ConsoleMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
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
fn the_console_screen_shows_the_reference_when_it_is_open() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Console));
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
