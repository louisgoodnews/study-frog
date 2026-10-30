#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Settings files: parsing, validation with line numbers, rendering.

use proptest::prelude::*;
use studyfrog_config::stack::{parse_stack_settings, render_stack_settings};
use studyfrog_config::vault::{parse_vault_settings, render_vault_settings};
use studyfrog_core::scheduler::Scheduler;
use studyfrog_core::settings::{StackSettings, VaultSettings, vault_settings_default};

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
            daily_review_limit: None
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
    ) {
        let s = StackSettings { scheduler: sched, request_retention: retention, daily_new_limit: new, daily_review_limit: review };
        prop_assert_eq!(parse_stack_settings(&render_stack_settings(&s)), Ok(s));
    }

    #[test]
    fn vault_settings_round_trip(
        sched in prop_oneof![Just(Scheduler::Fsrs), Just(Scheduler::Sm2)],
        retention in decimal(100),
        max in 1_u32..100_000,
        weights in prop::option::of(prop::array::uniform21((-5_000_i32..30_000).prop_map(|n| n as f32 / 1000.0))),
    ) {
        let s = VaultSettings { scheduler: sched, request_retention: retention, maximum_interval_days: max, weights };
        prop_assert_eq!(parse_vault_settings(&render_vault_settings(&s)), Ok(s));
    }
}
