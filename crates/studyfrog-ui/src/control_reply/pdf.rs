//! The answer of `pdf find`: whether a PDF is open, what the find field holds and the pages it was found on.

use serde_json::{Value, json};

use crate::state::State;

/// The open PDF's find state; `open` is false when no PDF is shown.
pub fn pdf_json(state: &State) -> Value {
    match &state.pdf {
        Some(v) => json!({
            "open": true,
            "path": v.path,
            "query": v.query,
            "found_pages": v.found.iter().map(|p| p + 1).collect::<Vec<_>>(),
            "pages_without_text": v.scanned.iter().map(|p| p + 1).collect::<Vec<_>>(),
        }),
        None => json!({"open": false}),
    }
}
