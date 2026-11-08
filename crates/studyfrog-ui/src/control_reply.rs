//! The answers of the control protocol: the result of a command once its work has settled, and the
//! JSON Lines envelope around it. Pure.

use serde_json::{Value, json};

use crate::control_cmd::{Command, StackCmd};
use crate::control_help::help_json;
use crate::control_parts::shown_rows;
use crate::control_state::{notices_json, state_json, ulid};
use crate::state::{Notice, NoticeKind, State};

/// Rows a `query` answers with at most.
pub const MAX_QUERY_ROWS: usize = 500;

/// A successful answer, as one line.
pub fn ok_line(id: &Value, result: Value) -> String {
    json!({"id": id, "ok": true, "result": result}).to_string()
}

/// A failed answer, as one line.
pub fn err_line(id: &Value, error: &str) -> String {
    json!({"id": id, "ok": false, "error": error}).to_string()
}

/// A line that is not an answer: a log entry while following the log.
pub fn event_line(event: &str, data: Value) -> String {
    json!({"event": event, "data": data}).to_string()
}

fn query_result(state: &State, text: &str) -> Result<Value, String> {
    if let Some(e) = &state.library.query_error {
        return Err(format!("query error at column {}: {}", e.column, e.message));
    }
    Ok(json!({
        "query": text,
        "count": state.library.shown.len(),
        "rows": shown_rows(state, MAX_QUERY_ROWS),
    }))
}

fn stack_settings(state: &State, folder: &str) -> Result<Value, String> {
    let folder = folder.trim_matches('/');
    let row = state
        .vault
        .as_ref()
        .and_then(|v| v.overview.stacks.iter().find(|s| s.folder == folder))
        .ok_or_else(|| format!("no stack in folder {folder}"))?;
    Ok(json!({
        "uid": ulid(row.uid), "folder": row.folder, "title": row.title,
        "scheduler": row.scheduler.map(|s| format!("{s:?}").to_lowercase()),
        "daily_new_limit": row.daily_new_limit, "daily_review_limit": row.daily_review_limit,
        "subjects": row.subjects,
    }))
}

/// An error the command raised: a new notice that reports a failure.
fn new_error(state: &State, before: &Option<Notice>) -> Option<String> {
    let n = state
        .notice
        .as_ref()
        .filter(|n| Some(*n) != before.as_ref())?;
    (n.kind == NoticeKind::Error).then(|| n.text.clone())
}

/// Why the create dialog refused, when it is still open with a reason.
fn form_error(state: &State) -> Option<String> {
    match &state.modal {
        Some(crate::modal_types::Modal::Create(f)) => f.error.clone(),
        _ => None,
    }
}

/// What `cmd` answers once its work is done; `before` is the notice from before it, so an error it
/// raised makes the answer an error.
pub fn result_for(state: &State, cmd: &Command, before: &Option<Notice>) -> Result<Value, String> {
    let reads_only = matches!(cmd, Command::State | Command::Help | Command::Notices);
    if let Some(e) = new_error(state, before).filter(|_| !reads_only) {
        return Err(e);
    }
    let creates = matches!(
        cmd,
        Command::Item(crate::control_cmd::ItemCmd::Create(_))
            | Command::Stack(StackCmd::Create(..))
    );
    if let Some(e) = form_error(state).filter(|_| creates) {
        return Err(format!("{e} (the dialog is still open; `close` closes it)"));
    }
    match cmd {
        Command::Help => Ok(help_json()),
        Command::Notices => Ok(notices_json(state)),
        Command::Query(text) => query_result(state, text),
        Command::Stack(StackCmd::Settings(folder)) => stack_settings(state, folder),
        _ => Ok(state_json(state)),
    }
}

/// The whole answer line for `cmd`.
pub fn reply_line(id: &Value, state: &State, cmd: &Command, before: &Option<Notice>) -> String {
    match result_for(state, cmd, before) {
        Ok(v) => ok_line(id, v),
        Err(e) => err_line(id, &e),
    }
}
