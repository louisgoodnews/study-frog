//! Progress towards every rule, and which rules just unlocked.

use std::collections::BTreeMap;

use studyfrog_core::achievement::{AchievementScope, Rule, Stats, metric_value};
use studyfrog_core::time::UnixMs;

/// An achievement that was unlocked, remembered so it only unlocks once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unlock {
    /// The rule.
    pub rule: String,
    /// `""` for a global rule, else the stack key.
    pub key: String,
    /// When.
    pub at: UnixMs,
}

/// Where one rule stands, for the whole vault or for one stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Progress {
    /// The rule's id.
    pub rule: String,
    /// `""` for global, else the stack key.
    pub key: String,
    /// Current value of the metric.
    pub value: u32,
    /// What it takes.
    pub threshold: u32,
    /// When it was unlocked, if it was.
    pub unlocked: Option<UnixMs>,
}

/// Whether the metric has reached the threshold.
pub fn reached(p: &Progress) -> bool {
    p.value >= p.threshold
}

/// Progress for every rule: one entry for a global rule, one per known stack for a stack rule.
pub fn progress(
    rules: &[Rule],
    global: &Stats,
    stacks: &BTreeMap<String, Stats>,
    unlocks: &[Unlock],
) -> Vec<Progress> {
    let at = |rule: &str, key: &str| {
        unlocks
            .iter()
            .find(|u| u.rule == rule && u.key == key)
            .map(|u| u.at)
    };
    let entry = |r: &Rule, key: &str, s: &Stats| Progress {
        rule: r.id.clone(),
        key: key.to_owned(),
        value: metric_value(s, r.metric),
        threshold: r.threshold,
        unlocked: at(&r.id, key),
    };
    rules
        .iter()
        .flat_map(|r| match r.scope {
            AchievementScope::Global => vec![entry(r, "", global)],
            AchievementScope::Stack => stacks.iter().map(|(k, s)| entry(r, k, s)).collect(),
        })
        .collect()
}

/// The entries that reached their threshold but are not remembered as unlocked yet.
pub fn new_unlocks(progress: &[Progress]) -> Vec<&Progress> {
    progress
        .iter()
        .filter(|p| reached(p) && p.unlocked.is_none())
        .collect()
}
