//! The Console screen (STORY-0133): control commands and `script run|list` typed by hand, answers,
//! script output and lifecycle events as they arrive. Pure: lines go out through `outbox`, which the
//! window's glue (`console_link`) sends to its own control protocol, and come back as `Line`.

use serde_json::Value;

use crate::effect::Effect;
use crate::state::State;

/// Entries kept; older ones are dropped.
pub const MAX_ENTRIES: usize = 500;
/// Characters of one answer shown; the rest is cut.
pub const MAX_TEXT: usize = 4000;

/// What an entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    /// A line the user typed.
    Command,
    /// A successful answer.
    Answer,
    /// An error answer.
    Error,
    /// A lifecycle event.
    Event,
    /// What a script printed, and how it ended.
    Script,
    /// Standard output message from a script.
    Stdout,
    /// Diagnostic stderr or error trace from a script.
    Stderr,
}

/// One entry of the transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// What it is.
    pub kind: EntryKind,
    /// The text shown.
    pub text: String,
}

/// The console's state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConsoleState {
    /// The line being typed.
    pub input: String,
    /// The transcript, oldest first.
    pub entries: Vec<Entry>,
    /// Lines typed before, oldest first.
    pub history: Vec<String>,
    /// Where Up/Down are in `history`.
    pub history_at: Option<usize>,
    /// Lines to send, taken by the glue each frame.
    pub outbox: Vec<String>,
    /// In-app script development workbench.
    pub studio: crate::script_studio::StudioState,
}

/// What the console is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleMsg {
    /// The input changed.
    Edit(String),
    /// Enter: send the line.
    Submit,
    /// Up: the line before.
    Older,
    /// Down: the line after.
    Newer,
    /// Empty the transcript.
    Clear,
    /// A line came back (a JSON answer or event, or plain text).
    Line(String),
    /// ScriptStudio: switch active tab.
    StudioSelectTab(usize),
    /// ScriptStudio: close tab.
    StudioCloseTab(usize),
    /// ScriptStudio: new draft tab.
    StudioNewTab,
    /// ScriptStudio: edit code in active tab.
    StudioEditCode(String),
    /// ScriptStudio: open a script from path.
    StudioOpenScript(std::path::PathBuf),
    /// ScriptStudio: save active tab.
    StudioSaveScript(Option<String>),
    /// ScriptStudio: execute active tab as FrogScript.
    StudioRunScript,
    /// ScriptStudio: set output stream filter.
    StudioSetFilter(crate::script_studio::ConsoleFilter),
    /// ScriptStudio: toggle editor visibility.
    StudioToggleExpanded,
    /// ScriptStudio: refresh script file list.
    StudioRefreshList,
}

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
pub fn entry_of(line: &str) -> Entry {
    let Ok(v) = serde_json::from_str::<Value>(line) else {
        return Entry {
            kind: EntryKind::Answer,
            text: line.to_owned(),
        };
    };
    if let Some(event) = v.get("event").and_then(Value::as_str) {
        let data = v.get("data").cloned().unwrap_or(Value::Null);
        if data.get("kind").and_then(Value::as_str) == Some("script_finished") {
            let inner = data.get("data").cloned().unwrap_or(Value::Null);
            return Entry {
                kind: EntryKind::Script,
                text: script_text(&inner),
            };
        }
        let kind = data.get("kind").and_then(Value::as_str).unwrap_or(event);
        let detail = data
            .get("data")
            .map_or_else(|| data.to_string(), Value::to_string);
        return Entry {
            kind: EntryKind::Event,
            text: format!("{kind} {detail}"),
        };
    }
    match v.get("ok").and_then(Value::as_bool) {
        Some(false) => Entry {
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
                kind: EntryKind::Answer,
                text: cut(text),
            }
        }
    }
}

fn push(c: &mut ConsoleState, e: Entry) {
    c.entries.push(e);
    let over = c.entries.len().saturating_sub(MAX_ENTRIES);
    c.entries.drain(..over);
}

fn step_history(c: &mut ConsoleState, older: bool) {
    let n = c.history.len();
    let at = match (c.history_at, older) {
        (None, true) if n > 0 => Some(n - 1),
        (Some(i), true) => Some(i.saturating_sub(1)),
        (Some(i), false) if i + 1 < n => Some(i + 1),
        _ => None,
    };
    c.history_at = at;
    c.input = at
        .and_then(|i| c.history.get(i).cloned())
        .unwrap_or_default();
}

/// Apply one message to the console.
pub fn apply_console(c: &mut ConsoleState, m: ConsoleMsg) {
    match m {
        ConsoleMsg::Edit(text) => c.input = text,
        ConsoleMsg::Submit => {
            let line = std::mem::take(&mut c.input).trim().to_owned();
            c.history_at = None;
            if line.is_empty() {
                return;
            }
            if c.history.last() != Some(&line) {
                c.history.push(line.clone());
            }
            push(
                c,
                Entry {
                    kind: EntryKind::Command,
                    text: line.clone(),
                },
            );
            c.outbox.push(line);
        }
        ConsoleMsg::Older => step_history(c, true),
        ConsoleMsg::Newer => step_history(c, false),
        ConsoleMsg::Clear => c.entries.clear(),
        ConsoleMsg::Line(line) => push(c, entry_of(&line)),
        _ => {}
    }
}

/// The console's part of `update`: the state is changed in place.
pub fn update(state: State, m: ConsoleMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match m {
        ConsoleMsg::StudioSelectTab(_)
        | ConsoleMsg::StudioCloseTab(_)
        | ConsoleMsg::StudioNewTab
        | ConsoleMsg::StudioEditCode(_)
        | ConsoleMsg::StudioOpenScript(_)
        | ConsoleMsg::StudioSaveScript(_)
        | ConsoleMsg::StudioRunScript
        | ConsoleMsg::StudioSetFilter(_)
        | ConsoleMsg::StudioToggleExpanded
        | ConsoleMsg::StudioRefreshList => {
            crate::script_studio_exec::apply_studio_msg(&mut state, m);
        }
        _ => apply_console(&mut state.console, m),
    }
    (state, Vec::new())
}
