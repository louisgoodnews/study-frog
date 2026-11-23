//! The calendar commands of the control protocol: the view, the period, a day's homework, a new
//! session and the reminders due.

use studyfrog_core::civil::{Date, LocalTime};

use crate::calendar_mode::CalendarViewMode;

/// What a `calendar` or `reminders` command asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarCmd {
    /// Read the view and the period on screen (opens the calendar).
    Show,
    /// Switch the view: month, week, day or timetable.
    View(CalendarViewMode),
    /// Show the month of this date and select the day.
    Period(Date),
    /// The homework due on this day, with its urgency.
    Homework(Date),
    /// Create a calendar session through the planner's form.
    SessionAdd(SessionAdd),
    /// The calendar and homework reminders due now, without marking them shown.
    RemindersDue,
}

/// A new calendar session: its title, its start and its length in minutes.
#[derive(Debug, Clone, PartialEq)]
pub struct SessionAdd {
    /// The title.
    pub title: String,
    /// The local start: the date and the minute of the day.
    pub start: LocalTime,
    /// The length in minutes.
    pub minutes: u16,
}
