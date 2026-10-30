//! Achievements: read the history, measure every rule, remember what just unlocked.

use std::fs;

use studyfrog_achieve::defaults::default_rules;
use studyfrog_achieve::evaluate::{Progress, Unlock, new_unlocks, progress};
use studyfrog_achieve::stats::{Fact, Mastered, compute_stats};
use studyfrog_config::achievements::{ACHIEVEMENTS_FILE, parse_achievements};
use studyfrog_core::achievement::Rule;
use studyfrog_core::time::UnixMs;
use studyfrog_md::ulid::format_ulid;
use studyfrog_store::achievements::{
    count_finished_runs, load_mastered, load_review_rows, load_unlocks, save_unlock,
};
use studyfrog_store::overview::load_overview;

use crate::session::{ServiceError, VaultSession};

const MS_PER_DAY: i64 = 86_400_000;

/// One achievement for the vault or for one stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The rule's id.
    pub rule: String,
    /// Short name.
    pub title: String,
    /// What it takes.
    pub description: String,
    /// The stack's title for a stack achievement.
    pub stack: Option<String>,
    /// Current value of the metric.
    pub value: u32,
    /// What it takes.
    pub threshold: u32,
    /// When it was unlocked.
    pub unlocked: Option<UnixMs>,
}

/// Everything the achievements screen needs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AchievementReport {
    /// Every achievement with its progress, in rule order.
    pub entries: Vec<Entry>,
    /// The ones that unlocked during this check.
    pub new: Vec<Entry>,
    /// Why `achievements.toml` was not used (the built-in set is used instead).
    pub rules_error: Option<String>,
}

/// The vault's rules: its `achievements.toml`, else the built-in set.
fn load_rules(session: &VaultSession) -> (Vec<Rule>, Option<String>) {
    let path = session.root.join(ACHIEVEMENTS_FILE);
    match fs::read_to_string(&path) {
        Ok(text) => match parse_achievements(&text) {
            Ok(rules) => (rules, None),
            Err(e) => (
                default_rules(),
                Some(format!(
                    "{ACHIEVEMENTS_FILE} is not used: {}",
                    studyfrog_config::error::describe_config_error(&e)
                )),
            ),
        },
        Err(_) => (default_rules(), None),
    }
}

fn day_of(at_ms: i64, offset_minutes: i32) -> i64 {
    (at_ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY)
}

fn entry(rules: &[Rule], p: &Progress, stacks: &[(String, String)]) -> Entry {
    let rule = rules.iter().find(|r| r.id == p.rule);
    Entry {
        rule: p.rule.clone(),
        title: rule.map_or_else(|| p.rule.clone(), |r| r.title.clone()),
        description: rule.map(|r| r.description.clone()).unwrap_or_default(),
        stack: (!p.key.is_empty()).then(|| {
            stacks
                .iter()
                .find(|(k, _)| *k == p.key)
                .map_or_else(|| p.key.clone(), |(_, t)| t.clone())
        }),
        value: p.value,
        threshold: p.threshold,
        unlocked: p.unlocked,
    }
}

/// Measure every achievement at `now` (`offset_minutes` is the local UTC offset), remember the ones
/// that unlocked, and report all of them. Safe to call as often as wanted: an unlock happens once.
pub fn check_achievements(
    session: &mut VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<AchievementReport, ServiceError> {
    let (rules, rules_error) = load_rules(session);
    let conn = &session.conn;
    let facts: Vec<Fact> = load_review_rows(conn)
        .map_err(ServiceError::Store)?
        .into_iter()
        .map(|r| Fact {
            item: r.item,
            stack: r.stack,
            rating: r.rating,
            day: day_of(r.at_ms, offset_minutes),
        })
        .collect();
    let mastered: Vec<Mastered> = load_mastered(conn)
        .map_err(ServiceError::Store)?
        .into_iter()
        .map(|(item, stack)| Mastered { item, stack })
        .collect();
    let runs = count_finished_runs(conn).map_err(ServiceError::Store)?;
    let titles: Vec<(String, String)> = load_overview(conn)
        .map_err(ServiceError::Store)?
        .stacks
        .iter()
        .map(|s| (format_ulid(s.uid), s.title.clone()))
        .collect();
    let known: Vec<String> = titles.iter().map(|(k, _)| k.clone()).collect();
    let (global, stats) = compute_stats(
        &facts,
        &mastered,
        &known,
        runs,
        day_of(now.0, offset_minutes),
    );
    let unlocks: Vec<Unlock> = load_unlocks(conn)
        .map_err(ServiceError::Store)?
        .into_iter()
        .map(|(rule, key, at)| Unlock {
            rule,
            key,
            at: UnixMs(at),
        })
        .collect();
    let mut progress = progress(&rules, &global, &stats, &unlocks);
    let fresh: Vec<(String, String)> = new_unlocks(&progress)
        .iter()
        .map(|p| (p.rule.clone(), p.key.clone()))
        .collect();
    for (rule, key) in &fresh {
        save_unlock(conn, rule, key, now.0).map_err(ServiceError::Store)?;
    }
    for p in progress
        .iter_mut()
        .filter(|p| fresh.contains(&(p.rule.clone(), p.key.clone())))
    {
        p.unlocked = Some(now);
    }
    let entries: Vec<Entry> = progress.iter().map(|p| entry(&rules, p, &titles)).collect();
    let new = progress
        .iter()
        .zip(&entries)
        .filter(|(p, _)| fresh.contains(&(p.rule.clone(), p.key.clone())))
        .map(|(_, e)| e.clone())
        .collect();
    Ok(AchievementReport {
        entries,
        new,
        rules_error,
    })
}
