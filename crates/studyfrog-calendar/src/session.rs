//! Study sessions as plain data.

use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

/// How a session repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Repeat {
    /// Once, on the start date.
    Never,
    /// Every `every` days.
    Daily {
        /// Interval in days (at least 1).
        every: u16,
    },
    /// Every `every` weeks, on the chosen weekdays (bit 0 = Monday ... bit 6 = Sunday). An empty
    /// mask means the weekday of the start date.
    Weekly {
        /// Interval in weeks (at least 1).
        every: u16,
        /// Weekday bit mask.
        days: u8,
    },
    /// Every `every` months on the start date's day of the month; months without that day are skipped.
    Monthly {
        /// Interval in months (at least 1).
        every: u16,
    },
}

/// When a repeating session stops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ends {
    /// Never.
    Forever,
    /// On this date (inclusive).
    On(Date),
    /// After this many occurrences, counted from the start (cancelled ones count).
    After(u32),
}

/// A planned study session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    /// Identity.
    pub id: Uid,
    /// Title.
    pub title: String,
    /// The stack to rehearse, if any.
    pub stack: Option<Uid>,
    /// Local date and minute of the first occurrence.
    pub start: LocalTime,
    /// Length in minutes.
    pub duration_min: u16,
    /// Repetition rule.
    pub repeat: Repeat,
    /// End of the repetition.
    pub ends: Ends,
    /// Dates on which this session does not take place.
    pub cancelled: Vec<Date>,
    /// Free-form notes.
    pub notes: String,
}

/// One concrete occurrence of a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Occurrence {
    /// The session.
    pub session: Uid,
    /// Local date.
    pub date: Date,
    /// Local start, minutes since midnight.
    pub start_minute: u16,
    /// Length in minutes.
    pub duration_min: u16,
}

/// Most minutes a session can last (a day).
pub const MAX_DURATION_MIN: u16 = 1440;

/// Why a session is not acceptable, in words for the user.
pub fn validate_session(s: &Session) -> Result<(), String> {
    use crate::recurrence::MAX_AFTER;
    if s.title.trim().is_empty() || s.title.chars().any(|c| c.is_control()) {
        return Err("the title must be one non-empty line".to_owned());
    }
    if s.start.minute >= 1440 {
        return Err("the start time must be within the day".to_owned());
    }
    if s.duration_min == 0 || s.duration_min > MAX_DURATION_MIN {
        return Err("the duration must be between 1 minute and 24 hours".to_owned());
    }
    let every_ok = match s.repeat {
        Repeat::Never => true,
        Repeat::Daily { every } | Repeat::Monthly { every } | Repeat::Weekly { every, .. } => {
            every >= 1
        }
    };
    if !every_ok {
        return Err("the repeat interval must be at least 1".to_owned());
    }
    if matches!(s.repeat, Repeat::Weekly { days, .. } if days >= 128) {
        return Err("weekdays are Monday to Sunday".to_owned());
    }
    match s.ends {
        Ends::On(last) if last < s.start.date => Err("the end date is before the start".to_owned()),
        Ends::After(n) if n == 0 || n > MAX_AFTER => Err(format!(
            "the number of occurrences must be 1 to {MAX_AFTER}"
        )),
        _ => Ok(()),
    }
}
