#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Homework deadline markers on the calendar (STORY-0166): loaded into the planner, and reloaded
//! with the calendar. Pure update, no I/O.

use studyfrog_calendar::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::msg::Msg;
use studyfrog_ui::plan_msg::PlanMsg;
use studyfrog_ui::state::State;
use studyfrog_ui::update::update;

fn clock() -> LocalTime {
    LocalTime {
        date: Date {
            year: 2026,
            month: 11,
            day: 21,
        },
        minute: 12 * 60,
    }
}

fn item() -> HomeworkItem {
    HomeworkItem {
        id: Uid(1),
        title: "Problem set".into(),
        course: None,
        start: None,
        due: clock(),
        estimate_min: 60,
        priority: HomeworkPriority::Medium,
        status: HomeworkStatus::NotStarted,
        checklist: Vec::new(),
        notes: String::new(),
    }
}

#[test]
fn loaded_homework_is_kept_with_the_clock_it_was_read_at() {
    let (s, fx) = update(
        State::default(),
        Msg::Plan(PlanMsg::HomeworkLoaded(Ok((vec![item()], clock())))),
    );
    assert!(fx.is_empty());
    assert_eq!(s.planner.homework.len(), 1);
    assert_eq!(s.planner.homework_now, Some(clock()));
}

#[test]
fn a_failed_homework_read_is_reported_and_keeps_the_markers_off() {
    let (s, _) = update(
        State::default(),
        Msg::Plan(PlanMsg::HomeworkLoaded(Err("disk gone".into()))),
    );
    assert!(s.planner.homework.is_empty());
    assert_eq!(s.planner.homework_now, None);
    assert!(s.notice.unwrap().text.contains("disk gone"));
}

#[test]
fn moving_the_calendar_asks_for_the_homework_again() {
    let (_, fx) = update(State::default(), Msg::Plan(PlanMsg::Today));
    assert!(fx.contains(&Effect::LoadHomework));
}
