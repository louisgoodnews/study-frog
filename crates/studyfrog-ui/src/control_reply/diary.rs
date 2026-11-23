//! The answers of the diary commands: the entries of a day or range, one entry (with whether the
//! editor shows it), and the entry a new one became. Pure.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};
use studyfrog_core::civil::{Date, format_date};
use studyfrog_core::diary::RatingValue;
use studyfrog_store::diary::StoredDiaryEntry;

use crate::control_cmd::diary::{DiaryCmd, DiaryNew};
use crate::control_reply::MAX_QUERY_ROWS;
use crate::control_state::ulid;
use crate::state::State;

fn rating_json(v: &RatingValue) -> Value {
    match v {
        RatingValue::Int(n) => json!(n),
        RatingValue::Text(t) => json!(t),
    }
}

fn ratings_json(ratings: &BTreeMap<String, RatingValue>) -> Value {
    let map: Map<String, Value> = ratings
        .iter()
        .map(|(k, v)| (k.clone(), rating_json(v)))
        .collect();
    Value::Object(map)
}

/// One entry as the diary stores it.
pub fn entry_json(e: &StoredDiaryEntry) -> Value {
    json!({
        "uid": ulid(e.uid), "path": e.path, "date": format_date(e.date), "title": e.title,
        "study_minutes": e.study_minutes, "tags": e.tags, "stacks": e.stacks,
        "items": e.items.iter().copied().map(ulid).collect::<Vec<_>>(),
        "ratings": ratings_json(&e.ratings), "content": e.content,
    })
}

fn day_key(d: Date) -> (i32, u8, u8) {
    (d.year, d.month, d.day)
}

/// The entries from `from` to `to` (inclusive; `to` defaults to `from`; both missing: all), by day.
fn list_json(state: &State, from: Option<Date>, to: Option<Date>) -> Value {
    let to = to.or(from);
    let mut shown: Vec<&StoredDiaryEntry> = state
        .diary
        .entries
        .iter()
        .filter(|e| from.is_none_or(|f| e.date >= f) && to.is_none_or(|t| e.date <= t))
        .collect();
    shown.sort_by_key(|e| (day_key(e.date), e.mtime));
    let count = shown.len();
    let rows: Vec<Value> = shown
        .into_iter()
        .take(MAX_QUERY_ROWS)
        .map(entry_json)
        .collect();
    json!({"count": count, "entries": rows})
}

/// The entry with this ULID, and whether the editor is showing it.
fn open_json(state: &State, uid: studyfrog_core::uid::Uid) -> Result<Value, String> {
    let entry = state
        .diary
        .entries
        .iter()
        .find(|e| e.uid == uid)
        .ok_or("no diary entry has that ULID")?;
    let editing = state
        .diary
        .active_entry
        .as_ref()
        .and_then(|d| d.uid)
        .is_some_and(|u| u == uid);
    let mut out = entry_json(entry);
    if let Some(o) = out.as_object_mut() {
        o.insert("editing".to_owned(), json!(editing));
    }
    Ok(out)
}

/// The entry a new one became: the latest with its day and title, or why it was not saved.
fn added(state: &State, new: &DiaryNew) -> Result<Value, String> {
    state
        .diary
        .entries
        .iter()
        .filter(|e| e.date == new.date && e.title.trim() == new.title.trim())
        .max_by_key(|e| e.mtime)
        .map(entry_json)
        .ok_or_else(|| "the entry was not saved".to_owned())
}

/// The answer to a `diary` command once the entries have loaded.
pub fn answer(state: &State, cmd: &DiaryCmd) -> Result<Value, String> {
    match cmd {
        DiaryCmd::List { from, to } => Ok(list_json(state, *from, *to)),
        DiaryCmd::Open(uid) => open_json(state, *uid),
        DiaryCmd::Add(new) => added(state, new),
    }
}
