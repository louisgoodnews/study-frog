#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The achievements screen and the moments that check them.

mod common;

use common::views::{assert_all_named, harness, open_model, running_model};
use common::{NOW_NS, sample_vault};
use egui_kittest::kittest::Queryable;
use studyfrog_core::achievement::Metric;
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::achievements::{AchievementReport, Entry};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exec::{execute, worker_new};
use studyfrog_ui::model::apply;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::{NoticeKind, Screen};

fn entry(
    rule: &str,
    title: &str,
    stack: Option<&str>,
    value: u32,
    threshold: u32,
    unlocked: bool,
) -> Entry {
    Entry {
        rule: rule.into(),
        title: title.into(),
        description: format!("Do {title}."),
        stack: stack.map(str::to_owned),
        key: stack
            .map(|_| studyfrog_md::ulid::format_ulid(Uid(1)))
            .unwrap_or_default(),
        metric: match rule {
            "first-run" => Metric::RunsFinished,
            "streak" => Metric::StreakDays,
            _ => Metric::ItemsLearned,
        },
        value,
        threshold,
        unlocked: unlocked.then_some(UnixMs(1_800_000_000_000)),
    }
}

fn report() -> AchievementReport {
    let first = entry("first-run", "First run", None, 1, 1, true);
    AchievementReport {
        entries: vec![
            first.clone(),
            entry("streak", "A week in a row", None, 3, 7, false),
            entry("deck", "Getting started", Some("French"), 10, 25, false),
        ],
        new: vec![first],
        rules_error: None,
    }
}

#[test]
fn opening_the_screen_checks_and_new_unlocks_are_announced() {
    let mut m = open_model();
    let effects = apply(&mut m, Msg::Navigate(Screen::Achievements));
    assert_eq!(effects, [Effect::CheckAchievements]);
    apply(&mut m, Msg::AchievementsChecked(Ok(report())));
    let n = m.state.notice.clone().unwrap();
    assert_eq!(
        (n.kind, n.text.as_str()),
        (NoticeKind::Info, "Achievement unlocked: First run")
    );
    assert_eq!(m.state.achievements.as_ref().unwrap().entries.len(), 3);

    let mut quiet = report();
    quiet.new.clear();
    m.state.notice = None;
    apply(&mut m, Msg::AchievementsChecked(Ok(quiet)));
    assert!(m.state.notice.is_none(), "nothing new, nothing announced");

    apply(&mut m, Msg::AchievementsChecked(Err("disk".into())));
    assert_eq!(m.state.notice.as_ref().unwrap().kind, NoticeKind::Error);
    assert!(m.state.achievements.is_some(), "the last report stays");
}

#[test]
fn a_stack_unlock_names_its_stack() {
    let mut r = report();
    r.new = vec![entry(
        "deck",
        "Getting started",
        Some("French"),
        25,
        25,
        true,
    )];
    assert_eq!(
        studyfrog_ui::achievements::unlock_text(&r).unwrap(),
        "Achievement unlocked: Getting started (French)"
    );
}

#[test]
fn the_end_of_a_run_checks_achievements() {
    let mut m = running_model(RunMode::Flip);
    m.state.now = 3.0;
    let effects = apply(&mut m, Msg::RunAction(Action::Abort));
    assert!(effects.contains(&Effect::CheckAchievements), "{effects:?}");
    let mid = apply(
        &mut running_model(RunMode::Flip),
        Msg::RunAction(Action::Flip),
    );
    assert!(
        !mid.contains(&Effect::CheckAchievements),
        "not during the run"
    );
}

#[test]
fn the_screen_groups_unlocked_and_in_progress_achievements() {
    let mut m = open_model();
    m.state.screen = Screen::Achievements;
    apply(&mut m, Msg::AchievementsChecked(Ok(report())));
    let mut h = harness(m);
    h.run();
    for text in [
        "Runs (1 / 1 unlocked)",
        "Streaks (0 / 1 unlocked)",
        "French (0 / 1 unlocked)",
        "First run - unlocked 2027-01-15",
        "A week in a row",
        "Getting started",
        "3 / 7",
        "10 / 25",
    ] {
        assert!(h.query_by_label(text).is_some(), "missing {text}");
    }
    assert_all_named(&h, "achievements");
}

#[test]
fn the_executor_measures_a_real_vault() {
    let vault = sample_vault("ach");
    let mut worker = worker_new(None);
    execute(
        &mut worker,
        Effect::OpenVault(vault.path.display().to_string()),
        NOW_NS,
    );
    let msgs = execute(&mut worker, Effect::CheckAchievements, NOW_NS);
    let Msg::AchievementsChecked(Ok(r)) = &msgs[0] else {
        panic!("{:?}", msgs[0])
    };
    assert!(
        r.entries
            .iter()
            .any(|e| e.rule == "first-run" && e.value == 0)
    );
    assert!(r.new.is_empty());
}
