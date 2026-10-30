//! Achievement rules and the statistics they are measured on. Plain data.

/// What an achievement is about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AchievementScope {
    /// Everything the user does.
    Global,
    /// One stack; unlocks separately for every stack.
    Stack,
}

/// A number computed from the review history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    /// Rehearsal runs that finished (global only).
    RunsFinished,
    /// Ratings given in schedule-updating runs.
    ItemsRated,
    /// Of those, how many were Easy.
    EasyRatings,
    /// Distinct items reviewed at least once.
    ItemsLearned,
    /// Items scheduled at least 21 days ahead.
    ItemsMastered,
    /// Consecutive days with a review, ending today or yesterday.
    StreakDays,
}

/// One achievement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Stable identity (used to remember unlocks).
    pub id: String,
    /// Short name.
    pub title: String,
    /// What it takes.
    pub description: String,
    /// Global or per stack.
    pub scope: AchievementScope,
    /// What is counted.
    pub metric: Metric,
    /// Unlocks when the metric reaches this (at least 1).
    pub threshold: u32,
}

/// The numbers rules are measured on, for everything or for one stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    /// See [`Metric::RunsFinished`].
    pub runs_finished: u32,
    /// See [`Metric::ItemsRated`].
    pub items_rated: u32,
    /// See [`Metric::EasyRatings`].
    pub easy_ratings: u32,
    /// See [`Metric::ItemsLearned`].
    pub items_learned: u32,
    /// See [`Metric::ItemsMastered`].
    pub items_mastered: u32,
    /// See [`Metric::StreakDays`].
    pub streak_days: u32,
}

/// Days of schedule distance from which an item counts as mastered.
pub const MASTERED_DAYS: f32 = 21.0;

/// The value of one metric.
pub fn metric_value(s: &Stats, m: Metric) -> u32 {
    match m {
        Metric::RunsFinished => s.runs_finished,
        Metric::ItemsRated => s.items_rated,
        Metric::EasyRatings => s.easy_ratings,
        Metric::ItemsLearned => s.items_learned,
        Metric::ItemsMastered => s.items_mastered,
        Metric::StreakDays => s.streak_days,
    }
}

/// Persisted name of a metric.
pub fn metric_name(m: Metric) -> &'static str {
    match m {
        Metric::RunsFinished => "runs_finished",
        Metric::ItemsRated => "items_rated",
        Metric::EasyRatings => "easy_ratings",
        Metric::ItemsLearned => "items_learned",
        Metric::ItemsMastered => "items_mastered",
        Metric::StreakDays => "streak_days",
    }
}

/// Parse a metric name.
pub fn metric_from_name(name: &str) -> Option<Metric> {
    [
        Metric::RunsFinished,
        Metric::ItemsRated,
        Metric::EasyRatings,
        Metric::ItemsLearned,
        Metric::ItemsMastered,
        Metric::StreakDays,
    ]
    .into_iter()
    .find(|m| metric_name(*m) == name)
}

/// Persisted name of a scope.
pub fn scope_name(s: AchievementScope) -> &'static str {
    match s {
        AchievementScope::Global => "global",
        AchievementScope::Stack => "stack",
    }
}

/// Parse a scope name.
pub fn scope_from_name(name: &str) -> Option<AchievementScope> {
    match name {
        "global" => Some(AchievementScope::Global),
        "stack" => Some(AchievementScope::Stack),
        _ => None,
    }
}

/// Why a rule cannot be used, in words for the user.
pub fn rule_problem(r: &Rule) -> Option<String> {
    if r.id.trim().is_empty() || r.id.chars().any(char::is_whitespace) {
        return Some("the id must be one word".to_owned());
    }
    if r.title.trim().is_empty() {
        return Some("the title must not be empty".to_owned());
    }
    if r.threshold == 0 {
        return Some("the threshold must be at least 1".to_owned());
    }
    (r.scope == AchievementScope::Stack && r.metric == Metric::RunsFinished)
        .then(|| "runs_finished only exists for global achievements".to_owned())
}
