//! The plans of the calendar commands. The screen is opened first so its data is read; then the view,
//! the month and the selected day are set, or the reminders are read without marking them shown. A new
//! session is made the way a user makes one: the planner's New form, its fields, then Save.

use studyfrog_core::civil::{Date, format_date, format_minute};

use crate::control_cmd::calendar::{CalendarCmd, SessionAdd};
use crate::control_plan::Plan;
use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::session_form::FormEdit;
use crate::state::Screen;

/// The steps of a `calendar` command.
pub fn plan(cmd: &CalendarCmd) -> Result<Plan, String> {
    let open = vec![Msg::Navigate(Screen::Calendar)];
    Ok(match cmd {
        CalendarCmd::Show => vec![open],
        CalendarCmd::View(mode) => vec![open, vec![Msg::Plan(PlanMsg::SetViewMode(*mode))]],
        CalendarCmd::Period(day) => period(*day, open),
        CalendarCmd::Homework(day) => vec![open, vec![Msg::Plan(PlanMsg::Select(*day))]],
        CalendarCmd::SessionAdd(s) => session_add(s, open),
        CalendarCmd::RemindersDue => vec![vec![Msg::Plan(PlanMsg::PeekDue)]],
    })
}

/// Show the start's month and day, open the new-session form on it, fill it and save it.
fn session_add(s: &SessionAdd, open: Vec<Msg>) -> Plan {
    let mut phases = period(s.start.date, open);
    phases.push(vec![
        Msg::Plan(PlanMsg::New),
        form(FormEdit::Title(s.title.clone())),
        form(FormEdit::Date(format_date(s.start.date))),
        form(FormEdit::Time(format_minute(s.start.minute))),
        form(FormEdit::Duration(s.minutes.to_string())),
    ]);
    phases.push(vec![Msg::Plan(PlanMsg::Save)]);
    phases
}

fn form(edit: FormEdit) -> Msg {
    Msg::Plan(PlanMsg::Form(edit))
}

/// Show the month of `day`, then select the day. The month is set, not moved by a count, so the plan
/// does not depend on the month the calendar had before it loaded.
fn period(day: Date, open: Vec<Msg>) -> Plan {
    vec![
        open,
        vec![Msg::Plan(PlanMsg::ShowMonth(day))],
        vec![Msg::Plan(PlanMsg::Select(day))],
    ]
}
