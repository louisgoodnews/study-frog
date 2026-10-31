#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `profiles.toml`.

use studyfrog_config::profiles::{
    DEFAULT_PROFILE, Profiles, name_problem, parse_profiles, profile_slug, render_profiles,
};

#[test]
fn profiles_round_trip_and_default_comes_first() {
    let p = Profiles {
        current: "Exam \"prep\"".into(),
        names: vec![
            DEFAULT_PROFILE.into(),
            "Work".into(),
            "Exam \"prep\"".into(),
        ],
    };
    assert_eq!(parse_profiles(&render_profiles(&p)).unwrap(), p);
    assert_eq!(parse_profiles("").unwrap(), Profiles::default());
}

#[test]
fn mistakes_are_named() {
    assert!(
        parse_profiles("current = \"Ghost\"\n")
            .unwrap_err()
            .message
            .contains("not listed")
    );
    assert!(
        parse_profiles("[[profile]]\nname = \"work\"\n[[profile]]\nname = \"WORK\"\n")
            .unwrap_err()
            .message
            .contains("already")
    );
    assert!(
        parse_profiles("[[profile]]\nname = \"default\"\n").is_err(),
        "the default one exists already"
    );
    assert!(parse_profiles("colour = 1").is_err());
}

#[test]
fn names_are_checked_and_folders_are_safe() {
    let have = vec![DEFAULT_PROFILE.to_owned(), "Work".to_owned()];
    assert!(name_problem("  ", &have).is_some() && name_problem("work", &have).is_some());
    assert!(
        name_problem(&"x".repeat(41), &have).is_some() && name_problem("a\nb", &have).is_some()
    );
    assert_eq!(name_problem("School", &have), None);
    assert_eq!(profile_slug(" Exam prep! "), "exam-prep");
    assert_eq!(profile_slug("../../etc"), "etc", "no path tricks survive");
    assert_eq!(profile_slug("日本"), "日本");
}
