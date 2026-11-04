#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The top bar, sidebar and menus, drawn headless.

mod common;

use common::views::{assert_all_named, harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::chrome::{ChromeMsg, Menu, unread};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::modal::Modal;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

const BAR: [&str; 8] = [
    "Show or hide the sidebar",
    "Switch vault",
    "Search items",
    "+ Create",
    "Notifications",
    "Help for this screen (F1)",
    "Application settings",
    "Profile and vault",
];

#[test]
fn the_top_bar_has_every_control_named_and_the_sidebar_lists_the_screens() {
    let mut h = harness(open_model());
    h.run();
    for name in BAR {
        assert!(h.query_by_label(name).is_some(), "missing {name}");
    }
    for screen in [
        "Home",
        "Folders",
        "Items",
        "Calendar",
        "Board",
        "Achievements",
        "Reports",
        "Graph",
        "Rehearse",
        "Settings",
    ] {
        assert!(h.query_by_label(screen).is_some(), "sidebar lacks {screen}");
    }
    assert!(
        h.query_by_label("notes").is_some(),
        "the bar shows the vault name next to StudyFrog"
    );
    assert!(
        h.query_by_label("v").is_none() && h.query_by_label("/v/notes").is_none(),
        "the path is not shown, only the folder name"
    );
    assert_all_named(&h, "top bar");
}

#[test]
fn without_a_vault_only_the_brand_and_help_are_shown() {
    let mut h = harness(Model::default());
    h.run();
    assert!(
        h.query_all_by_label("StudyFrog").count() >= 1
            && h.query_by_label("Help for this screen (F1)").is_some()
    );
    for gone in [
        "Search items",
        "+ Create",
        "Notifications",
        "Show or hide the sidebar",
        "Folders",
        "Items",
    ] {
        assert!(h.query_by_label(gone).is_none(), "{gone} needs a vault");
    }
    assert_all_named(&h, "welcome bar");
}

#[test]
fn the_sidebar_toggle_hides_and_shows_the_screen_list() {
    let mut h = harness(open_model());
    h.run();
    h.get_by_label("Show or hide the sidebar").click();
    h.run();
    assert!(h.query_by_label("Calendar").is_none(), "folded away");
    h.get_by_label("Show or hide the sidebar").click();
    h.run();
    h.get_by_label("Calendar").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Calendar);
}

#[test]
fn the_bell_shows_a_badge_and_a_list_of_notices_and_clicking_elsewhere_closes_it() {
    let mut m = open_model();
    apply(&mut m, Msg::Chrome(ChromeMsg::SearchEdited("q".into())));
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::SearchResults(
            "q".into(),
            Err("Disk is full".into()),
        )),
    );
    apply(&mut m, Msg::Chrome(ChromeMsg::SearchClear));
    let mut h = harness(m);
    h.run();
    h.get_by_label("Notifications").click();
    h.run();
    assert_eq!(h.state().0.state.chrome.menu, Some(Menu::Notifications));
    assert!(h.query_all_by_label("Search failed: Disk is full").count() >= 1);
    assert_eq!(
        unread(&h.state().0.state.chrome),
        1,
        "opening the list does not mark anything read"
    );
    h.get_by_label("Calendar").click();
    h.run();
    assert_eq!(
        h.state().0.state.chrome.menu,
        None,
        "a click outside closes the menu"
    );
}

#[test]
fn the_create_button_opens_the_create_dialog() {
    let mut h = harness(open_model());
    h.run();
    h.get_by_label("+ Create").click();
    h.run_steps(8);
    assert!(matches!(h.state().0.state.modal, Some(Modal::Create(_))));
    assert!(
        h.query_by_label("Title").is_some(),
        "the dialog is on screen"
    );
}

#[test]
fn the_vault_switcher_offers_the_other_recent_vaults() {
    let mut m = open_model();
    m.state.app.recent = vec!["/v/notes".into(), "/v/other".into()];
    let mut h = harness(m);
    h.run();
    h.get_by_label("Switch vault").click();
    h.run();
    assert!(h.query_by_label("Open: notes").is_some());
    assert!(
        h.query_by_label("/v/notes").is_none(),
        "the open vault is not offered again"
    );
    h.get_by_label("/v/other").click();
    h.run();
    assert!(h.state().1.contains(&Effect::OpenVault("/v/other".into())));
}

#[test]
fn the_help_button_opens_help_and_the_profile_menu_closes_the_vault() {
    let mut m = open_model();
    m.state.toasts.clear();
    let mut h = harness(m);
    h.run();
    h.get_by_label("Help for this screen (F1)").click();
    h.run();
    assert_eq!(h.state().0.state.screen, studyfrog_ui::state::Screen::Help);
    h.get_by_label("Profile and vault").click();
    h.run();
    assert!(h.query_by_label_contains("StudyFrog 0.").is_some());
    h.get_by_label("Close vault").click();
    h.run();
    assert!(h.state().0.state.vault.is_none());
}

#[test]
fn typing_in_the_search_asks_for_matches_and_choosing_one_opens_it() {
    let mut h = harness(open_model());
    h.run();
    h.get_by_label("Search items").focus();
    h.get_by_label("Search items").type_text("bon");
    h.run();
    assert!(
        h.state().1.contains(&Effect::SearchItems("bon".into())),
        "{:?}",
        h.state().1
    );

    let hit = ItemRow {
        uid: Uid(7),
        title: "Bonjour".into(),
        kind: ItemKind::Card,
        path: "french/a.md".into(),
        due: None,
        status: studyfrog_core::status::Status::New,
    };
    apply(
        &mut h.state_mut().0,
        Msg::Chrome(ChromeMsg::SearchResults("bon".into(), Ok(vec![hit]))),
    );
    h.run();
    h.get_by_label("card · Bonjour").click();
    h.run();
    assert!(h.state().1.contains(&Effect::LoadDocument(Uid(7))));
    assert!(
        h.state().0.state.chrome.search.is_empty(),
        "the search is cleared"
    );
}

#[test]
fn a_search_with_no_match_says_so() {
    let mut m = open_model();
    apply(&mut m, Msg::Chrome(ChromeMsg::SearchEdited("zzz".into())));
    apply(
        &mut m,
        Msg::Chrome(ChromeMsg::SearchResults("zzz".into(), Ok(vec![]))),
    );
    let mut h = harness(m);
    h.run();
    assert!(h.query_by_label("No items match \"zzz\".").is_some());
}

#[test]
fn the_executor_searches_a_real_vault() {
    let vault = sample_vault("search-ui");
    let mut w = worker_new(None);
    execute(
        &mut w,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let msgs = execute(&mut w, Effect::SearchItems("loose".into()), NOW_NS);
    let Msg::Chrome(ChromeMsg::SearchResults(q, Ok(found))) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(
        (q.as_str(), found.len(), found[0].title.as_str()),
        ("loose", 1, "Loose")
    );
}
