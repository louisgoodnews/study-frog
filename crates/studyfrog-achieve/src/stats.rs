//! From review facts to the numbers rules are measured on.

use std::collections::{BTreeMap, BTreeSet};

use studyfrog_core::achievement::Stats;
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;

/// One review, reduced to what achievements need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fact {
    /// The item.
    pub item: Uid,
    /// The key of its stack (`None` outside every stack).
    pub stack: Option<String>,
    /// The rating given.
    pub rating: Rating,
    /// The local day of the review, in days since 1970-01-01.
    pub day: i64,
}

/// An item scheduled far enough ahead to count as mastered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mastered {
    /// The item.
    pub item: Uid,
    /// The key of its stack.
    pub stack: Option<String>,
}

/// Consecutive days with a review, counting back from `today`; a streak that ended yesterday still
/// counts (today is not over). `days` may be in any order and contain duplicates.
pub fn streak_days(days: &BTreeSet<i64>, today: i64) -> u32 {
    let mut day = if days.contains(&today) {
        today
    } else {
        today - 1
    };
    let mut n = 0;
    while days.contains(&day) {
        n += 1;
        day -= 1;
    }
    n
}

fn stats_of<'a>(facts: impl Iterator<Item = &'a Fact>, mastered: u32, today: i64) -> Stats {
    let (mut s, mut items, mut days) = (Stats::default(), BTreeSet::new(), BTreeSet::new());
    for f in facts {
        s.items_rated += 1;
        s.easy_ratings += u32::from(f.rating == Rating::Easy);
        items.insert(f.item);
        days.insert(f.day);
    }
    s.items_learned = items.len() as u32;
    s.items_mastered = mastered;
    s.streak_days = streak_days(&days, today);
    s
}

/// The statistics for everything and for each stack that exists or has history. `known_stacks` are
/// the keys of the stacks in the vault, so a stack without any history still shows its goals.
pub fn compute_stats(
    facts: &[Fact],
    mastered: &[Mastered],
    known_stacks: &[String],
    runs_finished: u32,
    today: i64,
) -> (Stats, BTreeMap<String, Stats>) {
    let global = Stats {
        runs_finished,
        ..stats_of(facts.iter(), mastered.len() as u32, today)
    };
    let mut keys: BTreeSet<&str> = facts.iter().filter_map(|f| f.stack.as_deref()).collect();
    keys.extend(mastered.iter().filter_map(|m| m.stack.as_deref()));
    keys.extend(known_stacks.iter().map(String::as_str));
    let per_stack = keys
        .into_iter()
        .map(|key| {
            let count = mastered
                .iter()
                .filter(|m| m.stack.as_deref() == Some(key))
                .count();
            let own = facts.iter().filter(|f| f.stack.as_deref() == Some(key));
            (key.to_owned(), stats_of(own, count as u32, today))
        })
        .collect();
    (global, per_stack)
}
