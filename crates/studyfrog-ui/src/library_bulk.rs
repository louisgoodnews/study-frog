//! Selecting rows of the library, changing them together, undoing that, and the saved filters. Pure.

use studyfrog_config::filters::SavedFilter;
use studyfrog_core::uid::Uid;
use studyfrog_service::bulk::{BulkDone, BulkEdit, Snapshot};

use crate::effect::Effect;
use crate::library_search::SearchMode;
use crate::state::{Notice, NoticeKind, State, VaultInfo};

/// What the library asks the worker to do besides reading the rows.
#[derive(Debug, Clone, PartialEq)]
pub enum LibraryEffect {
    /// Read the saved filters.
    LoadFilters,
    /// Save a filter under its name.
    SaveFilter(SavedFilter),
    /// Remove the filter of that name.
    DeleteFilter(String),
    /// Change the items.
    Bulk(Vec<Uid>, BulkEdit),
    /// Put the files of a bulk edit back.
    Undo(Snapshot),
    /// Read the summary of the stack at this folder, for this generation of the list.
    LoadSummary(String, u64),
    /// Read the goals of the stack with this folder and uid.
    LoadGoals(String, Uid),
    /// Write the goals of a stack.
    SaveGoals(Uid, String),
    /// Give a stack or item a status.
    SetStatus(Uid, Option<studyfrog_core::status::Status>),
}

/// What happened to the selection, the bulk bar or the saved filters.
#[derive(Debug, Clone, PartialEq)]
pub enum BulkMsg {
    /// Select or deselect one row.
    Select(Uid, bool),
    /// Select every row the filter lets through, or none.
    SelectAll(bool),
    /// Change the selected items.
    Apply(BulkEdit),
    /// Show or hide the question before deleting.
    AskDelete(bool),
    /// Change one item, selected or not (the backlog's drag and drop).
    MoveOne(Uid, BulkEdit),
    /// The worker changed the items.
    Done(Result<Box<BulkDone>, String>),
    /// Take the last bulk edit back.
    Undo,
    /// The files were put back.
    Undone(Result<Box<VaultInfo>, String>),
    /// The saved filters were read, with the reason if the file cannot be used.
    FiltersLoaded(Result<(Vec<SavedFilter>, Option<String>), String>),
    /// The name field of "save filter" was edited.
    FilterName(String),
    /// The tag field of the bulk bar was edited.
    TagInput(String),
    /// Save the current query under the typed name.
    SaveFilter,
    /// Remove a saved filter.
    DeleteFilter(String),
    /// Use a saved filter.
    OpenFilter(String),
    /// The filter list stored now.
    FiltersSaved(Result<Vec<SavedFilter>, String>),
}

fn notice(state: &mut State, kind: NoticeKind, text: String) {
    state.notice = Some(Notice { kind, text });
}

fn selected(state: &State) -> Vec<Uid> {
    state.library.selected.iter().copied().collect()
}

fn plural(n: usize) -> String {
    format!("{n} item{}", if n == 1 { "" } else { "s" })
}

fn done(mut state: State, result: Result<Box<BulkDone>, String>) -> (State, Vec<Effect>) {
    let lib = &mut state.library;
    lib.confirm_delete = false;
    lib.stale = true;
    match result {
        Ok(d) => {
            let skipped = if d.skipped.is_empty() {
                String::new()
            } else {
                format!(" ({} left alone: {})", d.skipped.len(), d.skipped[0])
            };
            lib.selected.clear();
            lib.undo = Some(d.undo);
            state.vault = Some(d.info);
            notice(
                &mut state,
                NoticeKind::Info,
                format!("Changed {}{skipped}.", plural(d.changed)),
            );
            let reload = crate::browse::reload(&state.browser.selection);
            (state, reload)
        }
        Err(e) => {
            notice(
                &mut state,
                NoticeKind::Error,
                format!("Could not change the items: {e}"),
            );
            (state, Vec::new())
        }
    }
}

fn open_filter(mut state: State, name: &str) -> State {
    let found = state
        .library
        .filters
        .iter()
        .find(|f| f.name == name)
        .cloned();
    if let Some(f) = found {
        state.library.search.query = f.query;
        state.library.search.mode = SearchMode::Query;
    }
    state
}

fn save_current(mut state: State) -> (State, Vec<Effect>) {
    let name = state.library.filter_name.trim().to_owned();
    let query = match state.library.search.mode {
        SearchMode::Query => state.library.search.query.clone(),
        SearchMode::Standard => {
            let q = crate::library_search::effective(&state).unwrap_or_default();
            studyfrog_core::query_print::print_query(&q)
        }
    };
    if let Some(why) = studyfrog_config::filters::name_problem(&name) {
        notice(&mut state, NoticeKind::Error, format!("Not saved: {why}."));
        return (state, Vec::new());
    }
    let effect = Effect::Library(LibraryEffect::SaveFilter(SavedFilter { name, query }));
    (state, vec![effect])
}

/// Handle a message of the selection, the bulk bar or the saved filters.
pub fn update(state: State, msg: BulkMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    match msg {
        BulkMsg::Select(uid, on) => {
            if on {
                state.library.selected.insert(uid);
            } else {
                state.library.selected.remove(&uid);
            }
        }
        BulkMsg::SelectAll(on) => {
            let lib = &mut state.library;
            lib.selected = if on {
                lib.shown.iter().map(|i| lib.items[*i].uid).collect()
            } else {
                Default::default()
            };
        }
        BulkMsg::Apply(BulkEdit::Delete) if !state.library.confirm_delete => {
            state.library.confirm_delete = true;
        }
        BulkMsg::Apply(edit) if !state.library.selected.is_empty() => {
            let effect = Effect::Library(LibraryEffect::Bulk(selected(&state), edit));
            return (state, vec![effect]);
        }
        BulkMsg::Apply(_) => {}
        BulkMsg::AskDelete(on) => state.library.confirm_delete = on,
        BulkMsg::MoveOne(uid, edit) => {
            return (
                state,
                vec![Effect::Library(LibraryEffect::Bulk(vec![uid], edit))],
            );
        }
        BulkMsg::Done(result) => return done(state, result),
        BulkMsg::Undo => {
            if let Some(snapshot) = state.library.undo.take() {
                return (state, vec![Effect::Library(LibraryEffect::Undo(snapshot))]);
            }
        }
        BulkMsg::Undone(Ok(info)) => {
            state.vault = Some(*info);
            state.library.stale = true;
            notice(&mut state, NoticeKind::Info, "Undone.".to_owned());
            let reload = crate::browse::reload(&state.browser.selection);
            return (state, reload);
        }
        BulkMsg::Undone(Err(e)) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not undo: {e}"),
        ),
        BulkMsg::FiltersLoaded(Ok((list, why))) => {
            state.library.filters = list;
            state.library.filters_problem = why;
        }
        BulkMsg::FiltersLoaded(Err(e)) => notice(&mut state, NoticeKind::Error, e),
        BulkMsg::FilterName(text) => state.library.filter_name = text,
        BulkMsg::TagInput(text) => state.library.tag_input = text,
        BulkMsg::SaveFilter => return save_current(state),
        BulkMsg::DeleteFilter(name) => {
            return (
                state,
                vec![Effect::Library(LibraryEffect::DeleteFilter(name))],
            );
        }
        BulkMsg::OpenFilter(name) => return (open_filter(state, &name), Vec::new()),
        BulkMsg::FiltersSaved(Ok(list)) => {
            state.library.filters = list;
            state.library.filter_name.clear();
        }
        BulkMsg::FiltersSaved(Err(e)) => notice(
            &mut state,
            NoticeKind::Error,
            format!("Could not save the filter: {e}"),
        ),
    }
    (state, Vec::new())
}

/// Keep the selection to rows that still exist after a load.
pub fn prune_selection(state: &mut State) {
    let lib = &mut state.library;
    let have: std::collections::HashSet<Uid> = lib.items.iter().map(|i| i.uid).collect();
    lib.selected.retain(|u| have.contains(u));
}
