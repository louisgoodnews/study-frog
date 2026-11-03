#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The help screen drawn: the buttons that open it, the page tree, links, search and the tutorial card.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_service::help::PageRef;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::help_types::{HelpEffect, HelpMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;
use studyfrog_ui::tutorial::TutorialMsg;

fn without_toasts() -> Model {
    let mut m = open_model();
    m.state.toasts.clear();
    m
}

#[test]
fn the_question_mark_opens_the_page_of_the_screen_and_the_tree_opens_others() {
    let mut h = harness(without_toasts());
    h.run();
    h.get_by_label("Help for this screen (F1)").click();
    h.run();
    let state = &h.state().0.state;
    assert_eq!(state.screen, Screen::Help);
    assert_eq!(state.help.page, Some(PageRef::Supplied("library".into())));
    assert!(h.state().1.contains(&Effect::Help(HelpEffect::Load)));
    assert!(h.query_by_label("Back to Library").is_some());
    let keyboard = Msg::Help(HelpMsg::Open(PageRef::Supplied("keyboard".into())));
    apply(&mut h.state_mut().0, keyboard);
    h.run();
    assert!(h.query_by_label_contains("show the answer").is_some());
    assert!(h.query_by_label("Previous page").is_some());
    assert!(h.query_by_label("Copy to my pages").is_some());
    assert_all_named(&h, "help");
    h.get_by_label("Back to Library").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Library);
}

#[test]
fn the_status_bar_help_button_and_search_work_without_a_vault() {
    let mut h = harness(Model::default());
    h.run();
    h.get_by_label("Help").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Help);
    assert_eq!(
        h.state().0.state.help.page,
        Some(PageRef::Supplied("getting-started".into()))
    );
    assert!(
        h.query_by_label_contains("Open a vault to keep your own pages")
            .is_some()
    );
    h.get_by_label("Search help").focus();
    h.get_by_label("Search help").type_text("query");
    h.run();
    assert!(!h.state().0.state.help.hits.is_empty());
    assert!(h.query_by_label_contains("The query language").is_some());
}

#[test]
fn a_user_page_shows_edit_and_delete_and_the_editor_has_named_fields() {
    let mut m = without_toasts();
    apply(&mut m, Msg::Help(HelpMsg::ForScreen));
    let page = studyfrog_service::help::UserPage {
        file: "Mine".into(),
        text: "# Mine\nSee [[faq]].".into(),
    };
    apply(&mut m, Msg::Help(HelpMsg::Loaded(Ok(vec![page]))));
    apply(
        &mut m,
        Msg::Help(HelpMsg::Open(PageRef::User("Mine".into()))),
    );
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("Edit").is_some());
    h.get_by_label("faq").click();
    h.run();
    assert_eq!(
        h.state().0.state.help.page,
        Some(PageRef::Supplied("faq".into()))
    );
    h.get_by_label("Previous page").click();
    h.run();
    h.get_by_label("Edit").click();
    h.run();
    assert!(h.query_by_label("Page name").is_some());
    assert!(h.query_by_label("Save page").is_some());
    assert_all_named(&h, "help editor");
}

#[test]
fn the_tutorial_card_guides_and_can_be_ended() {
    let mut m = without_toasts();
    apply(&mut m, Msg::Help(HelpMsg::Tutorial(TutorialMsg::Start)));
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label_contains("step 1 of 4").is_some());
    assert!(h.query_by_label("Create a stack").is_some());
    h.get_by_label("End tutorial").click();
    h.run();
    assert!(h.state().0.state.help.tutorial.is_none());
}
