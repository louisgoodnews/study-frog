//! The answers of the screens that had only `nav`: the achievements, the graph, the assistant, the
//! settings sections, match mode, the folders, the PDF viewer, and a deleted stack. Pure.

use serde_json::{Value, json};

use crate::achievements::visible;
use crate::agent_types::selected_provider;
use crate::app_settings_ui::{SHOWN, current_tab, tab_key, tab_label};
use crate::control_cmd::screens::{FoldersCmd, ScreenCmd};
use crate::control_reply::pdf::pdf_json;
use crate::state::State;

/// Folder entries a `folders` answer lists at most.
const MAX_FOLDERS: usize = 500;

fn achievements_json(state: &State) -> Value {
    let entries = state.achievements.as_ref().map(|r| r.entries.as_slice());
    let shown: Vec<Value> = entries
        .map(|all| visible(all, &state.achievements_filter))
        .unwrap_or_default()
        .iter()
        .map(|e| {
            json!({
                "rule": e.rule, "title": e.title, "description": e.description,
                "stack": e.stack, "key": e.key, "metric": format!("{:?}", e.metric).to_lowercase(),
                "value": e.value, "threshold": e.threshold, "unlocked": e.unlocked.is_some(),
            })
        })
        .collect();
    json!({
        "loaded": entries.is_some(), "count": shown.len(),
        "filter": format!("{:?}", state.achievements_filter).to_lowercase(), "entries": shown,
    })
}

fn graph_json(state: &State) -> Result<Value, String> {
    let g = state
        .graph
        .as_ref()
        .ok_or("the graph is not loaded (open a vault first)")?;
    Ok(json!({"nodes": g.graph.nodes.len(), "edges": g.graph.edges.len()}))
}

fn assistant_json(state: &State) -> Value {
    let a = &state.agent;
    let providers: Vec<Value> = a
        .connections
        .providers
        .iter()
        .map(|p| {
            json!({
                "name": p.name, "protocol": format!("{:?}", p.protocol), "model": p.model,
                "allowed": p.allowed, "with_key": a.connections.with_key.contains(&p.name),
            })
        })
        .collect();
    let last_reply = a
        .chat_messages
        .iter()
        .rev()
        .find(|m| matches!(m.role, studyfrog_agent::provider::Role::Assistant))
        .map(|m| m.content.clone());
    json!({
        "tab": format!("{:?}", a.tab).to_lowercase(),
        "configured": selected_provider(a).is_some(),
        "selected": a.selected, "providers": providers,
        "chat_messages": a.chat_messages.len(), "busy": a.chat_busy,
        "last_reply": last_reply, "error": a.error,
    })
}

fn settings_json(state: &State) -> Value {
    let sections: Vec<Value> = SHOWN
        .iter()
        .map(|t| json!({"key": tab_key(*t), "label": tab_label(*t)}))
        .collect();
    json!({
        "tab": tab_key(current_tab(&state.app)),
        "sections": sections,
        "slow_answer_notice": !state.app.slow_answer_notice_off,
    })
}

fn match_json(state: &State) -> Value {
    let Some(m) = &state.match_state else {
        return json!({"active": false});
    };
    let s = &m.session;
    json!({
        "active": true, "stack": m.stack_path, "tiles": s.tiles.len(),
        "pairs_remaining": s.pairs_remaining, "completed": s.completed,
        "mismatches": s.mismatches, "best_streak": s.best_streak,
        "elapsed_ms": s.elapsed_ms, "high_score": m.high_score,
    })
}

fn folders_json(state: &State) -> Value {
    let entries: Vec<Value> = state
        .folders
        .entries
        .iter()
        .take(MAX_FOLDERS)
        .map(|e| folder_json(state, e))
        .collect();
    json!({
        "count": state.folders.entries.len(),
        "truncated": state.folders.entries.len() > MAX_FOLDERS,
        "entries": entries,
    })
}

fn folder_json(state: &State, e: &crate::folders_types::FolderEntry) -> Value {
    json!({
        "path": e.rel_path, "name": e.name, "folder": e.is_folder, "stack": e.is_stack,
        "depth": e.depth, "expanded": state.folders.expanded.contains(&e.rel_path),
        "selected": state.folders.selected.as_deref() == Some(e.rel_path.as_str()),
    })
}

/// The folder opened: its own entry and the names of its children.
fn folder_opened(state: &State, path: &str) -> Result<Value, String> {
    let path = path.trim_matches('/');
    let entry = state
        .folders
        .entries
        .iter()
        .find(|e| e.rel_path == path)
        .ok_or_else(|| format!("no file or folder {path} in the vault"))?;
    let children: Vec<&str> = state
        .folders
        .entries
        .iter()
        .filter(|e| e.parent == path)
        .map(|e| e.name.as_str())
        .collect();
    let mut answer = folder_json(state, entry);
    if let Some(o) = answer.as_object_mut() {
        o.insert("children".into(), json!(children));
    }
    Ok(answer)
}

/// `pdf open` answers the viewer, or the reason the file did not open.
fn pdf_opened(state: &State) -> Result<Value, String> {
    match state.pdf.as_ref().and_then(|v| v.error.clone()) {
        Some(why) => Err(format!("the PDF did not open: {why}")),
        None => Ok(pdf_json(state)),
    }
}

/// `stack delete` answers with the folder when the stack is gone, else why it is still there.
pub fn stack_deleted(state: &State, folder: &str) -> Result<Value, String> {
    let folder = folder.trim_matches('/');
    let still = state
        .vault
        .as_ref()
        .is_some_and(|v| v.overview.stacks.iter().any(|s| s.folder == folder));
    if still {
        return Err(format!(
            "stack {folder} was not deleted (it is still there)"
        ));
    }
    Ok(json!({"deleted": folder}))
}

/// The answer of a screen command once its work has settled.
pub fn answer(state: &State, cmd: &ScreenCmd) -> Result<Value, String> {
    Ok(match cmd {
        ScreenCmd::Achievements(_) => achievements_json(state),
        ScreenCmd::Graph => graph_json(state)?,
        ScreenCmd::Assistant(_) => assistant_json(state),
        ScreenCmd::Settings(_) => settings_json(state),
        ScreenCmd::Match(_) => match_json(state),
        ScreenCmd::Folders(FoldersCmd::List) => folders_json(state),
        ScreenCmd::Folders(FoldersCmd::Open(path)) => folder_opened(state, path)?,
        ScreenCmd::PdfOpen(_) => pdf_opened(state)?,
    })
}
