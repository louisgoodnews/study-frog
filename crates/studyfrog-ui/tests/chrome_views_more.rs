#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The top bar, sidebar and menus, drawn headless.
use common::views::{harness, open_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::chrome::ChromeMsg;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::effect::EffectBrowse;
use studyfrog_ui::effect::EffectEditor;
use studyfrog_ui::effect::EffectVault;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;

mod common;

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
    let title = format!("StudyFrog {}", env!("CARGO_PKG_VERSION"));
    assert!(h.query_by_label_contains(&title).is_some());
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
        h.state()
            .1
            .contains(&Effect::Browse(EffectBrowse::SearchItems("bon".into()))),
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
    assert!(
        h.state()
            .1
            .contains(&Effect::Editor(EffectEditor::LoadDocument(Uid(7))))
    );
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
        Effect::Vault(EffectVault::OpenVault(vault.path.display().to_string())),
        NOW_NS,
    );
    let msgs = execute(
        &mut w,
        Effect::Browse(EffectBrowse::SearchItems("loose".into())),
        NOW_NS,
    );
    let Msg::Chrome(ChromeMsg::SearchResults(q, Ok(found))) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert_eq!(
        (q.as_str(), found.len(), found[0].title.as_str()),
        ("loose", 1, "Loose")
    );
}
