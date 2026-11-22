//! The plans of the calendar commands. The screen is opened first so its data is read; then the view,
//! the month and the selected day are set, or the reminders are read without marking them shown.

use studyfrog_core::civil::Date;

use crate::control_cmd::calendar::CalendarCmd;
use crate::control_plan::Plan;
use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::state::{Screen, State};

/// The steps of a `calendar` command.
pub fn plan(state: &State, cmd: &CalendarCmd) -> Result<Plan, String> {
    let open = vec![Msg::Navigate(Screen::Calendar)];
    Ok(match cmd {
        CalendarCmd::View(mode) => vec![open, vec![Msg::Plan(PlanMsg::SetViewMode(*mode))]],
        CalendarCmd::Period(day) => period(state, *day, open),
        CalendarCmd::Homework(day) => vec![open, vec![Msg::Plan(PlanMsg::Select(*day))]],
        CalendarCmd::RemindersDue => vec![vec![Msg::Plan(PlanMsg::PeekDue)]],
    })
}

/// Move the visible month to the one of `day` (by the months between them), then select the day.
fn period(state: &State, day: Date, open: Vec<Msg>) -> Plan {
    let (year, month) = state.planner.month;
    let shown = year * 12 + i32::from(month) - 1;
    let wanted = day.year * 12 + i32::from(day.month) - 1;
    let mut phases = vec![open];
    if wanted != shown {
        phases.push(vec![Msg::Plan(PlanMsg::Month(i64::from(wanted - shown)))]);
    }
    phases.push(vec![Msg::Plan(PlanMsg::Select(day))]);
    phases
}
