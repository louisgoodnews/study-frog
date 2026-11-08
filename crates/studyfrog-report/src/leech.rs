//! Leeches: items that keep failing or eat time, found by several signals at once (STORY-0104). Pure.

use std::collections::HashMap;

use studyfrog_core::review::Rating;

/// What is known about one scheduled item.
#[derive(Debug, Clone, PartialEq)]
pub struct LeechInput {
    /// The item, as a number.
    pub item: u64,
    /// Its group (stack): time burden is measured against the group's median.
    pub group: u32,
    /// Its ratings, oldest first.
    pub ratings: Vec<Rating>,
    /// Time spent on it in runs, in milliseconds (showing to rating).
    pub total_ms: u64,
    /// Its FSRS stability now, in days.
    pub stability: Option<f32>,
}

/// Where each signal starts to count.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeechRules {
    /// Again ratings after the first review.
    pub min_lapses: u32,
    /// Hard or Again ratings in a row at the end.
    pub min_streak: u32,
    /// Times the group's median time per item.
    pub burden_ratio: f32,
    /// Reviews after which the stability should have grown.
    pub stagnant_reviews: u32,
    /// Stability (days) still under which an item with that many reviews is stuck.
    pub stagnant_stability: f32,
}

/// The defaults of the story: 4 lapses, a streak of 3, 3 times the median time, under 7 days after 6 reviews.
pub fn leech_rules_default() -> LeechRules {
    LeechRules {
        min_lapses: 4,
        min_streak: 3,
        burden_ratio: 3.0,
        stagnant_reviews: 6,
        stagnant_stability: 7.0,
    }
}

/// Why an item is a leech.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeechReason {
    /// Forgotten again and again.
    RepeatedLapses,
    /// Hard or Again several times in a row lately.
    StruggleStreak,
    /// Takes far longer than the other items of its stack.
    HighLatencyBurden,
    /// Many reviews and still a short stability.
    StagnantStability,
}

/// A flagged item.
#[derive(Debug, Clone, PartialEq)]
pub struct LeechItem {
    /// The item.
    pub item: u64,
    /// Its group.
    pub group: u32,
    /// Again ratings after the first review.
    pub lapses: u32,
    /// Hard or Again in a row at the end.
    pub streak: u32,
    /// Ratings.
    pub reviews: u32,
    /// Its time over the group's median time per item.
    pub time_burden: Option<f32>,
    /// Stability per review, in days.
    pub efficiency: Option<f32>,
    /// Every reason that applies, the strongest first.
    pub reasons: Vec<LeechReason>,
    /// How strongly it is a leech (the sum of each signal over its threshold).
    pub score: f32,
}

fn lapses(r: &[Rating]) -> u32 {
    r.iter().skip(1).filter(|x| **x == Rating::Again).count() as u32
}

fn streak(r: &[Rating]) -> u32 {
    r.iter()
        .rev()
        .take_while(|x| matches!(x, Rating::Again | Rating::Hard))
        .count() as u32
}

fn group_medians(items: &[LeechInput]) -> HashMap<u32, u64> {
    let mut by: HashMap<u32, Vec<u64>> = HashMap::new();
    for i in items.iter().filter(|i| i.total_ms > 0) {
        by.entry(i.group).or_default().push(i.total_ms);
    }
    by.into_iter()
        .map(|(g, mut v)| {
            v.sort_unstable();
            (g, v[(v.len() - 1) / 2])
        })
        .collect()
}

fn judge(i: &LeechInput, median: Option<u64>, r: &LeechRules) -> Option<LeechItem> {
    let (lapses, streak, reviews) = (
        lapses(&i.ratings),
        streak(&i.ratings),
        i.ratings.len() as u32,
    );
    let burden = median
        .filter(|m| *m > 0)
        .map(|m| i.total_ms as f32 / m as f32);
    let efficiency = i.stability.map(|s| s / reviews.max(1) as f32);
    let stagnant =
        reviews >= r.stagnant_reviews && i.stability.is_some_and(|s| s < r.stagnant_stability);
    let mut signals: Vec<(f32, LeechReason)> = Vec::new();
    if lapses >= r.min_lapses {
        signals.push((
            lapses as f32 / r.min_lapses as f32,
            LeechReason::RepeatedLapses,
        ));
    }
    if streak >= r.min_streak {
        signals.push((
            streak as f32 / r.min_streak as f32,
            LeechReason::StruggleStreak,
        ));
    }
    if let Some(b) = burden.filter(|b| *b >= r.burden_ratio) {
        signals.push((b / r.burden_ratio, LeechReason::HighLatencyBurden));
    }
    if stagnant {
        signals.push((1.0, LeechReason::StagnantStability));
    }
    if signals.is_empty() {
        return None;
    }
    signals.sort_by(|a, b| b.0.total_cmp(&a.0));
    Some(LeechItem {
        item: i.item,
        group: i.group,
        lapses,
        streak,
        reviews,
        time_burden: burden,
        efficiency,
        score: signals.iter().map(|s| s.0).sum(),
        reasons: signals.into_iter().map(|s| s.1).collect(),
    })
}

/// Every item that meets at least one signal, the strongest leech first (ties by item).
pub fn find_leeches(items: &[LeechInput], rules: &LeechRules) -> Vec<LeechItem> {
    let medians = group_medians(items);
    let mut out: Vec<LeechItem> = items
        .iter()
        .filter_map(|i| judge(i, medians.get(&i.group).copied(), rules))
        .collect();
    out.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.item.cmp(&b.item)));
    out
}
