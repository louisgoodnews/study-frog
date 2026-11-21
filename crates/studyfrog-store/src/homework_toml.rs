//! Deterministic TOML form of homework items (STORY-0165). Items are written in due order, then by
//! id, so the same items always give the same text. Used for vault exports and tests.

use serde::{Deserialize, Serialize};
use studyfrog_calendar::homework::{ChecklistItem, HomeworkItem};
use studyfrog_core::civil::{LocalTime, format_date, parse_date};
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::{format_ulid, parse_ulid};

use crate::error::StoreError;
use crate::homework::{priority_code, priority_from_code, status_code, status_from_code};

#[derive(Serialize, Deserialize)]
struct Doc {
    #[serde(default)]
    homework: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    id: String,
    title: String,
    course: Option<String>,
    start: Option<String>,
    due: String,
    due_minute: u16,
    estimate_min: u32,
    priority: String,
    status: String,
    notes: String,
    #[serde(default)]
    checklist: Vec<Step>,
}

#[derive(Serialize, Deserialize)]
struct Step {
    text: String,
    done: bool,
}

fn corrupt(what: &str, v: &str) -> StoreError {
    StoreError::Corrupt(format!("{what}: {v:?}"))
}

/// Render homework items as TOML, in due order then by id.
pub fn homework_to_toml(items: &[HomeworkItem]) -> String {
    let mut sorted: Vec<&HomeworkItem> = items.iter().collect();
    sorted.sort_by(|a, b| a.due.cmp(&b.due).then(a.id.0.cmp(&b.id.0)));
    let doc = Doc {
        homework: sorted.into_iter().map(entry_of).collect(),
    };
    toml::to_string(&doc).unwrap_or_default()
}

fn entry_of(item: &HomeworkItem) -> Entry {
    Entry {
        id: format_ulid(item.id),
        title: item.title.clone(),
        course: item.course.map(format_ulid),
        start: item.start.map(format_date),
        due: format_date(item.due.date),
        due_minute: item.due.minute,
        estimate_min: item.estimate_min,
        priority: priority_code(item.priority).to_owned(),
        status: status_code(item.status).to_owned(),
        notes: item.notes.clone(),
        checklist: item
            .checklist
            .iter()
            .map(|c| Step {
                text: c.text.clone(),
                done: c.done,
            })
            .collect(),
    }
}

/// Parse homework items from the TOML written by [`homework_to_toml`].
pub fn homework_from_toml(text: &str) -> Result<Vec<HomeworkItem>, StoreError> {
    let doc: Doc = toml::from_str(text).map_err(|e| StoreError::Corrupt(e.to_string()))?;
    doc.homework.into_iter().map(item_of).collect()
}

fn item_of(e: Entry) -> Result<HomeworkItem, StoreError> {
    let id: Uid = parse_ulid(&e.id).ok_or_else(|| corrupt("ulid", &e.id))?;
    let course = e
        .course
        .as_deref()
        .map(|s| parse_ulid(s).ok_or_else(|| corrupt("ulid", s)))
        .transpose()?;
    let start = e
        .start
        .as_deref()
        .map(|s| parse_date(s).ok_or_else(|| corrupt("date", s)))
        .transpose()?;
    let date = parse_date(&e.due).ok_or_else(|| corrupt("date", &e.due))?;
    Ok(HomeworkItem {
        id,
        title: e.title,
        course,
        start,
        due: LocalTime {
            date,
            minute: e.due_minute,
        },
        estimate_min: e.estimate_min,
        priority: priority_from_code(&e.priority)?,
        status: status_from_code(&e.status)?,
        checklist: e
            .checklist
            .into_iter()
            .map(|s| ChecklistItem {
                text: s.text,
                done: s.done,
            })
            .collect(),
        notes: e.notes,
    })
}
