//! The homework create and edit dialog's data (STORY-0167): the fields as the student types them,
//! and the rules that turn them into a homework item. Pure.

use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, format_date, parse_date, parse_minute};
use studyfrog_core::uid::Uid;

/// The form's text and numbers, as the student has left them.
#[derive(Debug, Clone, PartialEq)]
pub struct HomeworkForm {
    /// The item being edited, or `None` for a new one.
    pub existing: Option<Uid>,
    /// The title, one line.
    pub title: String,
    /// The stack the work belongs to, if any.
    pub course: Option<Uid>,
    /// The due date as `YYYY-MM-DD`.
    pub due_date: String,
    /// The due time as `HH:MM`.
    pub due_time: String,
    /// Estimated effort in minutes.
    pub estimate_min: u32,
    /// Priority.
    pub priority: HomeworkPriority,
    /// The checklist, one step per line.
    pub checklist: String,
    /// Free text.
    pub notes: String,
    /// Kept from the item being edited: the start date, status and ticked steps.
    start: Option<Date>,
    status: HomeworkStatus,
    original: Vec<ChecklistItem>,
}

/// One field the student changed.
#[derive(Debug, Clone, PartialEq)]
pub enum HomeworkEdit {
    /// The title.
    Title(String),
    /// The stack, or none.
    Course(Option<Uid>),
    /// The due date text.
    DueDate(String),
    /// The due time text.
    DueTime(String),
    /// The estimate in minutes.
    Estimate(u32),
    /// The priority.
    Priority(HomeworkPriority),
    /// The checklist text.
    Checklist(String),
    /// The notes.
    Notes(String),
}

/// The default estimate of a new item, in minutes.
pub const DEFAULT_ESTIMATE_MIN: u32 = 60;

/// A blank form for a new item due on `today` at the end of the day.
pub fn new_form(today: Date) -> HomeworkForm {
    HomeworkForm {
        existing: None,
        title: String::new(),
        course: None,
        due_date: format_date(today),
        due_time: "23:59".to_owned(),
        estimate_min: DEFAULT_ESTIMATE_MIN,
        priority: HomeworkPriority::Medium,
        checklist: String::new(),
        notes: String::new(),
        start: None,
        status: HomeworkStatus::NotStarted,
        original: Vec::new(),
    }
}

/// A form filled in from an existing item.
pub fn form_of(item: &HomeworkItem) -> HomeworkForm {
    HomeworkForm {
        existing: Some(item.id),
        title: item.title.clone(),
        course: item.course,
        due_date: format_date(item.due.date),
        due_time: studyfrog_core::civil::format_minute(item.due.minute),
        estimate_min: item.estimate_min,
        priority: item.priority,
        checklist: item
            .checklist
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        notes: item.notes.clone(),
        start: item.start,
        status: item.status,
        original: item.checklist.clone(),
    }
}

/// Apply one field change.
pub fn apply(form: &mut HomeworkForm, edit: HomeworkEdit) {
    match edit {
        HomeworkEdit::Title(v) => form.title = v,
        HomeworkEdit::Course(v) => form.course = v,
        HomeworkEdit::DueDate(v) => form.due_date = v,
        HomeworkEdit::DueTime(v) => form.due_time = v,
        HomeworkEdit::Estimate(v) => form.estimate_min = v,
        HomeworkEdit::Priority(v) => form.priority = v,
        HomeworkEdit::Checklist(v) => form.checklist = v,
        HomeworkEdit::Notes(v) => form.notes = v,
    }
}

/// The checklist from the text: one step per non-blank line. A step keeps its ticked state when
/// its text is unchanged.
fn steps(form: &HomeworkForm) -> Vec<ChecklistItem> {
    form.checklist
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|text| ChecklistItem {
            text: text.to_owned(),
            done: form.original.iter().any(|c| c.done && c.text == text),
        })
        .collect()
}

/// The homework item the form describes, under `id`. A new item's id is assigned by the caller.
pub fn to_item(form: &HomeworkForm, id: Uid) -> Result<HomeworkItem, String> {
    let date = parse_date(form.due_date.trim())
        .ok_or_else(|| "the due date must look like 2026-11-30".to_owned())?;
    let minute = parse_minute(&form.due_time)
        .ok_or_else(|| "the due time must look like 18:30".to_owned())?;
    Ok(HomeworkItem {
        id,
        title: form.title.trim().to_owned(),
        course: form.course,
        start: form.start,
        due: studyfrog_core::civil::LocalTime { date, minute },
        estimate_min: form.estimate_min,
        priority: form.priority,
        status: form.status,
        checklist: steps(form),
        notes: form.notes.clone(),
    })
}
