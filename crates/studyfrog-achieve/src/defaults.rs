//! The achievements that exist when the vault has no `achievements.toml`.

use studyfrog_core::achievement::{AchievementScope, Metric, Rule};

fn rule(
    id: &str,
    title: &str,
    description: &str,
    scope: AchievementScope,
    metric: Metric,
    threshold: u32,
) -> Rule {
    Rule {
        id: id.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        scope,
        metric,
        threshold,
    }
}

/// The built-in set.
pub fn default_rules() -> Vec<Rule> {
    use AchievementScope::{Global, Stack};
    vec![
        rule(
            "first-run",
            "First run",
            "Finish a rehearsal run.",
            Global,
            Metric::RunsFinished,
            1,
        ),
        rule(
            "ten-runs",
            "Regular",
            "Finish 10 rehearsal runs.",
            Global,
            Metric::RunsFinished,
            10,
        ),
        rule(
            "streak-7",
            "A week in a row",
            "Review something on 7 days in a row.",
            Global,
            Metric::StreakDays,
            7,
        ),
        rule(
            "streak-30",
            "A month in a row",
            "Review something on 30 days in a row.",
            Global,
            Metric::StreakDays,
            30,
        ),
        rule(
            "easy-20",
            "Easy does it",
            "Rate 20 items Easy.",
            Global,
            Metric::EasyRatings,
            20,
        ),
        rule(
            "mastered-100",
            "Hundred mastered",
            "Have 100 items scheduled 21 days or more ahead.",
            Global,
            Metric::ItemsMastered,
            100,
        ),
        rule(
            "stack-learned-25",
            "Getting started",
            "Review 25 different items of a stack.",
            Stack,
            Metric::ItemsLearned,
            25,
        ),
        rule(
            "stack-mastered-25",
            "Stack master",
            "Have 25 items of a stack scheduled 21 days or more ahead.",
            Stack,
            Metric::ItemsMastered,
            25,
        ),
    ]
}
