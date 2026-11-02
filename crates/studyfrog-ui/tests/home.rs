#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The home screen: its update logic and its view, headless.

mod common;

use common::views::{assert_all_named, harness, open_model};
use egui_kittest::kittest::Queryable;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::plan::ItemKind;
use studyfrog_core::uid::Uid;
use studyfrog_service::home::{HomeSummary, StackDue, TodaySession, Upcoming};
use studyfrog_store::home::RunRow;
use studyfrog_store::overview::ItemRow;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::home_ui::HomeMsg;
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;
use studyfrog_ui::state::Screen;

const DAY: i64 = 86_400_000;

fn summary(ready: usize) -> HomeSummary {
    HomeSummary {
        now_ms: 10 * DAY,
        today: studyfrog_core::civil::date_new(2026, 10, 30).unwrap(),
        hour: 9,
        ready_today: ready,
        due_now: ready.saturating_sub(1),
        new_items: usize::from(ready > 0),
        reviews_today: 4,
        streak_days: 3,
        stacks: vec![StackDue {
            uid: Uid(5),
            folder: "french".into(),
            title: "French".into(),
            due: 2,
            new: 1,
        }],
        attention: Vec::new(),
        recent_items: vec![ItemRow {
            uid: Uid(7),
            title: "Bonjour".into(),
            kind: ItemKind::Card,
            path: "french/a.md".into(),
            due: None,
            status: studyfrog_core::status::Status::New,
        }],
        recent_runs: vec![RunRow {
            uid: Uid(9),
            mode: "flip".into(),
            state: "finished".into(),
            started_ms: 8 * DAY,
            rated: 5,
            skipped: 1,
        }],
        due_items: vec![ItemRow {
            uid: Uid(8),
            title: "Merci".into(),
            kind: ItemKind::Card,
            path: "french/b.md".into(),
            due: Some(studyfrog_core::time::UnixMs(7 * DAY)),
            status: studyfrog_core::status::Status::New,
        }],
        upcoming: vec![Upcoming {
            title: "Final exam".into(),
            kind: EventKind::Exam,
            date: studyfrog_core::civil::date_new(2026, 11, 3).unwrap(),
            start_minute: 600,
        }],
        sessions_today: vec![TodaySession {
            title: "Vocabulary".into(),
            start_minute: 600,
            duration_min: 30,
            stack: Some("French".into()),
        }],
    }
}

fn home_model(ready: usize) -> Model {
    let mut m = open_model();
    apply(&mut m, Msg::Navigate(Screen::Home));
    apply(
        &mut m,
        Msg::Home(HomeMsg::Loaded(Ok(Box::new(summary(ready))))),
    );
    m
}

#[test]
fn going_home_asks_for_the_summary_and_a_failure_keeps_the_last_numbers() {
    let mut m = open_model();
    let fx = apply(&mut m, Msg::Navigate(Screen::Home));
    assert!(fx.contains(&Effect::LoadHome));
    apply(&mut m, Msg::Home(HomeMsg::Loaded(Ok(Box::new(summary(3))))));
    apply(&mut m, Msg::Home(HomeMsg::Loaded(Err("disk".into()))));
    assert_eq!(m.state.home.as_ref().unwrap().ready_today, 3);
    assert!(m.state.notice.unwrap().text.contains("disk"));
}

#[test]
fn the_home_view_shows_what_is_due_and_what_happened_recently() {
    let mut h = harness(home_model(3));
    h.run();
    for text in [
        "Good morning",
        "Start today's rehearsal",
        "items ready for today",
        "Reviews today",
        "Rehearse French",
        "Bonjour",
        "Merci",
    ] {
        assert!(h.query_all_by_label(text).count() >= 1, "missing {text:?}");
    }
    assert!(h.query_by_label_contains("2 days ago").is_some());
    assert!(h.query_by_label_contains("3 days overdue").is_some());
    assert!(
        h.query_by_label_contains("2026-11-03 10:00  Final exam")
            .is_some()
    );
    assert!(h.query_by_label_contains("10:00–10:30").is_some());
    assert_all_named(&h, "home");
}

#[test]
fn nothing_ready_offers_to_practise_instead() {
    let mut h = harness(home_model(0));
    h.run();
    assert!(h.query_by_label("Start today's rehearsal").is_none());
    h.get_by_label("Practice something else").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Rehearse);
}

#[test]
fn the_buttons_start_a_run_or_open_the_setup_for_one_stack() {
    let mut h = harness(home_model(3));
    h.run();
    h.get_by_label("Rehearse French").click();
    h.run();
    assert_eq!(h.state().0.state.screen, Screen::Rehearse);

    let mut h = harness(home_model(3));
    h.run();
    h.get_by_label("Bonjour").click();
    h.run();
    assert!(h.state().1.contains(&Effect::LoadDocument(Uid(7))));
}

#[test]
fn the_needs_attention_card_lists_the_items_with_their_reasons_and_opens_one() {
    let mut m = home_model(3);
    m.state.home.as_mut().unwrap().attention = vec![studyfrog_service::attention::Attention {
        uid: Uid(33),
        title: "Subjonctif".into(),
        percent: 12,
        reasons: vec!["Familiarity 12 %".into(), "Exam in 5 day(s)".into()],
    }];
    let mut h = harness(m);
    h.run_steps(4);
    assert!(h.query_by_label("Needs attention").is_some());
    assert!(h.query_by_label_contains("Exam in 5 day(s)").is_some());
    assert_all_named(&h, "home with attention");
    h.get_by_label("Subjonctif").click();
    h.run_steps(4);
    assert!(
        h.state().1.contains(&Effect::LoadDocument(Uid(33))),
        "{:?}",
        h.state().1
    );

    let mut none = home_model(3);
    none.state.home.as_mut().unwrap().attention.clear();
    let mut h = harness(none);
    h.run_steps(4);
    assert!(
        h.query_by_label("Needs attention").is_none(),
        "no card when nothing needs it"
    );
}
