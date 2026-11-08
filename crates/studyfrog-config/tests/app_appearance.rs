#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The `[appearance]` table of app.toml.

use studyfrog_config::app::{parse_app_settings, render_app_settings};
use studyfrog_core::appearance::{
    Accent, Appearance, Density, StartScreen, ThemeChoice, WindowState,
};
use studyfrog_core::settings::AppSettings;

fn custom() -> Appearance {
    Appearance {
        theme: ThemeChoice::Dark,
        zoom: 125,
        text: 110,
        density: Density::Spacious,
        accent: Accent::Teal,
        high_contrast: true,
        reduced_motion: true,
        start: StartScreen::Last,
        last_screen: Some("calendar".into()),
        sidebar_folded: true,
        zen_in_runs: true,
        remember_window: false,
        window: Some(WindowState {
            width: 1280,
            height: 800,
            x: Some(-20),
            y: Some(40),
            maximized: true,
        }),
    }
}

#[test]
fn every_setting_survives_a_write_and_a_read() {
    let s = AppSettings {
        appearance: custom(),
        ..AppSettings::default()
    };
    let text = render_app_settings(&s);
    assert!(text.contains("[appearance]"), "{text}");
    assert_eq!(parse_app_settings(&text).unwrap(), s);
}

#[test]
fn defaults_write_nothing_and_a_missing_table_reads_as_defaults() {
    let text = render_app_settings(&AppSettings::default());
    assert!(!text.contains("appearance"), "{text}");
    assert_eq!(
        parse_app_settings(&text).unwrap().appearance,
        Appearance::default()
    );
    let partial = parse_app_settings("[appearance]\ntheme = \"light\"\n").unwrap();
    assert_eq!(partial.appearance.theme, ThemeChoice::Light);
    assert_eq!(partial.appearance.zoom, 100);
}

#[test]
fn a_bad_value_names_its_key_and_line() {
    for (text, key) in [
        ("[appearance]\ntheme = \"blue\"\n", "theme"),
        ("[appearance]\nzoom = 900\n", "zoom"),
        ("[appearance]\ntext_size = 5\n", "text_size"),
        ("[appearance]\ndensity = \"tight\"\n", "density"),
        ("[appearance]\naccent = \"red\"\n", "accent"),
        ("[appearance]\nstart_screen = \"board\"\n", "start_screen"),
    ] {
        let e = parse_app_settings(text).unwrap_err().to_string();
        assert!(e.contains(key), "{e}");
    }
    assert!(
        parse_app_settings("[appearance]\nfont = 3\n").is_err(),
        "unknown keys are refused"
    );
}

#[test]
fn a_window_without_both_sizes_is_not_remembered() {
    let s = parse_app_settings("[appearance]\nwindow_width = 900\n").unwrap();
    assert_eq!(s.appearance.window, None);
}
