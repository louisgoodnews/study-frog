//! The calendar commands of the control protocol: the view, the period, a day's homework and the reminders due.

use studyfrog_core::civil::Date;

use crate::calendar_mode::CalendarViewMode;

/// What a `calendar` or `reminders` command asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum CalendarCmd {
    /// Switch the view: month, week, day or timetable.
    View(CalendarViewMode),
    /// Show the month of this date and select the day.
    Period(Date),
    /// The homework due on this day, with its urgency.
    Homework(Date),
    /// The calendar and homework reminders due now, without marking them shown.
    RemindersDue,
}
