//! Changes an AI client proposed over MCP and the user has not applied yet (STORY-0119), as JSON:
//! the same action objects the assistant's proposals use, plus `log_diary` and `plan_session`. Pure;
//! the service keeps the files.

use serde_json::{Value, json};
use studyfrog_core::civil::parse_date;

use crate::wiki::{line_of, text_of};
use crate::wiki_change::{Change, DiaryChange, SessionChange};

/// The kinds of calendar entries a client may plan.
pub const SESSION_KINDS: [&str; 4] = ["study", "exam", "task", "appointment"];

/// A proposal waiting for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedProposal {
    /// Its id (the file name without `.json`).
    pub id: String,
    /// When it was staged, in Unix milliseconds.
    pub created_ms: i64,
    /// The MCP tool that staged it.
    pub tool: String,
    /// What it is about, in a sentence.
    pub summary: String,
    /// The changes.
    pub changes: Vec<Change>,
}

fn date_of(v: &Value, key: &str) -> Result<String, String> {
    let text = line_of(v, key).ok_or_else(|| format!("\"{key}\" is missing"))?;
    parse_date(&text)
        .map(|_| text.clone())
        .ok_or_else(|| format!("\"{key}\" is not a date YYYY-MM-DD: {text}"))
}

fn ratings_of(v: &Value) -> Result<Vec<(String, u8)>, String> {
    let Some(map) = v.get("ratings").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    map.iter()
        .map(|(scale, n)| {
            let n = n.as_u64().filter(|n| (1..=5).contains(n));
            let clean =
                !scale.is_empty() && scale.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
            match (n, clean) {
                (Some(n), true) => Ok((scale.clone(), u8::try_from(n).unwrap_or(5))),
                _ => Err(format!(
                    "the rating \"{scale}\" must be a scale name with a value from 1 to 5"
                )),
            }
        })
        .collect()
}

fn folders_of(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

/// A diary entry from a tool's arguments (or a staged `log_diary` action).
pub fn diary_change_of(v: &Value) -> Result<Change, String> {
    let minutes = match v.get("study_minutes") {
        None | Some(Value::Null) => None,
        Some(m) => Some(
            m.as_u64()
                .filter(|m| *m <= 1440)
                .and_then(|m| u32::try_from(m).ok())
                .ok_or("\"study_minutes\" is a number of minutes up to 1440")?,
        ),
    };
    Ok(Change::LogDiary(DiaryChange {
        date: date_of(v, "date")?,
        title: line_of(v, "title").unwrap_or_default(),
        body: text_of(v, "body").unwrap_or_default(),
        minutes,
        ratings: ratings_of(v)?,
        stacks: folders_of(v, "stacks"),
    }))
}

/// `HH:MM` as minutes after midnight.
fn minute_of(text: &str) -> Option<u16> {
    let (h, m) = text.trim().split_once(':')?;
    let (h, m): (u16, u16) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then_some(h * 60 + m)
}

/// A calendar entry from a tool's arguments (or a staged `plan_session` action). `stacks` are the
/// vault's stack folders; a named stack must be one of them.
pub fn session_change_of(v: &Value, stacks: &[String]) -> Result<Change, String> {
    let start = line_of(v, "start").ok_or("\"start\" is missing (HH:MM)")?;
    let start_minute = minute_of(&start).ok_or("\"start\" is written HH:MM")?;
    let duration_min = v
        .get("duration_minutes")
        .and_then(Value::as_u64)
        .filter(|d| (1..=1440).contains(d))
        .and_then(|d| u16::try_from(d).ok())
        .ok_or("\"duration_minutes\" is from 1 to 1440")?;
    let stack = line_of(v, "stack");
    if let Some(s) = stack.as_ref().filter(|s| !stacks.contains(s)) {
        return Err(format!("there is no stack folder \"{s}\""));
    }
    let kind = line_of(v, "kind").unwrap_or_else(|| "study".to_owned());
    if !SESSION_KINDS.contains(&kind.as_str()) {
        return Err(format!("\"kind\" is one of {}", SESSION_KINDS.join(", ")));
    }
    Ok(Change::PlanSession(SessionChange {
        title: line_of(v, "title").ok_or("\"title\" is missing")?,
        date: date_of(v, "date")?,
        start_minute,
        duration_min,
        stack,
        kind,
        notes: text_of(v, "notes").unwrap_or_default(),
    }))
}

fn diary_json(d: &DiaryChange) -> Value {
    let ratings: serde_json::Map<String, Value> = d
        .ratings
        .iter()
        .map(|(k, n)| (k.clone(), json!(n)))
        .collect();
    json!({"action": "log_diary", "date": d.date, "title": d.title, "body": d.body,
        "study_minutes": d.minutes, "ratings": ratings, "stacks": d.stacks})
}

fn session_json(s: &SessionChange) -> Value {
    json!({"action": "plan_session", "title": s.title, "date": s.date,
        "start": format!("{:02}:{:02}", s.start_minute / 60, s.start_minute % 60),
        "duration_minutes": s.duration_min, "stack": s.stack, "kind": s.kind, "notes": s.notes})
}

/// A change as the action object it is staged as.
pub fn change_json(c: &Change) -> Value {
    match c {
        Change::CreateNote { stack, title, body } => {
            json!({"action": "create_note", "stack": stack, "title": title, "body": body})
        }
        Change::CreateCard {
            stack,
            front,
            back,
            source,
        } => {
            json!({"action": "create_card", "stack": stack, "front": front, "back": back, "source": source})
        }
        Change::CreateQuestion {
            stack,
            prompt,
            reference,
            source,
        } => {
            json!({"action": "create_question", "stack": stack, "prompt": prompt, "reference": reference, "source": source})
        }
        Change::CreatePage { name, text } => {
            json!({"action": "create_page", "name": name, "text": text})
        }
        Change::UpdatePage { name, text } => {
            json!({"action": "update_page", "name": name, "text": text})
        }
        Change::UpdateItem { uid, fields } => {
            let f: serde_json::Map<String, Value> =
                fields.iter().map(|(k, t)| (k.clone(), json!(t))).collect();
            json!({"action": "update_item", "uid": uid, "fields": f})
        }
        Change::LogDiary(d) => diary_json(d),
        Change::PlanSession(s) => session_json(s),
    }
}

/// The text of a staged proposal's file.
pub fn staged_json(p: &StagedProposal) -> String {
    let v = json!({
        "id": p.id, "created_ms": p.created_ms, "tool": p.tool, "summary": p.summary,
        "changes": p.changes.iter().map(change_json).collect::<Vec<_>>(),
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

/// A change from its action object, checked against the vault's stack folders: `create_note`,
/// `create_card`, `create_question`, `log_diary` or `plan_session`.
pub fn proposed_change(v: &Value, stacks: &[String]) -> Result<Change, String> {
    match v.get("action").and_then(Value::as_str) {
        Some("log_diary") => diary_change_of(v),
        Some("plan_session") => session_change_of(v, stacks),
        _ => crate::wiki::change_of(v, &[], stacks),
    }
}

/// Read a staged proposal's file, checking every change again against the vault's stack folders as
/// they are now. Changes that no longer pass are returned as sentences.
pub fn parse_staged(
    text: &str,
    stacks: &[String],
) -> Result<(StagedProposal, Vec<String>), String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("not a staged proposal: {e}"))?;
    let word = |k: &str| v.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
    let mut out = StagedProposal {
        id: word("id"),
        created_ms: v.get("created_ms").and_then(Value::as_i64).unwrap_or(0),
        tool: word("tool"),
        summary: word("summary"),
        changes: Vec::new(),
    };
    let mut rejected = Vec::new();
    let list = v
        .get("changes")
        .and_then(Value::as_array)
        .ok_or("the proposal has no changes")?;
    for (i, c) in list.iter().enumerate() {
        match proposed_change(c, stacks) {
            Ok(change) => out.changes.push(change),
            Err(why) => rejected.push(format!("Change {}: {why}", i + 1)),
        }
    }
    Ok((out, rejected))
}
