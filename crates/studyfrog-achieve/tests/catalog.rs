#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The built-in achievement tiers: what they are, that old ids keep their meaning, and that the
//! evaluator unlocks new tiers from an existing history without touching earlier unlocks.

use std::collections::{BTreeMap, BTreeSet};

use studyfrog_achieve::defaults::default_rules;
use studyfrog_achieve::evaluate::{Unlock, new_unlocks, progress};
use studyfrog_core::achievement::AchievementScope::{Global, Stack};
use studyfrog_core::achievement::{Metric, Stats};
use studyfrog_core::time::UnixMs;

fn rule_of(id: &str) -> (Metric, u32) {
    let all = default_rules();
    let r = all.iter().find(|r| r.id == id).expect(id);
    (r.metric, r.threshold)
}

#[test]
fn the_catalog_has_every_tier_of_the_story_with_unique_ids() {
    let rules = default_rules();
    assert_eq!(rules.len(), 32);
    let ids: BTreeSet<_> = rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), rules.len(), "ids are unique");
    let count = |metric, scope| {
        rules
            .iter()
            .filter(|r| r.metric == metric && r.scope == scope)
            .count()
    };
    assert_eq!(count(Metric::StreakDays, Global), 7);
    assert_eq!(count(Metric::RunsFinished, Global), 5);
    assert_eq!(count(Metric::ItemsRated, Global), 4);
    assert_eq!(count(Metric::EasyRatings, Global), 2);
    assert_eq!(count(Metric::ItemsMastered, Global), 5);
    assert_eq!(count(Metric::ItemsLearned, Stack), 4);
    assert_eq!(count(Metric::ItemsMastered, Stack), 4);
    assert_eq!(count(Metric::ItemsRated, Stack), 1);
    assert_eq!(rule_of("streak-365"), (Metric::StreakDays, 365));
    assert_eq!(rule_of("rated-5000"), (Metric::ItemsRated, 5000));
    assert_eq!(rule_of("stack-mastered-100"), (Metric::ItemsMastered, 100));
}

#[test]
fn the_ids_of_the_first_set_keep_their_meaning() {
    for (id, metric, n) in [
        ("first-run", Metric::RunsFinished, 1),
        ("ten-runs", Metric::RunsFinished, 10),
        ("streak-7", Metric::StreakDays, 7),
        ("streak-30", Metric::StreakDays, 30),
        ("easy-20", Metric::EasyRatings, 20),
        ("mastered-100", Metric::ItemsMastered, 100),
        ("stack-learned-25", Metric::ItemsLearned, 25),
        ("stack-mastered-25", Metric::ItemsMastered, 25),
    ] {
        assert_eq!(rule_of(id), (metric, n), "{id}");
    }
}

#[test]
fn tiers_of_one_measure_rise_and_every_rule_says_what_it_takes() {
    let mut last: BTreeMap<(String, bool), u32> = BTreeMap::new();
    for r in default_rules() {
        let key = (format!("{:?}", r.metric), r.scope == Stack);
        let before = last.insert(key, r.threshold).unwrap_or(0);
        assert!(r.threshold > before, "{} does not rise", r.id);
        assert!(
            r.description.ends_with('.') && !r.title.is_empty(),
            "{}",
            r.id
        );
    }
}

#[test]
fn a_history_unlocks_exactly_the_tiers_it_reached_and_earlier_unlocks_stay() {
    let rules = default_rules();
    let global = Stats {
        streak_days: 14,
        runs_finished: 26,
        ..Stats::default()
    };
    let remembered = [Unlock {
        rule: "streak-7".into(),
        key: String::new(),
        at: UnixMs(1),
    }];
    let p = progress(&rules, &global, &BTreeMap::new(), &remembered);
    let fresh: BTreeSet<_> = new_unlocks(&p).iter().map(|p| p.rule.as_str()).collect();
    let want = ["streak-3", "streak-14", "first-run", "ten-runs", "runs-25"];
    assert_eq!(fresh, BTreeSet::from(want), "streak-7 was already unlocked");
    let seven = p.iter().find(|p| p.rule == "streak-7").unwrap();
    assert_eq!(
        seven.unlocked,
        Some(UnixMs(1)),
        "the earlier unlock is kept as it was"
    );
}
