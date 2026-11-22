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
    for key in ["stdout", "stderr", "error"] {
        if let Some(s) = data
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            text = format!("{text}\n{s}");
        }
    }
    text
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
    if let Some(event) = v.get("event").and_then(Value::as_str) {
        let data = v.get("data").cloned().unwrap_or(Value::Null);
        if data.get("kind").and_then(Value::as_str) == Some("script_finished") {
            let inner = data.get("data").cloned().unwrap_or(Value::Null);
            return Entry {
                at,
                kind: EntryKind::Script,
                text: script_text(&inner),
            };
        }
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
