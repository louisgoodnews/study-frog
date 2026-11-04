#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! UI tests for Diary Settings and effects.

mod common;

use common::views::{assert_all_named, harness, open_model};
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;
use studyfrog_ui::app_settings_ui::{SettingsTab, SettingsTabMsg, current_tab};
use studyfrog_ui::diary_filter::has_entry_for_date;
use studyfrog_ui::diary_msg::DiaryMsg;
use studyfrog_ui::diary_types::{CustomScaleDef, DiaryEffect};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

#[test]
fn save_and_delete_entry_effects() {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Diary));
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::NewEntry(Date {
            year: 2026,
            month: 11,
            day: 4,
        })),
    );

    let fx = apply(&mut m, Msg::Diary(DiaryMsg::SaveDraft));
    assert!(m.state.diary.active_entry.is_none());
    assert_eq!(fx.len(), 1);
    assert!(matches!(fx[0], Effect::Diary(DiaryEffect::SaveEntry(_))));

    let del_fx = apply(&mut m, Msg::Diary(DiaryMsg::DeleteEntry(Uid(10))));
    assert_eq!(del_fx.len(), 1);
    assert!(matches!(
        del_fx[0],
        Effect::Diary(DiaryEffect::DeleteEntry(Uid(10)))
    ));
}

#[test]
fn settings_toggle_and_home_hint() {
    let mut m = open_model();
    let today = Date {
        year: 2026,
        month: 11,
        day: 4,
    };
    m.state.wall_ms = 1_793_793_600_000; // 2026-11-04 approx
    assert!(!has_entry_for_date(&m.state.diary.entries, today));
    assert!(m.state.diary.settings.show_home_hint);

    // Dismiss Home hint
    apply(&mut m, Msg::Diary(DiaryMsg::DismissHomeHint));
    assert!(m.state.diary.hint_dismissed);

    // Settings screen Diary tab
    apply(&mut m, Msg::Navigate(Screen::Settings));
    apply(
        &mut m,
        Msg::SettingsTab(SettingsTabMsg::Select(SettingsTab::Diary)),
    );
    assert_eq!(current_tab(&m.state.app), SettingsTab::Diary);

    // Toggle home hint setting off
    apply(&mut m, Msg::Diary(DiaryMsg::SetShowHomeHint(false)));
    assert!(!m.state.diary.settings.show_home_hint);

    // Toggle standard scale visibility
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::ToggleScaleVisibility("sleep".into(), false)),
    );
    assert!(
        !m.state
            .diary
            .settings
            .visible_scales
            .contains(&"sleep".to_string())
    );

    // Add custom scale
    apply(
        &mut m,
        Msg::Diary(DiaryMsg::AddCustomScale(CustomScaleDef {
            id: "caffeine".into(),
            name: "Caffeine".into(),
            prompt: "Cups drank".into(),
            min: 1,
            max: 5,
            labels: vec![
                "None".into(),
                "One".into(),
                "Two".into(),
                "Three".into(),
                "Jittery".into(),
            ],
        })),
    );
    assert_eq!(m.state.diary.settings.custom_scales.len(), 1);

    let h = harness(m);
    assert_all_named(&h, "settings diary tab");
}
