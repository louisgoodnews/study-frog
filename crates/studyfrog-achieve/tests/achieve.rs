#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Achievement statistics and unlocking.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use studyfrog_achieve::defaults::default_rules;
use studyfrog_achieve::evaluate::{Unlock, new_unlocks, progress};
use studyfrog_achieve::stats::{Fact, Mastered, compute_stats, streak_days};
use studyfrog_core::achievement::{AchievementScope, Metric, Rule, rule_problem};
use studyfrog_core::review::Rating;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;

fn fact(item: u128, stack: Option<&str>, rating: Rating, day: i64) -> Fact {
    Fact {
        item: Uid(item),
        stack: stack.map(str::to_owned),
        rating,
        day,
    }
}

fn days(list: &[i64]) -> BTreeSet<i64> {
    list.iter().copied().collect()
}

#[test]
fn a_streak_counts_back_from_today_or_yesterday() {
    assert_eq!(streak_days(&days(&[]), 10), 0);
    assert_eq!(streak_days(&days(&[8, 9, 10]), 10), 3);
    assert_eq!(
        streak_days(&days(&[7, 8, 9]), 10),
        3,
        "yesterday still counts: today is not over"
    );
    assert_eq!(
        streak_days(&days(&[6, 7, 8]), 10),
        0,
        "two days without a review break it"
    );
    assert_eq!(
        streak_days(&days(&[5, 6, 8, 9, 10]), 10),
        3,
        "a gap ends the count"
    );
    assert_eq!(streak_days(&days(&[10]), 10), 1);
}

#[test]
fn statistics_are_kept_for_everything_and_for_each_stack() {
    let facts = [
        fact(1, Some("a"), Rating::Easy, 10),
        fact(1, Some("a"), Rating::Good, 11),
        fact(2, Some("a"), Rating::Easy, 11),
        fact(3, Some("b"), Rating::Good, 11),
        fact(4, None, Rating::Easy, 11),
    ];
    let mastered = [
        Mastered {
            item: Uid(1),
            stack: Some("a".into()),
        },
        Mastered {
            item: Uid(3),
            stack: Some("b".into()),
        },
        Mastered {
            item: Uid(9),
            stack: None,
        },
    ];
    let (g, stacks) = compute_stats(&facts, &mastered, &[], 4, 11);
    assert_eq!(
        (
            g.runs_finished,
            g.items_rated,
            g.easy_ratings,
            g.items_learned
        ),
        (4, 5, 3, 4)
    );
    assert_eq!((g.items_mastered, g.streak_days), (3, 2));
    let a = stacks["a"];
    assert_eq!(
        (
            a.items_rated,
            a.easy_ratings,
            a.items_learned,
            a.items_mastered,
            a.streak_days
        ),
        (3, 2, 2, 1, 2)
    );
    assert_eq!(stacks["b"].items_learned, 1);
    assert_eq!(
        stacks.len(),
        2,
        "items outside any stack only count globally"
    );
}

#[test]
fn a_stack_without_history_still_shows_its_goals() {
    let (_, stacks) = compute_stats(&[], &[], &["empty".to_owned()], 0, 5);
    assert_eq!(
        stacks["empty"],
        studyfrog_core::achievement::Stats::default()
    );
}

#[test]
fn a_rule_unlocks_once_and_stack_rules_unlock_per_stack() {
    let rules = vec![
        Rule {
            id: "easy-2".into(),
            title: "T".into(),
            description: String::new(),
            scope: AchievementScope::Global,
            metric: Metric::EasyRatings,
            threshold: 2,
        },
        Rule {
            id: "learned-2".into(),
            title: "S".into(),
            description: String::new(),
            scope: AchievementScope::Stack,
            metric: Metric::ItemsLearned,
            threshold: 2,
        },
    ];
    let facts = [
        fact(1, Some("a"), Rating::Easy, 1),
        fact(2, Some("a"), Rating::Easy, 1),
        fact(3, Some("b"), Rating::Good, 1),
    ];
    let (g, stacks) = compute_stats(&facts, &[], &[], 0, 1);
    let p = progress(&rules, &g, &stacks, &[]);
    let fresh: Vec<(&str, &str)> = new_unlocks(&p)
        .iter()
        .map(|x| (x.rule.as_str(), x.key.as_str()))
        .collect();
    assert_eq!(
        fresh,
        [("easy-2", ""), ("learned-2", "a")],
        "stack b has only one item"
    );

    let known = [Unlock {
        rule: "easy-2".into(),
        key: String::new(),
        at: UnixMs(5),
    }];
    let p = progress(&rules, &g, &stacks, &known);
    assert_eq!(
        new_unlocks(&p).len(),
        1,
        "already unlocked rules do not unlock again"
    );
    assert_eq!(p[0].unlocked, Some(UnixMs(5)));
}

#[test]
fn the_built_in_achievements_are_valid_and_unique() {
    let rules = default_rules();
    assert!(rules.len() >= 6);
    let ids: BTreeSet<&str> = rules.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids.len(), rules.len());
    assert!(rules.iter().all(|r| rule_problem(r).is_none()));
    assert!(
        rules
            .iter()
            .any(|r| r.metric == Metric::StreakDays && r.threshold == 7)
    );
    assert!(
        rules
            .iter()
            .any(|r| r.metric == Metric::ItemsMastered && r.threshold == 100)
    );
}

#[test]
fn unusable_rules_are_explained() {
    let base = default_rules().remove(0);
    let bad = |f: fn(&mut Rule)| {
        let mut r = base.clone();
        f(&mut r);
        rule_problem(&r).unwrap()
    };
    assert!(bad(|r| r.threshold = 0).contains("at least 1"));
    assert!(bad(|r| r.title = " ".into()).contains("title"));
    assert!(bad(|r| r.id = "two words".into()).contains("id"));
    assert!(bad(|r| r.scope = AchievementScope::Stack).contains("global"));
}

proptest! {
    #[test]
    fn a_streak_never_exceeds_the_days_with_reviews(list in prop::collection::vec(0_i64..40, 0..40), today in 0_i64..45) {
        let d: BTreeSet<i64> = list.into_iter().collect();
        prop_assert!(streak_days(&d, today) as usize <= d.len());
    }

    #[test]
    fn stack_statistics_add_up_to_the_global_ones(
        raw in prop::collection::vec((0_u128..12, 0_u8..3, 1_u8..5, 0_i64..20), 0..60)
    ) {
        let facts: Vec<Fact> = raw.iter().map(|(i, s, r, d)| Fact {
            item: Uid(*i),
            stack: (*s < 2).then(|| format!("s{s}")),
            rating: studyfrog_core::review::rating_from_value(*r).unwrap(),
            day: *d,
        }).collect();
        let (g, stacks) = compute_stats(&facts, &[], &[], 0, 20);
        let in_stacks: u32 = stacks.values().map(|s| s.items_rated).sum();
        let outside = facts.iter().filter(|f| f.stack.is_none()).count() as u32;
        prop_assert_eq!(g.items_rated, in_stacks + outside);
        let per: BTreeMap<_, _> = stacks.iter().map(|(k, s)| (k.clone(), s.easy_ratings)).collect();
        prop_assert!(per.values().sum::<u32>() <= g.easy_ratings);
    }
}
