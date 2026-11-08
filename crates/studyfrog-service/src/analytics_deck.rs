//! The deck as the simulations see it (STORY-0105, STORY-0106): every card with its FSRS or SM-2 state, the
//! learner's rating mix, how often a day is missed, and how long a review takes in each stack.

use std::collections::HashMap;

use studyfrog_core::memory::{MemState, Sm2State};
use studyfrog_core::review::Rating;
use studyfrog_core::uid::Uid;
use studyfrog_report::latency::percentile;
use studyfrog_report::session_log::SessionSample;
use studyfrog_report::sim_engine::{Pace, RatingMix, SimCard, SimMemory, pace_default};
use studyfrog_store::analytics_rows::{ItemFact, StateRow};
use studyfrog_store::reports::HistoryRow;

const DAY_MS: i64 = 86_400_000;

fn day(ms: i64, offset: i32) -> i64 {
    (ms + i64::from(offset) * 60_000).div_euclid(DAY_MS)
}

/// How the learner rates, from the history; a usual learner without one.
pub fn rating_mix(rows: &[HistoryRow]) -> RatingMix {
    let (mut seen, mut first, mut first_again) = (std::collections::HashSet::new(), 0, 0);
    let (mut recalled, mut hard, mut good) = (0, 0, 0);
    for r in rows {
        if seen.insert(r.item) {
            (first, first_again) = (
                first + 1,
                first_again + u32::from(r.rating == Rating::Again),
            );
            continue;
        }
        recalled += u32::from(r.rating != Rating::Again);
        hard += u32::from(r.rating == Rating::Hard);
        good += u32::from(r.rating == Rating::Good);
    }
    let share = |a: u32, n: u32, fallback: f32| if n > 0 { a as f32 / n as f32 } else { fallback };
    RatingMix {
        first_again: share(first_again, first, 0.2),
        hard: share(hard, recalled, 0.1),
        good: share(good, recalled, 0.8),
    }
}

/// Share of days without a review over the last 60 days (or since the first review), at most one half.
pub fn miss_probability(rows: &[HistoryRow], today: i64, offset: i32) -> f32 {
    let Some(first) = rows.first().map(|r| day(r.at_ms, offset)) else {
        return 0.0;
    };
    let from = first.max(today - 60);
    let span = (today - from).max(0) as usize;
    if span == 0 {
        return 0.0;
    }
    let mut active = vec![false; span];
    for r in rows {
        if let Some(slot) = usize::try_from(day(r.at_ms, offset) - from)
            .ok()
            .and_then(|i| active.get_mut(i))
        {
            *slot = true;
        }
    }
    let missed = active.iter().filter(|a| !**a).count();
    (missed as f32 / span as f32).min(0.5)
}

/// New cards started per day over the last 14 days.
pub fn new_per_day(rows: &[HistoryRow], today: i64, offset: i32) -> f32 {
    let mut seen = std::collections::HashSet::new();
    let started = rows
        .iter()
        .filter(|r| seen.insert(r.item))
        .filter(|r| day(r.at_ms, offset) > today - 14)
        .count();
    started as f32 / 14.0
}

fn median_s(v: &mut [u32]) -> Option<f32> {
    v.sort_unstable();
    percentile(v, 50).map(|ms| ms as f32 / 1000.0)
}

/// The pace of each stack folder from the time items took in runs (showing to rating), by whether it was the
/// item's first review, a review after a lapse, or another one; the vault's pace fills what a stack lacks, and
/// [`pace_default`] what the vault lacks. Index 0 is the vault's.
pub fn paces(
    rows: &[HistoryRow],
    samples: &[SessionSample],
    starts: &HashMap<Uid, i64>,
    folder_of: &HashMap<Uid, String>,
) -> (Vec<Pace>, HashMap<String, u16>) {
    let mut history: HashMap<Uid, Vec<(i64, Rating)>> = HashMap::new();
    for r in rows {
        history.entry(r.item).or_default().push((r.at_ms, r.rating));
    }
    let mut by: HashMap<String, [Vec<u32>; 3]> = HashMap::new();
    for s in samples {
        let Some(at) = starts.get(&s.run).map(|t| t + i64::from(s.shown_t_ms)) else {
            continue;
        };
        let past = history
            .get(&s.item)
            .map_or(&[][..], |h| &h[..h.partition_point(|x| x.0 < at)]);
        let stage = match past.last() {
            None => 0,
            Some((_, Rating::Again)) => 1,
            Some(_) => 2,
        };
        let folder = folder_of.get(&s.item).cloned().unwrap_or_default();
        for key in [String::from("\u{0}vault"), folder] {
            by.entry(key).or_default()[stage].push(s.item_ms);
        }
    }
    let d = pace_default();
    let mut vault = by.remove("\u{0}vault").unwrap_or_default();
    let base = Pace {
        new_s: median_s(&mut vault[0]).unwrap_or(d.new_s),
        learn_s: median_s(&mut vault[1]).unwrap_or(d.learn_s),
        review_s: median_s(&mut vault[2]).unwrap_or(d.review_s),
    };
    let (mut list, mut index) = (vec![base], HashMap::new());
    for (folder, mut v) in by {
        index.insert(folder, list.len() as u16);
        list.push(Pace {
            new_s: median_s(&mut v[0]).unwrap_or(base.new_s),
            learn_s: median_s(&mut v[1]).unwrap_or(base.learn_s),
            review_s: median_s(&mut v[2]).unwrap_or(base.review_s),
        });
    }
    (list, index)
}

/// A card from its stored state; the later of FSRS and SM-2 when both exist.
pub fn card_of(row: &StateRow, offset: i32, relearning: bool, pace: u16) -> SimCard {
    let memory = match (row.fsrs, row.sm2) {
        (Some((stability, difficulty)), _) => SimMemory::Fsrs(MemState {
            stability,
            difficulty,
        }),
        (None, Some((ease, repetitions, interval_days))) => SimMemory::Sm2(Sm2State {
            ease,
            repetitions,
            interval_days,
        }),
        (None, None) => SimMemory::Fsrs(MemState {
            stability: 1.0,
            difficulty: 5.0,
        }),
    };
    SimCard {
        memory: Some(memory),
        last_day: day(row.last_ms, offset),
        due_day: day(row.due_ms, offset),
        relearning,
        sm2: row.sm2.is_some(),
        pace,
    }
}

/// A card never reviewed.
pub fn new_card(sm2: bool, pace: u16) -> SimCard {
    SimCard {
        memory: None,
        last_day: 0,
        due_day: 0,
        relearning: false,
        sm2,
        pace,
    }
}

/// One state per item: the most recently reviewed of its rows.
pub fn latest_per_item(rows: &[StateRow]) -> HashMap<Uid, StateRow> {
    let mut out: HashMap<Uid, StateRow> = HashMap::new();
    for r in rows {
        let keep = out.get(&r.item).is_some_and(|o| o.last_ms >= r.last_ms);
        if !keep {
            out.insert(r.item, *r);
        }
    }
    out
}

/// The folder of every card's stack.
pub fn folders(facts: &[ItemFact]) -> HashMap<Uid, String> {
    facts
        .iter()
        .map(|f| (f.uid, f.stack_path.clone().unwrap_or_default()))
        .collect()
}
