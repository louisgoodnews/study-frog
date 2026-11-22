//! The Console screen (STORY-0133): control commands and `script run|list` typed by hand, answers,
//! script output and lifecycle events as they arrive. Pure: lines go out through `outbox`, which the
//! window's glue (`console_link`) sends to its own control protocol, and come back as `Line`.

pub mod lines;

use crate::console_ui::lines::entry_of;
use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

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
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// Seconds on the app's clock when it was made (the console log shows it).
    pub at: f64,
    /// What it is.
    pub kind: EntryKind,
    /// The text shown.
    pub text: String,
}

/// The console's state.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConsoleState {
    /// The line being typed.
    pub input: String,
    /// The transcript, oldest first.
    pub entries: Vec<Entry>,
    /// The text the output is filtered by (empty: everything).
    pub search: String,
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
    /// The FrogScript reference panel: show or hide, search, pick a topic.
    Wiki(crate::script_studio::wiki_panel::WikiMsg),
    /// ScriptStudio: refresh script file list.
    StudioRefreshList,
    /// The output search text changed.
    Search(String),
    /// Write the transcript to the vault's log file.
    ExportLog,
    /// The transcript was written to this path, or not.
    Exported(Result<String, String>),
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
pub fn apply_console(c: &mut ConsoleState, m: ConsoleMsg, now: f64) {
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
                    at: now,
                    kind: EntryKind::Command,
                    text: line.clone(),
                },
            );
            c.outbox.push(line);
        }
        ConsoleMsg::Older => step_history(c, true),
        ConsoleMsg::Newer => step_history(c, false),
        ConsoleMsg::Clear => c.entries.clear(),
        ConsoleMsg::Line(line) => push(c, entry_of(&line, now)),
        ConsoleMsg::Search(query) => c.search = query,
        _ => {}
    }
}

/// The console's part of `update`: the state is changed in place.
pub fn update(state: State, m: ConsoleMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let now = state.now;
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
        ConsoleMsg::Wiki(w) => crate::script_studio::wiki_panel::apply(&mut state.console, w),
        ConsoleMsg::ExportLog => {
            let text = crate::console_ui::lines::export_text(&state.console);
            return (state, vec![Effect::ExportConsole(text)]);
        }
        ConsoleMsg::Exported(Ok(path)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Info,
                text: format!("The console log was written to {path}."),
            });
        }
        ConsoleMsg::Exported(Err(why)) => {
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not write the console log: {why}"),
            });
        }
        _ => apply_console(&mut state.console, m, now),
    }
    (state, Vec::new())
}
