//! Performing the item-list effects of the stack browser. Thin: the queries live in the store.

use studyfrog_store::overview::{list_items, status_counts};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::state::Selection;

/// The items directly in the selected stack (or the inbox).
pub fn items(w: &Worker, selection: &Selection) -> Msg {
    let folder = match selection {
        Selection::Stack(f) => Some(f.as_str()),
        Selection::Inbox | Selection::Nothing => None,
    };
    match &w.session {
        Some(s) => Msg::ItemsLoaded(list_items(&s.conn, folder).map_err(|e| e.to_string())),
        None => Msg::ItemsLoaded(Ok(Vec::new())),
    }
}

/// How many items of the selected stack and its sub-stacks are in each status.
pub fn counts(w: &Worker, selection: &Selection) -> Msg {
    let (Selection::Stack(folder), Some(s)) = (selection, &w.session) else {
        return Msg::StatusCountsLoaded(Ok(Vec::new()));
    };
    Msg::StatusCountsLoaded(status_counts(&s.conn, folder).map_err(|e| e.to_string()))
}
