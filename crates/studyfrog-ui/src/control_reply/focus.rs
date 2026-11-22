//! The answer of `search --focus`: whether the request is still pending and which field has the keyboard.

use serde_json::{Value, json};

use crate::state::State;

/// The focus state: `pending` while the search field has not taken the keyboard yet.
pub fn focus_json(state: &State) -> Value {
    json!({
        "pending": state.chrome.search_focus,
        "focused": state.chrome.focused,
    })
}
