#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Settings files: parsing, validation with line numbers, rendering.

use proptest::prelude::*;
use studyfrog_config::app::{parse_app_settings, render_app_settings};
use studyfrog_config::stack::{parse_stack_settings, render_stack_settings};
use studyfrog_config::vault::{parse_vault_settings, render_vault_settings};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::{
    AppSettings, StackSettings, VaultSettings, vault_settings_default, with_opened_vault,
};

#[test]
fn stack_settings_parse_with_comments_and_partial_keys() {
    let text = "# my stack\nscheduler = \"sm2\"   # simple mode\ndaily_new_limit = 12\n";
    let s = parse_stack_settings(text).unwrap();
    assert_eq!(
        s,
        StackSettings {
            scheduler: Some(Scheduler::Sm2),
            request_retention: None,
            daily_new_limit: Some(12),
            daily_review_limit: None,
            familiarity: None
        }
    );
    assert_eq!(parse_stack_settings("").unwrap(), StackSettings::default());
}

#[test]
fn stack_errors_name_the_problem_and_the_line() {
    let cases = [
        ("scheduler = \"leitner\"\n", Some(1), "scheduler"),
        (
            "daily_new_limit = 5\nrequest_retention = 1.5\n",
            Some(2),
            "request_retention",
        ),
        ("request_retention = 0.0\n", Some(1), "request_retention"),
        ("daily_new_limit = -3\n", Some(1), "daily_new_limit"),
        ("\n\ncolour = \"red\"\n", Some(3), "colour"),
        ("scheduler = \n", Some(1), ""),
    ];
    for (text, line, needle) in cases {
        let err = parse_stack_settings(text).unwrap_err();
        assert_eq!(err.line, line, "{text:?}: {err}");
        assert!(err.message.contains(needle), "{text:?}: {err}");
    }
}

#[test]
fn vault_settings_default_when_empty_and_override_when_given() {
    assert_eq!(parse_vault_settings("").unwrap(), vault_settings_default());
    let text =
        "[srs]\nscheduler = \"sm2\"\nrequest_retention = 0.85\nmaximum_interval_days = 365\n";
    let s = parse_vault_settings(text).unwrap();
    assert_eq!(
        (s.scheduler, s.request_retention, s.maximum_interval_days),
        (Scheduler::Sm2, 0.85, 365)
    );
    assert_eq!(s.weights, None);
}

#[test]
fn vault_weights_must_be_exactly_21_finite_numbers() {
    let good: Vec<String> = (0..21).map(|i| format!("{i}.5")).collect();
    let ok = parse_vault_settings(&format!("[srs]\nweights = [{}]\n", good.join(", "))).unwrap();
    assert_eq!(ok.weights.unwrap()[20], 20.5);
    let short = parse_vault_settings("[srs]\nweights = [1.0, 2.0]\n").unwrap_err();
    assert!(short.message.contains("21"), "{short}");
    assert_eq!(short.line, Some(2));
    assert!(parse_vault_settings("[srs]\nweights = [nan, 1.0]\n").is_err());
    assert!(parse_vault_settings("[srs]\nmaximum_interval_days = 0\n").is_err());
    assert!(
        parse_vault_settings("[other]\nx = 1\n").is_err(),
        "unknown tables are rejected"
    );
}

fn scheduler() -> impl Strategy<Value = Option<Scheduler>> {
    prop::option::of(prop_oneof![Just(Scheduler::Fsrs), Just(Scheduler::Sm2)])
}

fn decimal(max: u32) -> impl Strategy<Value = f32> {
    (1..max).prop_map(|n| n as f32 / 100.0)
}

proptest! {
    #[test]
    fn stack_settings_round_trip(
        sched in scheduler(),
        retention in prop::option::of(decimal(100)),
        new in prop::option::of(any::<u32>()),
        review in prop::option::of(any::<u32>()),
        flag in prop::option::of(any::<bool>()),
    ) {
        let s = StackSettings { scheduler: sched, request_retention: retention, daily_new_limit: new, daily_review_limit: review, familiarity: flag };
        prop_assert_eq!(parse_stack_settings(&render_stack_settings(&s)), Ok(s));
    }

    #[test]
    fn vault_settings_round_trip(
        sched in prop_oneof![Just(Scheduler::Fsrs), Just(Scheduler::Sm2)],
        retention in decimal(100),
        max in 1_u32..100_000,
        strength in decimal(1000),
        threshold in decimal(100),
        weights in prop::option::of(prop::array::uniform21((-5_000_i32..30_000).prop_map(|n| n as f32 / 1000.0))),
    ) {
        let s = VaultSettings { scheduler: sched, request_retention: retention, maximum_interval_days: max, weights, familiarity_strength: strength, familiarity_threshold: threshold };
        prop_assert_eq!(parse_vault_settings(&render_vault_settings(&s)), Ok(s));
    }
}

#[test]
fn app_settings_round_trip_awkward_paths_and_track_recent_vaults() {
    let s = AppSettings {
        vault: Some("/home/me/\"notes\" \\ tab\there \u{e9}".into()),
        recent: vec!["/a".into(), "C:\\vault".into()],
        reading_wpm: Some(215),
        answer_engine: studyfrog_core::settings::AnswerEngine::Words,
        reading: studyfrog_core::settings::ReadingTuning {
            fast_percent: Some(60),
            slow_percent: Some(250),
            min_words: Some(12),
        },
        settings_tab: Some("rehearsal".into()),
        views_untracked: true,
        library_columns: Some("title,status,due".into()),
        calendar_reviews_off: true,
        stack_tabs: vec!["french:list".into(), "a/b:summary".into()],
        tutorial_offered: true,
    };
    assert_eq!(parse_app_settings(&render_app_settings(&s)), Ok(s));
    assert_eq!(parse_app_settings("").unwrap(), AppSettings::default());
    assert!(
        parse_app_settings("theme = \"dark\"\n").is_err(),
        "unknown keys are rejected"
    );

    let first = with_opened_vault(&AppSettings::default(), "/a");
    let second = with_opened_vault(&first, "/b");
    let again = with_opened_vault(&second, "/a");
    assert_eq!(
        (again.vault.as_deref(), again.recent.as_slice()),
        (Some("/a"), ["/a".to_owned(), "/b".to_owned()].as_slice())
    );
    let many = (0..20).fold(AppSettings::default(), |s, i| {
        with_opened_vault(&s, &format!("/v{i}"))
    });
    assert_eq!(many.recent.len(), 8);
    assert_eq!(many.recent[0], "/v19");
}

proptest! {
    #[test]
    fn any_vault_path_round_trips(path in "\\PC{0,40}") {
        let s = AppSettings { vault: Some(path.clone()), recent: vec![path], reading_wpm: None, answer_engine: Default::default(), reading: Default::default(), settings_tab: None, views_untracked: false, library_columns: None, calendar_reviews_off: false, stack_tabs: Vec::new(), tutorial_offered: false, };
        prop_assert_eq!(parse_app_settings(&render_app_settings(&s)), Ok(s));
    }
}

#[test]
fn reading_speed_is_stored_and_must_be_plausible() {
    assert_eq!(
        parse_app_settings("reading_wpm = 215\n")
            .unwrap()
            .reading_wpm,
        Some(215)
    );
    assert_eq!(parse_app_settings("").unwrap().reading_wpm, None);
    for bad in ["reading_wpm = 10\n", "reading_wpm = 5000\n"] {
        let e = parse_app_settings(&format!("vault = \"/v\"\n{bad}")).unwrap_err();
        assert!(
            e.message.contains("between 50 and 1000") && e.line == Some(2),
            "{e:?}"
        );
    }
    assert!(parse_app_settings("reading_wpm = \"fast\"\n").is_err());
}

#[test]
fn answer_engine_and_reading_thresholds_are_read_and_checked() {
    use studyfrog_core::settings::AnswerEngine;
    let ok = parse_app_settings("answer_engine = \"words\"\nreading_fast_percent = 60\nreading_slow_percent = 250\nreading_min_words = 12\n").unwrap();
    assert_eq!(ok.answer_engine, AnswerEngine::Words);
    assert_eq!(
        (
            ok.reading.fast_percent,
            ok.reading.slow_percent,
            ok.reading.min_words
        ),
        (Some(60), Some(250), Some(12))
    );
    assert_eq!(
        parse_app_settings("").unwrap().answer_engine,
        AnswerEngine::Auto
    );
    for (bad, needle) in [
        ("answer_engine = \"magic\"\n", "\"auto\" or \"words\""),
        (
            "reading_fast_percent = 5\n",
            "reading_fast_percent must be between 10 and 100",
        ),
        ("reading_fast_percent = 101\n", "between 10 and 100"),
        (
            "reading_slow_percent = 50\n",
            "reading_slow_percent must be between 100 and 1000",
        ),
        (
            "reading_min_words = 0\n",
            "reading_min_words must be between 1 and 200",
        ),
    ] {
        let e = parse_app_settings(&format!("vault = \"/v\"\n{bad}")).unwrap_err();
        assert!(
            e.message.contains(needle) && e.line == Some(2),
            "{needle}: {e:?}"
        );
    }
    assert!(
        !render_app_settings(&Default::default()).contains("answer_engine"),
        "defaults are not written"
    );
}

#[test]
fn the_familiarity_threshold_is_between_zero_and_one() {
    let ok = parse_vault_settings("[srs]\nfamiliarity_threshold = 0.9\n").unwrap();
    assert!((ok.familiarity_threshold - 0.9).abs() < 1e-6);
    assert_eq!(parse_vault_settings("").unwrap().familiarity_threshold, 1.0);
    assert!(parse_vault_settings("[srs]\nfamiliarity_threshold = 1.5\n").is_err());
}
