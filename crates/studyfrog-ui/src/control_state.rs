//! The state as JSON for the control protocol's `state` command: what is on screen and what an
//! agent needs to decide the next step. Pure.

use serde_json::{Value, json};

use crate::control_parts::{editor_json, library_json, modal_json, rehearse_json};
use crate::state::{NoticeKind, Screen, Selection, State};

/// Recent notifications listed in a snapshot.
pub const RECENT_NOTICES: usize = 10;

/// The name `nav` takes for a screen.
pub fn screen_name(screen: Screen) -> &'static str {
    match screen {
        Screen::Welcome => "welcome",
        Screen::Home => "home",
        Screen::Folders => "folders",
        Screen::Agent => "assistant",
        Screen::Prompts => "prompts",
        Screen::Library => "library",
        Screen::Rehearse => "rehearse",
        Screen::Settings => "settings",
        Screen::Editor => "editor",
        Screen::Graph => "graph",
        Screen::Calendar => "calendar",
        Screen::Board => "board",
        Screen::Achievements => "achievements",
        Screen::Reports => "reports",
        Screen::Diary => "diary",
        Screen::Pdf => "pdf",
        Screen::Help => "help",
    }
}

fn kind_name(kind: NoticeKind) -> &'static str {
    match kind {
        NoticeKind::Info => "info",
        NoticeKind::Error => "error",
    }
}

/// A ULID as text.
pub fn ulid(uid: studyfrog_core::uid::Uid) -> String {
    studyfrog_md::ulid::format_ulid(uid)
}

fn vault_json(state: &State) -> Value {
    let Some(v) = &state.vault else {
        return Value::Null;
    };
    let stacks: Vec<Value> = v
        .overview
        .stacks
        .iter()
        .map(|s| {
            json!({"uid": ulid(s.uid), "key": s.key, "title": s.title, "folder": s.folder,
                   "parent": s.parent, "notes": s.notes, "cards": s.cards, "questions": s.questions})
        })
        .collect();
    json!({
        "root": v.root, "name": v.name, "stacks": stacks,
        "inbox_items": v.overview.inbox_items, "problems": v.overview.problems.len(),
    })
}

fn selection_json(s: &Selection) -> Value {
    match s {
        Selection::Nothing => Value::Null,
        Selection::Inbox => json!("inbox"),
        Selection::Stack(folder) => json!({ "stack": folder }),
    }
}

/// The latest notifications, newest last.
pub fn notices_json(state: &State) -> Value {
    let list = &state.chrome.notifications;
    let from = list.len().saturating_sub(RECENT_NOTICES);
    let recent: Vec<Value> = list[from..]
        .iter()
        .map(|n| json!({"kind": kind_name(n.kind), "text": n.text, "at_ms": n.at_ms, "read": n.read}))
        .collect();
    Value::Array(recent)
}

fn search_json(state: &State) -> Value {
    let c = &state.chrome;
    if c.search.trim().is_empty() {
        return Value::Null;
    }
    let results: Vec<Value> = c
        .results
        .iter()
        .map(|r| json!({"uid": ulid(r.uid), "title": r.title, "path": r.path}))
        .collect();
    json!({"text": c.search, "results": results, "pdf_hits": c.pdf_results.len()})
}

/// The snapshot `state` answers with.
pub fn state_json(state: &State) -> Value {
    let notice = state
        .notice
        .as_ref()
        .map(|n| json!({"kind": kind_name(n.kind), "text": n.text}));
    let toasts: Vec<Value> = state
        .toasts
        .iter()
        .map(|t| json!({"title": t.title, "detail": t.detail}))
        .collect();
    json!({
        "screen": screen_name(state.screen),
        "vault": vault_json(state),
        "opening": state.opening,
        "selection": selection_json(&state.browser.selection),
        "editor": state.editor.as_ref().map(editor_json),
        "modal": state.modal.as_ref().map(modal_json),
        "rehearse": rehearse_json(state),
        "library": library_json(state),
        "search": search_json(state),
        "notice": notice,
        "notice_item": state.notice_link.as_ref().map(|(_, uid)| ulid(*uid)),
        "toasts": toasts,
        "notices": notices_json(state),
        "history": {"back": state.history.at > 0, "forward": state.history.at + 1 < state.history.entries.len()},
    })
}
