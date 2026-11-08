//! The achievements that exist when the vault has no `achievements.toml`. The ids are stable: an
//! unlock is remembered by its rule id, so an id never changes and a tier is only ever added.

use studyfrog_core::achievement::AchievementScope::{Global, Stack};
use studyfrog_core::achievement::{AchievementScope, Metric, Rule};

/// One row of the catalog: id, title, scope, what is measured, and the number it takes.
type Row = (&'static str, &'static str, AchievementScope, Metric, u32);

/// The built-in tiers, grouped by what they measure.
const CATALOG: &[Row] = &[
    (
        "streak-3",
        "Getting into the rhythm",
        Global,
        Metric::StreakDays,
        3,
    ),
    ("streak-7", "A week in a row", Global, Metric::StreakDays, 7),
    (
        "streak-14",
        "Two weeks strong",
        Global,
        Metric::StreakDays,
        14,
    ),
    (
        "streak-30",
        "A month in a row",
        Global,
        Metric::StreakDays,
        30,
    ),
    (
        "streak-60",
        "Two months of habit",
        Global,
        Metric::StreakDays,
        60,
    ),
    ("streak-100", "Centurion", Global, Metric::StreakDays, 100),
    (
        "streak-365",
        "Year of study",
        Global,
        Metric::StreakDays,
        365,
    ),
    ("first-run", "First run", Global, Metric::RunsFinished, 1),
    ("ten-runs", "Regular", Global, Metric::RunsFinished, 10),
    ("runs-25", "Dedicated", Global, Metric::RunsFinished, 25),
    ("runs-50", "Half-century", Global, Metric::RunsFinished, 50),
    (
        "runs-100",
        "Century club",
        Global,
        Metric::RunsFinished,
        100,
    ),
    ("rated-50", "Warming up", Global, Metric::ItemsRated, 50),
    (
        "rated-250",
        "Deep practice",
        Global,
        Metric::ItemsRated,
        250,
    ),
    (
        "rated-1000",
        "Knowledge worker",
        Global,
        Metric::ItemsRated,
        1000,
    ),
    (
        "rated-5000",
        "Study machine",
        Global,
        Metric::ItemsRated,
        5000,
    ),
    ("easy-20", "Easy does it", Global, Metric::EasyRatings, 20),
    (
        "easy-100",
        "Effortless recall",
        Global,
        Metric::EasyRatings,
        100,
    ),
    (
        "mastered-10",
        "First foothold",
        Global,
        Metric::ItemsMastered,
        10,
    ),
    (
        "mastered-50",
        "Growing foundation",
        Global,
        Metric::ItemsMastered,
        50,
    ),
    (
        "mastered-100",
        "Hundred mastered",
        Global,
        Metric::ItemsMastered,
        100,
    ),
    (
        "mastered-250",
        "Scholar",
        Global,
        Metric::ItemsMastered,
        250,
    ),
    (
        "mastered-500",
        "Grandmaster",
        Global,
        Metric::ItemsMastered,
        500,
    ),
    (
        "stack-learned-10",
        "First steps",
        Stack,
        Metric::ItemsLearned,
        10,
    ),
    (
        "stack-learned-25",
        "Getting started",
        Stack,
        Metric::ItemsLearned,
        25,
    ),
    (
        "stack-learned-50",
        "Familiar territory",
        Stack,
        Metric::ItemsLearned,
        50,
    ),
    (
        "stack-learned-100",
        "Comprehensive grasp",
        Stack,
        Metric::ItemsLearned,
        100,
    ),
    (
        "stack-mastered-10",
        "First mastery",
        Stack,
        Metric::ItemsMastered,
        10,
    ),
    (
        "stack-mastered-25",
        "Stack master",
        Stack,
        Metric::ItemsMastered,
        25,
    ),
    (
        "stack-mastered-50",
        "Deep deck fluency",
        Stack,
        Metric::ItemsMastered,
        50,
    ),
    (
        "stack-mastered-100",
        "Stack perfection",
        Stack,
        Metric::ItemsMastered,
        100,
    ),
    (
        "stack-rated-50",
        "Deck dedication",
        Stack,
        Metric::ItemsRated,
        50,
    ),
];

/// What a rule asks for, in a sentence.
fn describe(scope: AchievementScope, metric: Metric, n: u32) -> String {
    let of = if scope == Stack { " of a stack" } else { "" };
    match metric {
        Metric::RunsFinished if n == 1 => "Finish a rehearsal run.".to_owned(),
        Metric::RunsFinished => format!("Finish {n} rehearsal runs."),
        Metric::StreakDays => format!("Review something on {n} days in a row."),
        Metric::ItemsRated if scope == Stack => format!("Rate {n} items in a stack."),
        Metric::ItemsRated => format!("Rate {n} items."),
        Metric::EasyRatings => format!("Rate {n} items Easy."),
        Metric::ItemsLearned => format!("Review {n} different items{of}."),
        Metric::ItemsMastered => {
            format!("Have {n} items{of} scheduled 21 days or more ahead.")
        }
    }
}

fn rule((id, title, scope, metric, threshold): &Row) -> Rule {
    Rule {
        id: (*id).to_owned(),
        title: (*title).to_owned(),
        description: describe(*scope, *metric, *threshold),
        scope: *scope,
        metric: *metric,
        threshold: *threshold,
    }
}

/// The built-in set.
pub fn default_rules() -> Vec<Rule> {
    CATALOG.iter().map(rule).collect()
}
