//! Execution bridge and message reducer for ScriptStudio FrogScript workflows.

use std::path::Path;

use studyfrog_script::log_stream::{LogEntry, StreamKind};
use studyfrog_script::val::{ScriptItem, Val};

use crate::console_ui::{ConsoleMsg, Entry, EntryKind};
use crate::script_studio::{
    StudioState, active_tab, close_tab, discover_scripts, new_tab, open_script, save_active_script,
    select_tab, set_active_code,
};
use crate::state::State;

/// Convert items currently in browser into FrogScript ScriptItem entities.
pub fn extract_script_items(state: &State) -> Vec<ScriptItem> {
    state
        .browser
        .items
        .iter()
        .map(|row| {
            let p = Path::new(&row.path);
            let stack = p
                .parent()
                .map(|d| d.to_string_lossy().to_string())
                .unwrap_or_default();
            let kind = match row.kind {
                studyfrog_core::plan::ItemKind::Note => "note",
                studyfrog_core::plan::ItemKind::Card => "card",
                studyfrog_core::plan::ItemKind::Question => "question",
            };
            ScriptItem {
                uid: Some(row.uid.0.to_string()),
                title: row.title.clone(),
                prompt: row.title.clone(),
                answer: String::new(),
                stack,
                kind: kind.to_string(),
                tags: Vec::new(),
                familiarity: 0.75,
                difficulty: "normal".to_string(),
                priority: "normal".to_string(),
                due_day: row.due.map(|ms| ms.0 / 86_400_000),
            }
        })
        .collect()
}

/// Execute active tab content through the FrogScript engine.
pub fn execute_studio_script(
    studio: &StudioState,
    state: &State,
) -> (Result<Val, String>, Vec<LogEntry>) {
    let Some(tab) = active_tab(studio) else {
        return (Err("no active tab to execute".to_string()), Vec::new());
    };

    let items = extract_script_items(state);
    let now_ms = state.wall_ms;

    match studyfrog_script::run_frogscript(&tab.content, items, now_ms) {
        Ok((val, logs)) => (Ok(val), logs.entries),
        Err(err) => (Err(err.to_string()), Vec::new()),
    }
}

fn push_entry(state: &mut State, kind: EntryKind, text: impl Into<String>) {
    state.console.entries.push(Entry {
        kind,
        text: text.into(),
    });
    let over = state
        .console
        .entries
        .len()
        .saturating_sub(crate::console_ui::MAX_ENTRIES);
    state.console.entries.drain(..over);
}

/// Dispatch ScriptStudio messages to update editor tabs, run scripts, or filter output.
pub fn apply_studio_msg(state: &mut State, msg: ConsoleMsg) {
    match msg {
        ConsoleMsg::StudioSelectTab(i) => select_tab(&mut state.console.studio, i),
        ConsoleMsg::StudioCloseTab(i) => close_tab(&mut state.console.studio, i),
        ConsoleMsg::StudioNewTab => new_tab(&mut state.console.studio, None),
        ConsoleMsg::StudioEditCode(code) => set_active_code(&mut state.console.studio, code),
        ConsoleMsg::StudioSetFilter(f) => state.console.studio.filter = f,
        ConsoleMsg::StudioToggleExpanded => {
            state.console.studio.expanded = !state.console.studio.expanded;
        }
        ConsoleMsg::StudioRefreshList => {
            let vault_root = state.vault.as_ref().map(|v| Path::new(&v.root));
            let app_scripts = crate::script_studio::resolve_app_scripts();
            let app_ref = app_scripts
                .as_deref()
                .unwrap_or_else(|| Path::new("scripts"));
            state.console.studio.available_scripts = discover_scripts(vault_root, app_ref);
        }
        ConsoleMsg::StudioOpenScript(path) => match std::fs::read_to_string(&path) {
            Ok(src) => {
                open_script(&mut state.console.studio, &path, &src);
                push_entry(
                    state,
                    EntryKind::Answer,
                    format!("Opened {}", path.display()),
                );
            }
            Err(e) => push_entry(
                state,
                EntryKind::Error,
                format!("Failed to open {}: {e}", path.display()),
            ),
        },
        ConsoleMsg::StudioSaveScript(name_opt) => {
            if let Some(name) = name_opt
                && let Some(tab) = crate::script_studio::active_tab_mut(&mut state.console.studio)
            {
                tab.name = name;
            }
            let vault_root = state.vault.as_ref().map(|v| Path::new(&v.root));
            match save_active_script(&mut state.console.studio, vault_root) {
                Ok(path) => push_entry(
                    state,
                    EntryKind::Answer,
                    format!("Saved script to {}", path.display()),
                ),
                Err(e) => push_entry(
                    state,
                    EntryKind::Error,
                    format!("Failed to save script: {e}"),
                ),
            }
        }
        ConsoleMsg::StudioRunScript => {
            let (result, logs) = execute_studio_script(&state.console.studio, state);
            for entry in logs {
                let kind = match entry.stream {
                    StreamKind::Stdout => EntryKind::Stdout,
                    StreamKind::Stderr => EntryKind::Stderr,
                };
                push_entry(state, kind, format!("[{}] {}", entry.level, entry.message));
            }
            match result {
                Ok(val) => {
                    push_entry(state, EntryKind::Script, format!("Result: {val}"));
                    push_entry(
                        state,
                        EntryKind::Event,
                        "script_finished {\"ok\": true}".to_string(),
                    );
                }
                Err(err) => {
                    push_entry(state, EntryKind::Error, format!("Script error: {err}"));
                    push_entry(
                        state,
                        EntryKind::Event,
                        "script_finished {\"ok\": false}".to_string(),
                    );
                }
            }
        }
        _ => {}
    }
}
