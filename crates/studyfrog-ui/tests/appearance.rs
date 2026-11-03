#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Appearance and start-up settings in the state: stored and saved at once, apart per setting.

mod common;

use common::flow::{opened, send};
use studyfrog_core::appearance::{
    Accent, Appearance, Density, StartScreen, ThemeChoice, WindowState,
};
use studyfrog_ui::appearance_ui::{AppearanceMsg, start_screen, wants_window};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

fn set(m: &mut Model, a: AppearanceMsg) -> Vec<Effect> {
    apply(m, Msg::Appearance(a))
}

fn saves(fx: &[Effect]) -> usize {
    fx.iter()
        .filter(|e| matches!(e, Effect::SaveAppSettings(_)))
        .count()
}

#[test]
fn every_setting_is_stored_and_saved_when_it_changes_and_nothing_is_saved_when_it_does_not() {
    let mut m = Model::default();
    for (msg, check) in [
        (AppearanceMsg::Theme(ThemeChoice::Light), "theme"),
        (AppearanceMsg::Zoom(125), "zoom"),
        (AppearanceMsg::Text(110), "text"),
        (AppearanceMsg::Density(Density::Compact), "density"),
        (AppearanceMsg::Accent(Accent::Green), "accent"),
        (AppearanceMsg::HighContrast(true), "high contrast"),
        (AppearanceMsg::ReducedMotion(true), "motion"),
        (AppearanceMsg::Start(StartScreen::Library), "start"),
        (AppearanceMsg::SidebarFolded(true), "sidebar"),
        (AppearanceMsg::RememberWindow(false), "remember"),
    ] {
        let fx = set(&mut m, msg);
        assert_eq!(saves(&fx), 1, "{check} is saved");
        let again = set(&mut m, msg);
        assert_eq!(saves(&again), 0, "{check} unchanged: nothing to save");
    }
    let a = &m.state.app.appearance;
    assert_eq!(
        (a.theme, a.zoom, a.text, a.density, a.accent),
        (
            ThemeChoice::Light,
            125,
            110,
            Density::Compact,
            Accent::Green
        )
    );
    assert!(a.high_contrast && a.reduced_motion && a.sidebar_folded && !a.remember_window);
}

#[test]
fn zoom_steps_through_the_sizes_stays_at_the_ends_and_resets() {
    let mut m = Model::default();
    set(&mut m, AppearanceMsg::ZoomStep(1));
    assert_eq!(m.state.app.appearance.zoom, 110);
    set(&mut m, AppearanceMsg::ZoomStep(1));
    assert_eq!(m.state.app.appearance.zoom, 125);
    set(&mut m, AppearanceMsg::ZoomStep(-1));
    set(&mut m, AppearanceMsg::ZoomStep(-1));
    set(&mut m, AppearanceMsg::ZoomStep(-1));
    assert_eq!(m.state.app.appearance.zoom, 90);
    for _ in 0..10 {
        set(&mut m, AppearanceMsg::ZoomStep(-1));
    }
    assert_eq!(m.state.app.appearance.zoom, 70, "the smallest");
    set(&mut m, AppearanceMsg::ZoomReset);
    assert_eq!(m.state.app.appearance.zoom, 100);
    set(&mut m, AppearanceMsg::Zoom(137));
    set(&mut m, AppearanceMsg::ZoomStep(1));
    assert_eq!(
        m.state.app.appearance.zoom, 150,
        "an odd size snaps to the nearest step"
    );
}

#[test]
fn defaults_restore_the_look_but_keep_the_window_and_the_last_screen() {
    let mut m = Model::default();
    set(&mut m, AppearanceMsg::Accent(Accent::Pink));
    set(&mut m, AppearanceMsg::Zoom(150));
    m.state.app.appearance.last_screen = Some("board".into());
    let w = WindowState {
        width: 900,
        height: 700,
        x: Some(1),
        y: Some(2),
        maximized: false,
    };
    set(&mut m, AppearanceMsg::Window(w));
    set(&mut m, AppearanceMsg::Defaults);
    let a = &m.state.app.appearance;
    assert_eq!((a.accent, a.zoom), (Accent::Blue, 100));
    assert_eq!(
        (a.window, a.last_screen.as_deref()),
        (Some(w), Some("board"))
    );
}

#[test]
fn the_window_is_saved_only_when_remembering_is_on_and_it_changed() {
    let mut a = Appearance::default();
    let w = WindowState {
        width: 1000,
        height: 700,
        x: Some(10),
        y: Some(20),
        maximized: false,
    };
    assert!(wants_window(&a, w));
    a.window = Some(w);
    assert!(!wants_window(&a, w), "the same window is not saved again");
    let max = WindowState {
        width: 1920,
        height: 1080,
        maximized: true,
        ..w
    };
    assert!(wants_window(&a, max), "becoming maximised is saved");
    let mut m = Model::default();
    m.state.app.appearance.window = Some(w);
    set(&mut m, AppearanceMsg::Window(max));
    assert_eq!(
        m.state.app.appearance.window,
        Some(WindowState {
            maximized: true,
            ..w
        }),
        "a maximised window keeps the size and place it had"
    );
    set(&mut m, AppearanceMsg::RememberWindow(false));
    let fx = set(
        &mut m,
        AppearanceMsg::Window(WindowState { width: 640, ..w }),
    );
    assert_eq!(saves(&fx), 0, "not remembering: nothing is saved");
    assert!(!wants_window(
        &m.state.app.appearance,
        WindowState { width: 1, ..w }
    ));
}

#[test]
fn the_start_screen_follows_the_setting_and_the_last_screen_is_remembered() {
    let mut a = Appearance::default();
    assert_eq!(start_screen(&a), Screen::Home);
    a.start = StartScreen::Library;
    assert_eq!(start_screen(&a), Screen::Library);
    a.start = StartScreen::Last;
    assert_eq!(start_screen(&a), Screen::Home, "nothing remembered yet");
    a.last_screen = Some("calendar".into());
    assert_eq!(start_screen(&a), Screen::Calendar);
    a.last_screen = Some("nonsense".into());
    assert_eq!(start_screen(&a), Screen::Home);

    let (_v, mut m, mut w) = opened("appearance-last");
    set(&mut m, AppearanceMsg::Start(StartScreen::Last));
    send(&mut m, &mut w, Msg::Navigate(Screen::Board));
    assert_eq!(m.state.app.appearance.last_screen.as_deref(), Some("board"));
    let fx = apply(&mut m, Msg::Navigate(Screen::Reports));
    assert_eq!(saves(&fx), 1, "the new last screen is saved");
    assert_eq!(
        m.state.app.appearance.last_screen.as_deref(),
        Some("reports")
    );
}

#[test]
fn a_vault_opens_on_the_chosen_screen_and_the_sidebar_starts_folded_when_asked() {
    let mut m = Model::default();
    m.state.app.appearance.start = StartScreen::Library;
    m.state.app.appearance.sidebar_folded = true;
    let app = m.state.app.clone();
    apply(&mut m, Msg::Started { app, error: None });
    assert!(m.state.chrome.sidebar_hidden, "folded at start");
    m.state.app.tutorial_offered = true;
    apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    assert_eq!(m.state.screen, Screen::Library);
    m.state.app.appearance.start = StartScreen::Home;
    apply(&mut m, Msg::VaultOpened(Ok(common::views::info())));
    assert_eq!(m.state.screen, Screen::Home);
}
