#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Memory-game records in `app.toml` round-trip through render and parse (STORY-0175).

use studyfrog_config::app::{parse_app_settings, render_app_settings};
use studyfrog_core::settings::{AppSettings, MemoryBest};

#[test]
fn memory_records_survive_a_save_and_reload() {
    let settings = AppSettings {
        first_run: false,
        vault: Some("/home/me/vault".to_owned()),
        memory_best: vec![
            MemoryBest {
                vault: "/home/me/vault".to_owned(),
                stack: "bio".to_owned(),
                best_ms: 41_250,
            },
            MemoryBest {
                vault: "/home/me/other".to_owned(),
                stack: String::new(),
                best_ms: 90_000,
            },
        ],
        ..AppSettings::default()
    };
    let text = render_app_settings(&settings);
    let back = parse_app_settings(&text).expect("the saved file parses");
    assert_eq!(back.memory_best, settings.memory_best, "{text}");
    assert_eq!(back.vault, settings.vault);
}

#[test]
fn a_file_without_records_has_none() {
    let back = parse_app_settings("vault = \"/v\"\n").expect("parses");
    assert!(back.memory_best.is_empty());
}
