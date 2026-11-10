//! Messages of the calendar screen.

use studyfrog_calendar::session::Session;
use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::civil::Date;
use studyfrog_core::uid::Uid;

use crate::calendar_mode::CalendarViewMode;
use crate::session_form::FormEdit;

/// Everything the calendar screen can be asked to do.
#[derive(Debug, Clone, PartialEq)]
pub enum PlanMsg {
    /// Sessions and today's date were read.
    Loaded(Result<(Vec<Session>, Date), String>),
    /// Switch calendar display mode (Month, Week, Day, Timetable).
    SetViewMode(CalendarViewMode),
    /// Shift visible date period (day, week, or month).
    ShiftPeriod(i64),
    /// Timetable entries were loaded.
    TimetableLoaded(Result<Vec<TimetableEntry>, String>),
    /// Open the timetable creation or editing modal.
    OpenTimetableForm(Option<TimetableEntry>),
    /// Close the timetable form.
    CloseTimetableForm,
    /// Save a timetable entry.
    SaveTimetable(TimetableEntry),
    /// Delete a timetable entry.
    DeleteTimetable(Uid),
    /// Timetable entries changed on disk.
    TimetableChanged(Result<Vec<TimetableEntry>, String>),
    /// Show another month (relative).
    Month(i64),
    /// Jump to today.
    Today,
    /// Choose a day.
    Select(Date),
    /// Open the form for a new session on the selected day.
    New,
    /// Open the form for an existing session.
    Edit(Uid),
    /// Close the form without saving.
    CloseForm,
    /// A form field changed.
    Form(FormEdit),
    /// "Save" was activated.
    Save,
    /// "Delete" was activated for a session.
    Delete(Uid),
    /// Skip one occurrence of a session.
    Skip(Uid, Date),
    /// A save, delete or skip finished; carries all sessions.
    Changed(Result<Vec<Session>, String>),
    /// Rehearse the stack of a session.
    Rehearse(Uid),
    /// Set or change the due date of a stack.
    StackDeadline(Uid),
    /// Set or clear the due date of a stack directly.
    SetStackDueDate {
        /// The stack to set or clear the deadline for.
        stack: Uid,
        /// The new date, or `None` to clear.
        date: Option<Date>,
        /// Reminders in minutes before the due date.
        reminders: Vec<u32>,
    },
    /// The reviews the schedule planned.
    Reviews(crate::planner_reviews::ReviewMsg),
}
