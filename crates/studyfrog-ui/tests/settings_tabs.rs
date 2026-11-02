#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The tabs of the settings screen: one tab at a time, remembered, reachable from the keyboard.

mod common;

use common::views::{assert_all_named, harness, open_model};
use eframe::egui::Key;
use egui_kittest::kittest::Queryable;
use studyfrog_ui::app_settings_ui::{
    SHOWN, SettingsTab, SettingsTabMsg, current_tab, tab_from_key, tab_key, tab_label,
};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn model(tab: Option<&str>) -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Settings;
    m.state.app.settings_tab = tab.map(str::to_owned);
    m
}

#[test]
fn choosing_a_tab_is_remembered_in_the_app_settings_and_unknown_keys_fall_back() {
    let mut m = model(None);
    assert_eq!(
        current_tab(&m.state.app),
        SettingsTab::Study,
        "the first tab by default"
    );
    let fx = apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::Select(SettingsTab::Data)),
    );
    let [Effect::SaveAppSettings(app)] = &fx[..] else {
        panic!("{fx:?}")
    };
    assert_eq!(app.settings_tab.as_deref(), Some("data"));
    assert_eq!(current_tab(&m.state.app), SettingsTab::Data);
    assert!(
        apply(
            &mut m,
            Msg::SettingsTab(SettingsTabMsg::Select(SettingsTab::Data))
        )
        .is_empty(),
        "the same tab again saves nothing"
    );
    for odd in [Some("nonsense"), Some("general")] {
        assert_eq!(
            current_tab(&model(odd).state.app),
            SettingsTab::Study,
            "{odd:?}"
        );
    }
}

#[test]
fn stepping_moves_between_the_tabs_and_wraps_around() {
    let mut m = model(Some("subjects"));
    apply(&mut m, Msg::SettingsTab(SettingsTabMsg::Step(1)));
    assert_eq!(
        current_tab(&m.state.app),
        SHOWN[0],
        "after the last comes the first"
    );
    apply(&mut m, Msg::SettingsTab(SettingsTabMsg::Step(-1)));
    assert_eq!(current_tab(&m.state.app), SettingsTab::Subjects, "and back");
}

#[test]
fn every_tab_has_a_unique_key_and_label_that_read_back() {
    for tab in SHOWN {
        assert_eq!(tab_from_key(tab_key(tab)), Some(tab));
        assert!(!tab_label(tab).is_empty());
    }
    for tab in [
        SettingsTab::General,
        SettingsTab::Appearance,
        SettingsTab::Notifications,
    ] {
        assert!(!SHOWN.contains(&tab));
    }
}

/// The headings of the settings that exist, each of which must be on exactly one tab.
const SETTINGS: [&str; 5] = [
    "Reading speed",
    "Answer comparison",
    "Teachers",
    "Backup and restore",
    "Scheduling",
];

#[test]
fn every_setting_is_on_exactly_one_tab() {
    let mut seen = [0_usize; 5];
    for tab in SHOWN {
        let mut h = harness(model(Some(tab_key(tab))));
        h.run();
        assert_all_named(&h, tab_label(tab));
        for (i, heading) in SETTINGS.iter().enumerate() {
            seen[i] += h.query_all_by_label(heading).count().min(1);
        }
    }
    assert_eq!(seen, [1; 5], "{SETTINGS:?}");
}

#[test]
fn the_tab_bar_names_every_shown_tab_and_a_click_changes_the_tab() {
    let mut h = harness(model(None));
    h.run();
    for tab in SHOWN {
        assert!(
            h.query_by_label(tab_label(tab)).is_some(),
            "{}",
            tab_label(tab)
        );
    }
    h.get_by_label("Rehearsal").click();
    h.run();
    assert_eq!(current_tab(&h.state().0.state.app), SettingsTab::Rehearsal);
    assert!(h.query_by_label("Reading speed").is_some());
}

#[test]
fn left_and_right_keys_move_between_the_tabs() {
    let mut h = harness(model(Some("rehearsal")));
    h.run();
    h.key_press(Key::ArrowRight);
    h.run();
    assert_eq!(current_tab(&h.state().0.state.app), SettingsTab::Answers);
    h.key_press(Key::ArrowLeft);
    h.run();
    assert_eq!(current_tab(&h.state().0.state.app), SettingsTab::Rehearsal);
}
