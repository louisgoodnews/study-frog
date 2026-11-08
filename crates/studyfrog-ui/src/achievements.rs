//! The achievements screen's update logic. Pure.

use studyfrog_core::achievement::Metric;
use studyfrog_service::achievements::{AchievementReport, Entry};

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// Which achievements the screen lists.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Filter {
    /// Everything.
    #[default]
    All,
    /// The achievements of the whole vault.
    Global,
    /// The achievements of one stack, by its key.
    Stack(String),
}

/// What the user did with the achievements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementsMsg {
    /// List these achievements.
    Filter(Filter),
    /// A view needs the report (the stack page shows a summary); measure it once.
    Wanted,
}

/// Handle the achievements messages.
pub fn update(state: State, msg: AchievementsMsg) -> (State, Vec<Effect>) {
    match msg {
        AchievementsMsg::Filter(achievements_filter) => (
            State {
                achievements_filter,
                ..state
            },
            Vec::new(),
        ),
        AchievementsMsg::Wanted if state.achievements.is_none() && !state.achievements_wanted => (
            State {
                achievements_wanted: true,
                ..state
            },
            vec![Effect::CheckAchievements],
        ),
        AchievementsMsg::Wanted => (state, Vec::new()),
    }
}

/// The entries the filter lets through, in the order of the rules.
pub fn visible<'a>(entries: &'a [Entry], filter: &Filter) -> Vec<&'a Entry> {
    entries
        .iter()
        .filter(|e| match filter {
            Filter::All => true,
            Filter::Global => e.key.is_empty(),
            Filter::Stack(key) => e.key == *key,
        })
        .collect()
}

/// The stacks that have achievements as (key, title), once each, in the order they first appear.
pub fn stacks_of(entries: &[Entry]) -> Vec<(&str, &str)> {
    let mut stacks: Vec<(&str, &str)> = Vec::new();
    for e in entries.iter().filter(|e| !e.key.is_empty()) {
        if !stacks.iter().any(|(key, _)| *key == e.key) {
            stacks.push((&e.key, e.stack.as_deref().unwrap_or(&e.key)));
        }
    }
    stacks
}

/// How many of a stack's achievements are unlocked, and how many it has.
pub fn stack_counts(entries: &[Entry], key: &str) -> (usize, usize) {
    let own: Vec<&Entry> = entries.iter().filter(|e| e.key == key).collect();
    (
        own.iter().filter(|e| e.unlocked.is_some()).count(),
        own.len(),
    )
}

/// The heading for what a rule measures.
pub fn category(metric: Metric) -> &'static str {
    match metric {
        Metric::StreakDays => "Streaks",
        Metric::RunsFinished => "Runs",
        Metric::ItemsRated | Metric::EasyRatings => "Ratings",
        Metric::ItemsLearned => "Cards",
        Metric::ItemsMastered => "Mastery",
    }
}

const CATEGORY_ORDER: [&str; 5] = ["Streaks", "Runs", "Ratings", "Cards", "Mastery"];

/// The entries under their headings: the vault's categories first, then one group per stack. The
/// unlocked ones come first within a group.
pub fn groups<'a>(entries: &[&'a Entry]) -> Vec<(String, Vec<&'a Entry>)> {
    let mut groups: Vec<(String, Vec<&'a Entry>)> = Vec::new();
    for e in entries {
        let label = match &e.stack {
            Some(title) => title.clone(),
            None => category(e.metric).to_owned(),
        };
        match groups.iter_mut().find(|(l, _)| *l == label) {
            Some((_, list)) => list.push(e),
            None => groups.push((label, vec![e])),
        }
    }
    let rank = |label: &str| CATEGORY_ORDER.iter().position(|c| *c == label);
    groups.sort_by_key(|(label, _)| rank(label).unwrap_or(CATEGORY_ORDER.len()));
    for (_, list) in &mut groups {
        list.sort_by_key(|e| e.unlocked.is_none());
    }
    groups
}

/// A message announcing what just unlocked, if anything.
pub fn unlock_text(report: &AchievementReport) -> Option<String> {
    if report.new.is_empty() {
        return None;
    }
    let names: Vec<String> = report
        .new
        .iter()
        .map(|e| match &e.stack {
            Some(stack) => format!("{} ({stack})", e.title),
            None => e.title.clone(),
        })
        .collect();
    Some(format!("Achievement unlocked: {}", names.join(", ")))
}

/// Keep the report; announce new unlocks. A failed check is reported but never interrupts anything.
pub fn checked(state: State, result: Result<AchievementReport, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(report) => {
            let notice = unlock_text(&report)
                .map(|text| Notice {
                    kind: NoticeKind::Info,
                    text,
                })
                .or(state.notice.clone());
            (
                State {
                    achievements: Some(report),
                    notice,
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not check achievements: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}
