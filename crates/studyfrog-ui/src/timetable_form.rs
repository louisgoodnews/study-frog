//! Form data and validation for weekly timetable entries (STORY-0141).

use studyfrog_calendar::timetable::{TimetableEntry, validate_timetable_entry};
use studyfrog_core::uid::Uid;

/// Form state for editing or creating a timetable entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimetableForm {
    /// Identity, or `None` if new.
    pub id: Option<Uid>,
    /// Course or lecture title.
    pub title: String,
    /// Day of the week: 0 = Mon, 6 = Sun.
    pub day_of_week: u8,
    /// Start hour (0..23).
    pub start_hour: u8,
    /// Start minute (0..59).
    pub start_minute: u8,
    /// Length of the class in minutes.
    pub duration_min: u16,
    /// Lecture hall, room or virtual meeting link.
    pub location: String,
    /// Professor or instructor.
    pub instructor: String,
    /// Linked flashcard stack for rapid review.
    pub stack_uid: Option<Uid>,
}

/// A fresh form with sensible defaults.
pub fn timetable_form_new(day_of_week: u8) -> TimetableForm {
    TimetableForm {
        id: None,
        title: String::new(),
        day_of_week: day_of_week.min(6),
        start_hour: 9,
        start_minute: 0,
        duration_min: 90,
        location: String::new(),
        instructor: String::new(),
        stack_uid: None,
    }
}

/// Populate a form from an existing timetable entry.
pub fn timetable_form_from(e: &TimetableEntry) -> TimetableForm {
    TimetableForm {
        id: Some(e.id),
        title: e.title.clone(),
        day_of_week: e.day_of_week,
        start_hour: (e.start_minute / 60) as u8,
        start_minute: (e.start_minute % 60) as u8,
        duration_min: e.duration_min,
        location: e.location.clone().unwrap_or_default(),
        instructor: e.instructor.clone().unwrap_or_default(),
        stack_uid: e.stack_uid,
    }
}

/// Convert form state into a valid TimetableEntry.
pub fn timetable_from_form(f: &TimetableForm) -> Result<TimetableEntry, String> {
    let title = f.title.trim().to_owned();
    if title.is_empty() {
        return Err("the course title cannot be empty".to_owned());
    }
    let start_minute = u16::from(f.start_hour) * 60 + u16::from(f.start_minute);
    let location = if f.location.trim().is_empty() {
        None
    } else {
        Some(f.location.trim().to_owned())
    };
    let instructor = if f.instructor.trim().is_empty() {
        None
    } else {
        Some(f.instructor.trim().to_owned())
    };
    let entry = TimetableEntry {
        id: f.id.unwrap_or(Uid(0)),
        title,
        day_of_week: f.day_of_week,
        start_minute,
        duration_min: f.duration_min,
        location,
        instructor,
        stack_uid: f.stack_uid,
    };
    validate_timetable_entry(&entry)?;
    Ok(entry)
}
