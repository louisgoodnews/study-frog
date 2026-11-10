//! Timetable entries: recurring weekly lectures, seminars, and lessons.
//! Pure; converted to sessions or occurrences for calendar integration.

use studyfrog_core::civil::{Date, date_from_days, date_to_days, weekday, weekday_index};
use studyfrog_core::uid::Uid;

use crate::session::{Ends, EventKind, Occurrence, Repeat, Session};

/// A recurring weekly course, lecture, seminar, or class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimetableEntry {
    /// Identity.
    pub id: Uid,
    /// Course title (e.g., "Calculus I", "Cell Biology").
    pub title: String,
    /// Day of the week: 0 = Monday, 6 = Sunday.
    pub day_of_week: u8,
    /// Start time in minutes since midnight (0..1440).
    pub start_minute: u16,
    /// Length of the session in minutes.
    pub duration_min: u16,
    /// Lecture hall, room or link.
    pub location: Option<String>,
    /// Professor or instructor name.
    pub instructor: Option<String>,
    /// Associated flashcard stack for rapid rehearsal.
    pub stack_uid: Option<Uid>,
}

/// Validate a timetable entry.
pub fn validate_timetable_entry(entry: &TimetableEntry) -> Result<(), String> {
    if entry.title.trim().is_empty() || entry.title.chars().any(|c| c.is_control()) {
        return Err("the title must be one non-empty line".to_owned());
    }
    if entry.day_of_week > 6 {
        return Err("day of week must be between 0 (Monday) and 6 (Sunday)".to_owned());
    }
    if entry.start_minute >= 1440 {
        return Err("start time must be within the day".to_owned());
    }
    if entry.duration_min == 0 || entry.duration_min > 1440 {
        return Err("duration must be between 1 minute and 24 hours".to_owned());
    }
    Ok(())
}

/// Project a timetable entry into a recurring weekly calendar session.
pub fn timetable_to_session(entry: &TimetableEntry, start_date: Date) -> Session {
    let mut notes_parts = Vec::new();
    if let Some(loc) = &entry.location {
        notes_parts.push(format!("Room: {loc}"));
    }
    if let Some(inst) = &entry.instructor {
        notes_parts.push(format!("Instructor: {inst}"));
    }
    Session {
        id: entry.id,
        title: entry.title.clone(),
        stack: entry.stack_uid,
        start: studyfrog_core::civil::LocalTime {
            date: start_date,
            minute: entry.start_minute,
        },
        duration_min: entry.duration_min,
        repeat: Repeat::Weekly {
            every: 1,
            days: 1 << entry.day_of_week,
        },
        ends: Ends::Forever,
        cancelled: Vec::new(),
        notes: notes_parts.join(" · "),
        kind: EventKind::Appointment,
        reminders: vec![15],
    }
}

/// Expand timetable entries into concrete occurrences between `from` and `to` inclusive.
pub fn timetable_occurrences(entries: &[TimetableEntry], from: Date, to: Date) -> Vec<Occurrence> {
    let from_day = date_to_days(from);
    let to_day = date_to_days(to);
    if from_day > to_day {
        return Vec::new();
    }
    let mut out = Vec::new();
    for day_num in from_day..=to_day {
        let d = date_from_days(day_num);
        let w = weekday_index(weekday(d));
        for e in entries {
            if e.day_of_week == w {
                out.push(Occurrence {
                    session: e.id,
                    date: d,
                    start_minute: e.start_minute,
                    duration_min: e.duration_min,
                });
            }
        }
    }
    out.sort_by_key(|o| (o.date, o.start_minute, o.session));
    out
}
