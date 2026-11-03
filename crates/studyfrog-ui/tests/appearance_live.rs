#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Appearance settings shown at once on the running application, from the keyboard and from the Settings tab.

mod common;

use common::views::{assert_all_named, harness, open_model};
use eframe::egui::{self, Key, Modifiers};
use egui_kittest::kittest::Queryable;
use studyfrog_core::appearance::{Accent, Density, StartScreen, ThemeChoice};
use studyfrog_ui::appearance_ui::AppearanceMsg;
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn style_of(
    h: &egui_kittest::Harness<'_, common::views::App>,
) -> (egui::ThemePreference, f32, f32, f32) {
    let ctx = &h.ctx;
    (
        ctx.options(|o| o.theme_preference),
        ctx.zoom_factor(),
        ctx.style_of(ctx.theme()).spacing.interact_size.y,
        ctx.style_of(ctx.theme()).text_styles[&egui::TextStyle::Body].size,
    )
}

#[test]
fn a_change_shows_at_once_on_the_running_application_without_a_restart() {
    let mut m = open_model();
    m.state.toasts.clear();
    m.state.app.appearance.theme = ThemeChoice::Dark; // a look is in use from the start
    let mut h = harness(m);
    h.run_steps(3);
    assert_eq!(style_of(&h), (egui::ThemePreference::Dark, 1.0, 30.0, 15.0));
    let apply_now = |h: &mut egui_kittest::Harness<'_, common::views::App>, a| {
        apply(&mut h.state_mut().0, Msg::Appearance(a));
        h.run_steps(2);
    };
    apply_now(&mut h, AppearanceMsg::Theme(ThemeChoice::Light));
    assert_eq!(style_of(&h).0, egui::ThemePreference::Light);
    apply_now(&mut h, AppearanceMsg::Theme(ThemeChoice::System));
    assert_eq!(style_of(&h).0, egui::ThemePreference::System);
    apply_now(&mut h, AppearanceMsg::Zoom(150));
    assert_eq!(style_of(&h).1, 1.5);
    apply_now(&mut h, AppearanceMsg::Text(125));
    assert_eq!(style_of(&h).3, 15.0 * 1.25);
    apply_now(&mut h, AppearanceMsg::Density(Density::Spacious));
    assert_eq!(style_of(&h).2, 34.0);
    apply_now(&mut h, AppearanceMsg::Density(Density::Compact));
    assert_eq!(style_of(&h).2, 26.0);
    apply_now(&mut h, AppearanceMsg::ReducedMotion(true));
    assert_eq!(h.ctx.style_of(h.ctx.theme()).animation_time, 0.0);
    apply_now(&mut h, AppearanceMsg::ReducedMotion(false));
    assert!(
        h.ctx.style_of(h.ctx.theme()).animation_time > 0.0,
        "the animations come back"
    );
    let accent = |h: &egui_kittest::Harness<'_, common::views::App>| {
        studyfrog_ui::theme::look_of(&h.ctx).accent
    };
    apply_now(&mut h, AppearanceMsg::Accent(Accent::Purple));
    assert_eq!(accent(&h), Accent::Purple);
    apply_now(&mut h, AppearanceMsg::Defaults);
    assert_eq!(
        style_of(&h),
        (egui::ThemePreference::System, 1.0, 30.0, 15.0)
    );
    assert_eq!(accent(&h), Accent::Blue);
}

#[test]
fn ctrl_plus_ctrl_minus_and_ctrl_zero_change_the_size_anywhere() {
    let mut m = open_model();
    m.state.toasts.clear();
    let mut h = harness(m);
    h.run();
    h.key_press_modifiers(Modifiers::COMMAND, Key::Equals);
    h.run();
    assert_eq!(h.state().0.state.app.appearance.zoom, 110);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Minus);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Minus);
    h.run();
    assert_eq!(h.state().0.state.app.appearance.zoom, 90);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Num0);
    h.run();
    assert_eq!(h.state().0.state.app.appearance.zoom, 100);
}

#[test]
fn the_appearance_tab_offers_every_setting_with_names_and_a_click_changes_it() {
    let mut m = open_model();
    m.state.toasts.clear();
    m.state.app.settings_tab = Some("appearance".into());
    apply(&mut m, Msg::Navigate(Screen::Settings));
    // Tall enough for every group, so no control is below the fold.
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 2200.0))
        .build_ui_state(
            |ui, (model, effects): &mut common::views::App| {
                effects.extend(studyfrog_ui::model::frame(model, ui))
            },
            (m, Vec::new()),
        );
    h.run_steps(3);
    assert_all_named(&h, "appearance tab");
    for name in [
        "Theme: Dark",
        "Theme: Light",
        "Theme: Follow system",
        "Interface size",
        "Text size",
        "Density: Compact",
        "Density: Comfortable",
        "Density: Spacious",
        "Accent colour: blue",
        "Accent colour: teal",
        "High contrast",
        "Reduce motion",
        "Start screen: Home",
        "Start screen: Library",
        "Start screen: The screen I used last",
        "Start with the sidebar folded away",
        "Remember the window's size and place",
        "Restore the default appearance",
    ] {
        assert!(h.query_all_by_label(name).count() >= 1, "missing {name}");
    }
    h.get_by_label("Theme: Light").click();
    h.run_steps(6); // a change of the look re-lays everything out; let it settle before the next click
    assert_eq!(h.state().0.state.app.appearance.theme, ThemeChoice::Light);
    for name in [
        "Accent colour: orange",
        "Density: Spacious",
        "High contrast",
        "Start screen: The screen I used last",
    ] {
        h.get_by_label(name).click();
        h.run_steps(6); // a change of the look re-lays everything out; let it settle before the next click
    }
    let a = &h.state().0.state.app.appearance;
    assert_eq!(
        (a.accent, a.density, a.high_contrast, a.start),
        (Accent::Orange, Density::Spacious, true, StartScreen::Last)
    );
    h.get_by_label("Restore the default appearance").click();
    h.run_steps(2);
    assert_eq!(h.state().0.state.app.appearance.accent, Accent::Blue);
}

#[test]
fn the_profile_menu_switches_the_theme_too() {
    let mut m = open_model();
    m.state.toasts.clear();
    let mut h = harness(m);
    h.run();
    h.get_by_label("Profile and vault").click();
    h.run();
    h.get_by_label("Theme: Dark").click();
    h.run();
    assert_eq!(h.state().0.state.app.appearance.theme, ThemeChoice::Dark);
}
