#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The achievements filter, their grouping, and the summary on a stack's page.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_core::achievement::Metric;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::achievements::{AchievementReport, Entry};
use studyfrog_ui::achievements::{
    AchievementsMsg, Filter, groups, stack_counts, stacks_of, visible,
};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{Screen, Selection};

fn entry(rule: &str, metric: Metric, stack: Option<(&str, Uid)>, unlocked: bool) -> Entry {
    Entry {
        rule: rule.into(),
        title: format!("Title of {rule}"),
        description: format!("Do {rule}."),
        stack: stack.map(|(title, _)| title.to_owned()),
        key: stack.map(|(_, uid)| format_ulid(uid)).unwrap_or_default(),
        metric,
        value: 1,
        threshold: 5,
        unlocked: unlocked.then_some(UnixMs(1_800_000_000_000)),
    }
}

fn entries() -> Vec<Entry> {
    let (french, maths) = (("French", Uid(1)), ("Maths", Uid(2)));
    vec![
        entry("streak-3", Metric::StreakDays, None, true),
        entry("rated-50", Metric::ItemsRated, None, false),
        entry("first-run", Metric::RunsFinished, None, true),
        entry("fr-learned", Metric::ItemsLearned, Some(french), true),
        entry("fr-mastered", Metric::ItemsMastered, Some(french), false),
        entry("ma-learned", Metric::ItemsLearned, Some(maths), false),
    ]
}

fn loaded() -> Model {
    let mut m = open_model();
    m.state.screen = Screen::Achievements;
    let report = AchievementReport {
        entries: entries(),
        ..AchievementReport::default()
    };
    apply(&mut m, Msg::AchievementsChecked(Ok(report)));
    m
}

#[test]
fn the_filter_lets_through_everything_the_vaults_own_or_one_stack() {
    let all = entries();
    assert_eq!(visible(&all, &Filter::All).len(), 6);
    assert_eq!(visible(&all, &Filter::Global).len(), 3);
    let french = visible(&all, &Filter::Stack(format_ulid(Uid(1))));
    assert_eq!(french.len(), 2);
    assert!(french.iter().all(|e| e.stack.as_deref() == Some("French")));
    assert!(visible(&all, &Filter::Stack("nobody".into())).is_empty());
    assert_eq!(
        stacks_of(&all),
        [
            (format_ulid(Uid(1)).as_str(), "French"),
            (format_ulid(Uid(2)).as_str(), "Maths")
        ]
    );
}

#[test]
fn entries_are_grouped_by_category_then_by_stack_with_unlocked_ones_first() {
    let all = entries();
    let shown = visible(&all, &Filter::All);
    let labels: Vec<String> = groups(&shown).into_iter().map(|(l, _)| l).collect();
    assert_eq!(labels, ["Streaks", "Runs", "Ratings", "French", "Maths"]);
    let french = &groups(&shown)[3].1;
    assert_eq!(french[0].rule, "fr-learned", "the unlocked one comes first");
    assert_eq!(stack_counts(&all, &format_ulid(Uid(1))), (1, 2));
    assert_eq!(stack_counts(&all, &format_ulid(Uid(2))), (0, 1));
    assert_eq!(stack_counts(&all, "nobody"), (0, 0));
}

#[test]
fn the_pills_set_the_filter_and_the_list_follows_it() {
    let mut h = harness(loaded());
    h.run();
    for pill in ["All", "Global", "French", "Maths"] {
        assert!(
            h.query_all_by_label(pill).count() >= 1,
            "missing pill {pill}"
        );
    }
    assert!(h.query_by_label("French (1 / 2 unlocked)").is_some());
    h.get_by_label("Global").click();
    h.run_steps(3);
    assert_eq!(h.state().0.state.achievements_filter, Filter::Global);
    assert!(
        h.query_by_label("French (1 / 2 unlocked)").is_none(),
        "stack groups are hidden"
    );
    assert!(h.query_by_label("Streaks (1 / 1 unlocked)").is_some());
    h.get_by_label("Maths").click();
    h.run_steps(3);
    assert_eq!(
        h.state().0.state.achievements_filter,
        Filter::Stack(format_ulid(Uid(2)))
    );
    assert!(h.query_by_label("Maths (0 / 1 unlocked)").is_some());
    assert_all_named(&h, "achievements filter");
}

#[test]
fn a_stack_page_shows_its_summary_and_the_link_opens_its_achievements() {
    let mut m = loaded();
    apply(&mut m, Msg::Navigate(Screen::Library));
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let mut h = harness(m);
    h.run_steps(4);
    h.get_by_label("1 / 2 achievements unlocked").click();
    h.run_steps(4);
    let model = &h.state().0;
    assert_eq!(model.state.screen, Screen::Achievements);
    assert_eq!(
        model.state.achievements_filter,
        Filter::Stack(format_ulid(Uid(1)))
    );
}

#[test]
fn the_report_is_asked_for_once_when_a_stack_page_needs_it() {
    let mut m = open_model();
    apply(&mut m, Msg::Select(Selection::Stack("french".into())));
    let first = apply(&mut m, Msg::Achievements(AchievementsMsg::Wanted));
    assert_eq!(first, [Effect::CheckAchievements]);
    let second = apply(&mut m, Msg::Achievements(AchievementsMsg::Wanted));
    assert!(second.is_empty(), "asked for once");
    let mut h = harness(m);
    h.run_steps(4);
    assert!(
        h.state().1.iter().all(|e| *e != Effect::CheckAchievements),
        "the page does not ask again while the report is on its way"
    );
}
