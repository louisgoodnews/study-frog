//! `.studyfrog/homework.toml`: TOML DTOs, validation, canonicalization, and deterministic rendering.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use studyfrog_core::civil::{parse_date, parse_minute};

use crate::error::{ConfigError, config_error, from_toml};
pub use crate::homework_render::render_homework_toml;

/// File name inside `.studyfrog/`.
pub const HOMEWORK_FILE: &str = "homework.toml";
/// Canonical priority names accepted in TOML.
pub const HOMEWORK_PRIORITY_NAMES: [&str; 4] = ["low", "medium", "high", "urgent"];
/// Canonical status names accepted in TOML.
pub const HOMEWORK_STATUS_NAMES: [&str; 3] = ["not_started", "in_progress", "completed"];
/// Maximum reminders per homework item.
pub const MAX_HOMEWORK_REMINDERS: usize = 5;
/// Maximum reminder lead time in minutes (4 weeks).
pub const MAX_HOMEWORK_REMINDER_MIN: u32 = 40_320;
/// Maximum estimated duration in minutes (365 days).
pub const MAX_HOMEWORK_ESTIMATED_MINUTES: u32 = 525_600;

/// Root structure of `.studyfrog/homework.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomeworkFileDto {
    /// Homework entries stored in this vault.
    #[serde(default)]
    pub homework: Vec<HomeworkItemDto>,
}

/// Serialized representation of one homework assignment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomeworkItemDto {
    /// ULID string.
    pub id: String,
    /// Single-line assignment title.
    pub title: String,
    /// Optional single-line subject/course name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Optional linked stack ULID string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_uid: Option<String>,
    /// Optional linked timetable entry ULID string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timetable_id: Option<String>,
    /// Due date formatted as `YYYY-MM-DD`.
    pub due_date: String,
    /// Due time of day formatted as `HH:MM`.
    pub due_time: String,
    /// Optional start date formatted as `YYYY-MM-DD`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// Estimated effort in minutes (`>= 1`).
    pub estimated_minutes: u32,
    /// Priority (`low`, `medium`, `high`, `urgent`).
    pub priority: String,
    /// Status (`not_started`, `in_progress`, `completed`).
    pub status: String,
    /// Free-form multi-line notes.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,
    /// Reminder lead times in minutes before due time.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reminders: Vec<u32>,
    /// Ordered checklist sub-tasks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checklist: Vec<ChecklistItemDto>,
}

/// Serialized representation of one checklist item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChecklistItemDto {
    /// Single-line sub-task description.
    pub text: String,
    /// Completion flag.
    #[serde(default)]
    pub done: bool,
}

/// Sort homework DTOs deterministically by `(due_date, due_time, id)`.
pub fn sort_homework_dtos(items: &mut [HomeworkItemDto]) {
    items.sort_by(|a, b| (&a.due_date, &a.due_time, &a.id).cmp(&(&b.due_date, &b.due_time, &b.id)));
}

/// Return a clone with sorted, deduplicated reminders.
pub fn canonicalize_homework_dto(item: &HomeworkItemDto) -> HomeworkItemDto {
    let mut out = item.clone();
    out.reminders.sort_unstable();
    out.reminders.dedup();
    out
}

fn err(msg: impl Into<String>) -> ConfigError {
    config_error(None, msg)
}

fn is_single_line(text: &str) -> bool {
    !text.trim().is_empty()
        && !text
            .chars()
            .any(|c| c.is_control() || c == '\u{2028}' || c == '\u{2029}')
}

fn is_clean_id(id: &str) -> bool {
    !id.is_empty() && !id.chars().any(|c| c.is_whitespace() || c.is_control())
}

fn is_hhmm(text: &str) -> bool {
    let b = text.as_bytes();
    b.len() == 5
        && b[0].is_ascii_digit()
        && b[1].is_ascii_digit()
        && b[2] == b':'
        && b[3].is_ascii_digit()
        && b[4].is_ascii_digit()
}

fn validate_dates(item: &HomeworkItemDto) -> Result<(), ConfigError> {
    let due_date = parse_date(&item.due_date)
        .ok_or_else(|| err(format!("invalid due_date {:?}", item.due_date)))?;
    if !is_hhmm(&item.due_time) || parse_minute(&item.due_time).is_none() {
        return Err(err(format!("invalid due_time {:?}", item.due_time)));
    }
    if let Some(start_str) = item.start_date.as_deref() {
        let start = parse_date(start_str)
            .ok_or_else(|| err(format!("invalid start_date {start_str:?}")))?;
        if start > due_date {
            return Err(err("start_date cannot be after due_date"));
        }
    }
    Ok(())
}

fn validate_extras(item: &HomeworkItemDto) -> Result<(), ConfigError> {
    if !(1..=MAX_HOMEWORK_ESTIMATED_MINUTES).contains(&item.estimated_minutes) {
        return Err(err("estimated_minutes must be between 1 and 525600"));
    }
    if !HOMEWORK_PRIORITY_NAMES.contains(&item.priority.as_str()) {
        return Err(err(format!(
            "unknown homework priority {:?}",
            item.priority
        )));
    }
    if !HOMEWORK_STATUS_NAMES.contains(&item.status.as_str()) {
        return Err(err(format!("unknown homework status {:?}", item.status)));
    }
    if item.reminders.len() > MAX_HOMEWORK_REMINDERS {
        return Err(err("too many homework reminders"));
    }
    if item
        .reminders
        .iter()
        .any(|m| *m > MAX_HOMEWORK_REMINDER_MIN)
    {
        return Err(err("homework reminder exceeds four weeks"));
    }
    for c in &item.checklist {
        if !is_single_line(&c.text) {
            return Err(err("checklist item text must be one non-empty line"));
        }
    }
    Ok(())
}

/// Validate a [`HomeworkItemDto`] against schema and domain rules.
pub fn validate_homework_dto(item: &HomeworkItemDto) -> Result<(), ConfigError> {
    if !is_clean_id(&item.id) {
        return Err(err("homework id must be non-empty without whitespace"));
    }
    if !is_single_line(&item.title) {
        return Err(err("homework title must be one non-empty line"));
    }
    if item.subject.as_deref().is_some_and(|s| !is_single_line(s)) {
        return Err(err("homework subject must be one non-empty line"));
    }
    for (label, val) in [
        ("stack_uid", &item.stack_uid),
        ("timetable_id", &item.timetable_id),
    ] {
        if val.as_deref().is_some_and(|s| !is_clean_id(s)) {
            return Err(err(format!("homework {label} must be non-empty")));
        }
    }
    validate_dates(item)?;
    validate_extras(item)
}

/// Parse and validate `.studyfrog/homework.toml`.
pub fn parse_homework_toml(text: &str) -> Result<HomeworkFileDto, ConfigError> {
    let raw: HomeworkFileDto = toml::from_str(text).map_err(|e| from_toml(text, &e))?;
    let mut seen = HashSet::with_capacity(raw.homework.len());
    let mut out = Vec::with_capacity(raw.homework.len());
    for item in &raw.homework {
        if !seen.insert(item.id.to_ascii_uppercase()) {
            return Err(err(format!("duplicate homework id {:?}", item.id)));
        }
        validate_homework_dto(item)?;
        out.push(canonicalize_homework_dto(item));
    }
    Ok(HomeworkFileDto { homework: out })
}
