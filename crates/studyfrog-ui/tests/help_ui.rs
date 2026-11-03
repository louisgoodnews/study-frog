#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The help screen's logic: the page for each screen, links, back, search and the user's pages.

mod common;

use common::flow::{opened, send};
use studyfrog_service::help::PageRef;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::help_types::{HelpEffect, HelpMsg};
use studyfrog_ui::help_ui::page_for;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection, State};

fn help(m: &mut Model, msg: HelpMsg) -> Vec<Effect> {
    apply(m, Msg::Help(msg))
}

fn supplied(id: &str) -> PageRef {
    PageRef::Supplied(id.into())
}

#[test]
fn every_screen_has_a_supplied_page() {
    let mut state = State::default();
    for screen in [
        Screen::Welcome,
        Screen::Home,
        Screen::Library,
        Screen::Rehearse,
        Screen::Settings,
        Screen::Editor,
        Screen::Graph,
        Screen::Calendar,
        Screen::Board,
        Screen::Achievements,
        Screen::Reports,
        Screen::Agent,
        Screen::Pdf,
        Screen::Help,
    ] {
        state.screen = screen;
        let id = page_for(&state);
        assert!(
            studyfrog_service::help::supplied(id).is_some(),
            "{screen:?} -> {id}"
        );
    }
    state.screen = Screen::Library;
    state.browser.selection = Selection::Stack("french".into());
    assert_eq!(page_for(&state), "stack-page");
}

#[test]
fn help_opens_without_a_vault_at_the_page_of_the_screen_and_returns() {
    let mut m = Model::default();
    let fx = help(&mut m, HelpMsg::ForScreen);
    assert_eq!(m.state.screen, Screen::Help);
    assert!(fx.is_empty(), "no vault: no user pages to read");
    assert_eq!(m.state.help.page, Some(supplied("getting-started")));
    assert!(!m.state.help.blocks.is_empty(), "parsed once on opening");
    help(&mut m, HelpMsg::Leave);
    assert_eq!(m.state.screen, Screen::Welcome);
}

#[test]
fn links_open_pages_back_goes_back_and_leave_returns_to_the_screen() {
    let (_v, mut m, mut w) = opened("help-links");
    send(&mut m, &mut w, Msg::Navigate(Screen::Calendar));
    send(&mut m, &mut w, Msg::Help(HelpMsg::ForScreen));
    assert_eq!(m.state.help.page, Some(supplied("calendar")));
    assert_eq!(m.state.help.came_from, Some(Screen::Calendar));
    help(&mut m, HelpMsg::Link("faq".into()));
    assert_eq!(m.state.help.page, Some(supplied("faq")));
    help(&mut m, HelpMsg::Link("no such page".into()));
    assert_eq!(m.state.help.page, Some(supplied("faq")), "unchanged");
    assert!(m.state.notice.is_some(), "says the page does not exist");
    help(&mut m, HelpMsg::Back);
    assert_eq!(m.state.help.page, Some(supplied("calendar")));
    assert_eq!(m.state.help.came_from, Some(Screen::Calendar));
    send(&mut m, &mut w, Msg::Help(HelpMsg::Leave));
    assert_eq!(m.state.screen, Screen::Calendar);
}

#[test]
fn search_lists_matching_pages() {
    let mut m = Model::default();
    help(&mut m, HelpMsg::Search("rehears".into()));
    assert!(
        m.state
            .help
            .hits
            .iter()
            .any(|h| h.page == supplied("rehearse"))
    );
    help(&mut m, HelpMsg::Search(String::new()));
    assert!(m.state.help.hits.is_empty());
}

#[test]
fn a_supplied_page_is_copied_saved_renamed_and_deleted_as_a_user_page() {
    let (v, mut m, mut w) = opened("help-pages");
    send(&mut m, &mut w, Msg::Help(HelpMsg::Open(supplied("faq"))));
    assert!(help(&mut m, HelpMsg::Edit).is_empty());
    assert!(m.state.help.edit.is_none(), "supplied pages are read-only");
    help(&mut m, HelpMsg::Copy);
    let edit = m.state.help.edit.clone().unwrap();
    assert!(edit.name.ends_with("(my copy)"), "{}", edit.name);
    help(&mut m, HelpMsg::EditName("My FAQ".into()));
    help(
        &mut m,
        HelpMsg::EditText("# My FAQ\nSee [[library]].".into()),
    );
    let fx = help(&mut m, HelpMsg::Save);
    assert_eq!(
        fx,
        [Effect::Help(HelpEffect::Save {
            old: None,
            name: "My FAQ".into(),
            text: "# My FAQ\nSee [[library]].".into(),
        })]
    );
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(v.path.join("wiki/My FAQ.md").is_file());
    assert_eq!(m.state.help.page, Some(PageRef::User("My FAQ".into())));
    assert!(m.state.help.edit.is_none());
    help(&mut m, HelpMsg::Link("library".into()));
    assert_eq!(m.state.help.page, Some(supplied("library")));
    help(&mut m, HelpMsg::Back);

    help(&mut m, HelpMsg::Edit);
    help(&mut m, HelpMsg::EditName("Answers".into()));
    let fx = help(&mut m, HelpMsg::Save);
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(v.path.join("wiki/Answers.md").is_file());
    assert!(!v.path.join("wiki/My FAQ.md").exists());

    help(&mut m, HelpMsg::AskDelete(true));
    assert!(m.state.help.ask_delete);
    let fx = help(&mut m, HelpMsg::Delete);
    common::run_to_idle(&mut m, &mut w, fx);
    assert!(!v.path.join("wiki/Answers.md").exists());
    assert!(m.state.help.user.is_empty());
    assert_eq!(m.state.help.page, Some(supplied("getting-started")));
}

#[test]
fn a_bad_name_is_refused_before_saving_and_the_text_is_kept() {
    let (_v, mut m, mut w) = opened("help-bad");
    send(&mut m, &mut w, Msg::Help(HelpMsg::ForScreen));
    help(&mut m, HelpMsg::New);
    help(&mut m, HelpMsg::EditText("# Kept".into()));
    help(&mut m, HelpMsg::EditName("a/b".into()));
    let fx = help(&mut m, HelpMsg::Save);
    assert!(!fx.iter().any(|e| matches!(e, Effect::Help(_))));
    let edit = m.state.help.edit.clone().unwrap();
    assert!(edit.problem.is_some());
    assert_eq!(edit.text, "# Kept");
    help(&mut m, HelpMsg::Cancel);
    assert!(m.state.help.edit.is_none());
}

#[test]
fn the_assistant_reads_the_user_pages_only_when_asked_and_shows_them_before_sending() {
    use studyfrog_service::agent_vault::WikiContext;
    use studyfrog_ui::agent_types::AgentMsg;
    let mut m = Model::default();
    apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    let page = studyfrog_service::help::UserPage {
        file: "Mine".into(),
        text: "# Mine".into(),
    };
    help(&mut m, HelpMsg::Loaded(Ok(vec![page])));
    let prepared = || Msg::Agent(AgentMsg::WikiPrepared(Ok(Box::new(WikiContext::default()))));
    apply(&mut m, prepared());
    assert!(m.state.agent.pending.as_ref().unwrap().items.is_empty());
    apply(&mut m, Msg::Agent(AgentMsg::UseHelpPages(true)));
    apply(&mut m, prepared());
    let pending = m.state.agent.pending.clone().unwrap();
    assert_eq!(pending.items.len(), 1);
    assert_eq!(pending.items[0].uid, "help:Mine");
    let fx = apply(&mut m, Msg::Navigate(Screen::Agent));
    assert!(fx.contains(&Effect::Help(HelpEffect::Load)), "read fresh");
}
