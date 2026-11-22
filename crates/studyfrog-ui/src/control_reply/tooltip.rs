//! The answer of `help tooltip NAME`: the explanation of a control, with its shortcut in the text.

use serde_json::{Value, json};

use crate::view::tips::{explain_of, for_name, names};

/// The tooltip of the control called `name`, or an error that lists the names it knows.
pub fn tooltip_json(name: &str) -> Result<Value, String> {
    match explain_of(name) {
        Some(explain) => Ok(json!({
            "name": name,
            "text": for_name(name),
            "explanation": explain,
        })),
        None => Err(format!(
            "no tooltip named {name:?}; the names are: {}",
            names().join(", ")
        )),
    }
}
