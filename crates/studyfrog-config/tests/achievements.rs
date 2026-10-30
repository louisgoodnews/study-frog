#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `achievements.toml`.

#[test]
fn achievements_are_parsed_and_checked() {
    use studyfrog_config::achievements::parse_achievements;
    use studyfrog_core::achievement::{AchievementScope, Metric};
    let text = "[[achievement]]\nid = \"week\"\ntitle = \"Week\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 7\n\n\
                [[achievement]]\nid = \"deck\"\ntitle = \"Deck\"\ndescription = \"x\"\nscope = \"stack\"\nmetric = \"items_learned\"\nthreshold = 10\n";
    let rules = parse_achievements(text).unwrap();
    assert_eq!(
        (rules[0].metric, rules[0].scope, rules[0].threshold),
        (Metric::StreakDays, AchievementScope::Global, 7)
    );
    assert_eq!(
        (rules[1].description.as_str(), rules[1].scope),
        ("x", AchievementScope::Stack)
    );
    assert!(
        parse_achievements("").unwrap().is_empty(),
        "an empty file means no achievements"
    );
    for (bad, needle) in [
        (
            "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"galaxy\"\nmetric = \"streak_days\"\nthreshold = 1\n",
            "scope must be",
        ),
        (
            "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"global\"\nmetric = \"vibes\"\nthreshold = 1\n",
            "unknown metric",
        ),
        (
            "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 0\n",
            "at least 1",
        ),
        (
            "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"stack\"\nmetric = \"runs_finished\"\nthreshold = 1\n",
            "global",
        ),
        (
            "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 1\ncolour = \"red\"\n",
            "colour",
        ),
    ] {
        let e = parse_achievements(bad).unwrap_err();
        assert!(e.message.contains(needle), "{needle}: {e:?}");
    }
    let twice = "[[achievement]]\nid = \"a\"\ntitle = \"A\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 1\n\n[[achievement]]\nid = \"a\"\ntitle = \"B\"\nscope = \"global\"\nmetric = \"streak_days\"\nthreshold = 2\n";
    assert!(
        parse_achievements(twice)
            .unwrap_err()
            .message
            .contains("used twice")
    );
}
