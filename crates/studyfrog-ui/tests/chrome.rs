#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The sidebar, menus, notifications, global search and Create menu (update logic).

mod common;

use common::views::open_model;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::chrome::{ChromeMsg, MAX_NOTIFICATIONS, Menu, unread};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

fn chrome(m: &mut Model, msg: ChromeMsg) -> Vec<Effect> {
    apply(m, Msg::Chrome(msg))
}

/// A notice made the way the app makes them: an update that fails.
fn notify(m: &mut Model, text: &str) {
    chrome(m, ChromeMsg::SearchEdited("q".into()));
    chrome(m, ChromeMsg::SearchResults("q".into(), Err(text.into())));
}

fn row(n: u128, title: &str) -> ItemRow {
    ItemRow {
        uid: Uid(n),
        title: title.into(),
        kind: ItemKind::Card,
        path: format!("{n}.md"),
        due: None,
    }
}

#[test]
fn the_sidebar_folds_away_and_comes_back() {
    let mut m = open_model();
    assert!(!m.state.chrome.sidebar_hidden);
    chrome(&mut m, ChromeMsg::ToggleSidebar);
    assert!(m.state.chrome.sidebar_hidden);
    chrome(&mut m, ChromeMsg::ToggleSidebar);
    assert!(!m.state.chrome.sidebar_hidden);
}

#[test]
fn one_menu_is_open_at_a_time_and_opening_the_bell_marks_notices_seen() {
    let mut m = open_model();
    chrome(&mut m, ChromeMsg::ToggleMenu(Menu::Profile));
    assert_eq!(m.state.chrome.menu, Some(Menu::Profile));
    chrome(&mut m, ChromeMsg::ToggleMenu(Menu::Help));
    assert_eq!(
        m.state.chrome.menu,
        Some(Menu::Help),
        "another menu replaces it"
    );
    chrome(&mut m, ChromeMsg::ToggleMenu(Menu::Help));
    assert_eq!(m.state.chrome.menu, None, "the same button closes it");
    chrome(&mut m, ChromeMsg::ToggleMenu(Menu::Profile));
    chrome(&mut m, ChromeMsg::CloseMenu);
    assert_eq!(m.state.chrome.menu, None);

    let before = unread(&m.state.chrome);
    notify(&mut m, "first");
    assert_eq!(unread(&m.state.chrome), before + 1);
    chrome(&mut m, ChromeMsg::ToggleMenu(Menu::Notifications));
    assert_eq!(
        unread(&m.state.chrome),
        0,
        "opening the list marks them seen"
    );
}

#[test]
fn every_new_notice_is_kept_once_and_the_list_is_capped() {
    let mut m = Model::default();
    m.state.vault = open_model().state.vault;
    for i in 0..MAX_NOTIFICATIONS + 5 {
        notify(&mut m, &format!("n{i}"));
    }
    let list = &m.state.chrome.notifications;
    assert_eq!(list.len(), MAX_NOTIFICATIONS);
    assert_eq!(
        (list[0].text.as_str(), list.last().unwrap().text.as_str()),
        ("Search failed: n5", "Search failed: n34")
    );
    apply(&mut m, Msg::Tick);
    assert_eq!(
        m.state.chrome.notifications.len(),
        MAX_NOTIFICATIONS,
        "an unchanged notice is not added again"
    );
    assert_eq!(unread(&m.state.chrome), MAX_NOTIFICATIONS);
}

#[test]
fn searching_asks_for_matches_and_ignores_stale_answers() {
    let mut m = open_model();
    assert_eq!(
        chrome(&mut m, ChromeMsg::SearchEdited(" bon ".into())),
        [Effect::SearchItems("bon".into())]
    );
    chrome(
        &mut m,
        ChromeMsg::SearchResults("bonj".into(), Ok(vec![row(9, "stale")])),
    );
    assert!(
        m.state.chrome.results.is_empty(),
        "an answer for an older query is dropped"
    );
    chrome(
        &mut m,
        ChromeMsg::SearchResults("bon".into(), Ok(vec![row(1, "Bonjour")])),
    );
    assert_eq!(m.state.chrome.results[0].title, "Bonjour");

    assert!(
        chrome(&mut m, ChromeMsg::SearchEdited("   ".into())).is_empty(),
        "nothing to look for"
    );
    assert!(m.state.chrome.results.is_empty(), "and the old results go");
    chrome(&mut m, ChromeMsg::SearchEdited("x".into()));
    chrome(
        &mut m,
        ChromeMsg::SearchResults("x".into(), Err("disk".into())),
    );
    assert!(
        m.state
            .notice
            .as_ref()
            .unwrap()
            .text
            .contains("Search failed: disk")
    );
    chrome(&mut m, ChromeMsg::SearchClear);
    assert!(m.state.chrome.search.is_empty() && m.state.chrome.results.is_empty());

    let mut none = Model::default();
    assert!(
        chrome(&mut none, ChromeMsg::SearchEdited("x".into())).is_empty(),
        "no vault, no search"
    );
}
