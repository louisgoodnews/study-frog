#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Integration tests for calendar views and timetable updates.

use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;
use studyfrog_ui::calendar_mode::CalendarViewMode;
use studyfrog_ui::effect::Effect;
use studyfrog_ui::plan_msg::PlanMsg;
use studyfrog_ui::state::State;
use studyfrog_ui::update::update;

#[test]
fn calendar_view_mode_switches() {
    let state = State::default();
    assert_eq!(state.planner.view_mode, CalendarViewMode::Month);

    let (state, fx) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::SetViewMode(CalendarViewMode::Week)),
    );
    assert_eq!(state.planner.view_mode, CalendarViewMode::Week);
    assert!(fx.is_empty());

    let (state, _) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::SetViewMode(CalendarViewMode::Day)),
    );
    assert_eq!(state.planner.view_mode, CalendarViewMode::Day);

    let (state, _) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::SetViewMode(CalendarViewMode::Timetable)),
    );
    assert_eq!(state.planner.view_mode, CalendarViewMode::Timetable);
}

#[test]
fn calendar_period_shifts() {
    let mut state = State::default();
    state.planner.loaded = true;
    state.planner.selected = Date {
        year: 2026,
        month: 11,
        day: 10,
    };
    state.planner.view_mode = CalendarViewMode::Day;

    let (state, _) = update(state, studyfrog_ui::msg::Msg::Plan(PlanMsg::ShiftPeriod(1)));
    assert_eq!(
        state.planner.selected,
        Date {
            year: 2026,
            month: 11,
            day: 11
        }
    );

    let (mut state, _) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::ShiftPeriod(-2)),
    );
    assert_eq!(
        state.planner.selected,
        Date {
            year: 2026,
            month: 11,
            day: 9
        }
    );

    state.planner.view_mode = CalendarViewMode::Week;
    let (state, _) = update(state, studyfrog_ui::msg::Msg::Plan(PlanMsg::ShiftPeriod(1)));
    assert_eq!(
        state.planner.selected,
        Date {
            year: 2026,
            month: 11,
            day: 16
        }
    );
}

#[test]
fn timetable_form_open_and_save_flow() {
    let state = State::default();
    let entry = TimetableEntry {
        id: Uid(10),
        title: "Microeconomics".to_owned(),
        day_of_week: 0,
        start_minute: 540,
        duration_min: 90,
        location: Some("Audimax 1".to_owned()),
        instructor: Some("Prof. Smith".to_owned()),
        stack_uid: None,
    };

    // Open form
    let (state, _) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::OpenTimetableForm(Some(entry.clone()))),
    );
    assert!(state.planner.timetable_form.is_some());
    let form = state.planner.timetable_form.as_ref().unwrap();
    assert_eq!(form.title, "Microeconomics");

    // Save timetable entry
    let (state, fx) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::SaveTimetable(entry.clone())),
    );
    assert_eq!(fx, vec![Effect::SaveTimetable(entry)]);

    // Delete timetable entry
    let (_, fx_del) = update(
        state,
        studyfrog_ui::msg::Msg::Plan(PlanMsg::DeleteTimetable(Uid(10))),
    );
    assert_eq!(fx_del, vec![Effect::DeleteTimetable(Uid(10))]);
}
