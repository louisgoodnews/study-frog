//! Turning what the console receives into its entries (STORY-0133, STORY-0152): a returned JSON line, a script's
//! answer or events, and the export of the transcript. Pure.

use serde_json::Value;

use crate::console_ui::{ConsoleState, Entry, EntryKind, MAX_TEXT};

fn cut(text: String) -> String {
    if text.chars().count() <= MAX_TEXT {
        return text;
    }
    let kept: String = text.chars().take(MAX_TEXT).collect();
    format!("{kept}\n… (cut)")
}

/// The outcome line of a script run (its error, when it could not start, is added).
fn script_text(data: &Value) -> String {
    let job = data.get("job").and_then(Value::as_str).unwrap_or("?");
    let how = match (data.get("ok"), data.get("timed_out")) {
        (Some(Value::Bool(true)), _) => "finished".to_owned(),
        (_, Some(Value::Bool(true))) => "ran over its timeout".to_owned(),
        _ => format!(
            "failed (exit code {})",
            data.get("code").unwrap_or(&Value::Null)
        ),
    };
    let mut text = format!("script {job} {how}");
    if let Some(s) = text_of(data, "error") {
        text = format!("{text}\n{s}");
    }
    text
}

/// A non-empty string field of a script run's data.
fn text_of<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
    data.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
}

/// The data of a `script_finished` event, when the line is one.
fn script_run(v: &Value) -> Option<Value> {
    let data = v.get("data")?;
    if v.get("event").and_then(Value::as_str).is_none()
        || data.get("kind").and_then(Value::as_str) != Some("script_finished")
    {
        return None;
    }
    Some(data.get("data").cloned().unwrap_or(Value::Null))
}

/// The entries a returned line becomes. A script run gives its outcome, then its stdout and its stderr as
/// separate entries of their own streams (BUG-0033); any other line is one entry.
pub fn entries_of(line: &str, at: f64) -> Vec<Entry> {
    let Some(inner) = serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|v| script_run(&v))
    else {
        return vec![entry_of(line, at)];
    };
    let mut out = vec![Entry {
        at,
        kind: EntryKind::Script,
        text: script_text(&inner),
    }];
    for (key, kind) in [("stdout", EntryKind::Stdout), ("stderr", EntryKind::Stderr)] {
        if let Some(s) = text_of(&inner, key) {
            out.push(Entry {
                at,
                kind,
                text: cut(s.to_owned()),
            });
        }
    }
    out
}

/// The entry a returned line becomes.
pub fn entry_of(line: &str, at: f64) -> Entry {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Entry {
            at,
            kind: EntryKind::Answer,
            text: line.to_owned(),
        };
    };
    if let Some(inner) = script_run(&v) {
        return Entry {
            at,
            kind: EntryKind::Script,
            text: script_text(&inner),
        };
    }
    if let Some(event) = v.get("event").and_then(Value::as_str) {
        let data = v.get("data").cloned().unwrap_or(Value::Null);
        let kind = data.get("kind").and_then(Value::as_str).unwrap_or(event);
        let detail = data
            .get("data")
            .map_or_else(|| data.to_string(), Value::to_string);
        return Entry {
            at,
            kind: EntryKind::Event,
            text: format!("{kind} {detail}"),
        };
    }
    match v.get("ok").and_then(Value::as_bool) {
        Some(false) => Entry {
            at,
            kind: EntryKind::Error,
            text: v
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_owned(),
        },
        _ => {
            let result = v.get("result").unwrap_or(&v);
            let text = serde_json::to_string_pretty(result).unwrap_or_default();
            Entry {
                at,
                kind: EntryKind::Answer,
                text: cut(text),
            }
        }
    }
}

/// Whether an entry matches a search text (case-insensitive; an empty search matches all).
pub fn matches(e: &Entry, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    q.is_empty() || e.text.to_lowercase().contains(&q)
}

/// The transcript as text for the log file: one line per entry, with its time in seconds on the app's clock.
pub fn export_text(c: &ConsoleState) -> String {
    c.entries
        .iter()
        .map(|e| format!("[{:>10.1}s] {:?}: {}", e.at, e.kind, e.text))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
