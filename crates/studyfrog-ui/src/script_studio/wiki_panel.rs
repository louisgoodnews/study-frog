//! The FrogScript reference panel beside ScriptStudio (STORY-0171): whether it is open, what it is
//! searched for, and which topic it shows. Pure.

use crate::console_ui::{ConsoleMsg, ConsoleState};

/// The panel's state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WikiView {
    /// Whether the panel is shown beside the editor.
    pub open: bool,
    /// The search text; empty shows every topic by category.
    pub query: String,
    /// The topic whose details are shown, as `Category:name` (the name alone is not unique: `tag` is a
    /// keyword and a function).
    pub topic: Option<String>,
}

/// What the panel is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WikiMsg {
    /// Show or hide the panel.
    Toggle,
    /// The search text changed.
    Query(String),
    /// A topic was picked from the list, as `Category:name`.
    Pick(String),
}

/// Apply a panel message to the console's state.
pub fn apply(console: &mut ConsoleState, msg: WikiMsg) {
    let w = &mut console.studio.wiki;
    match msg {
        WikiMsg::Toggle => w.open = !w.open,
        WikiMsg::Query(q) => w.query = q,
        WikiMsg::Pick(name) => w.topic = Some(name),
    }
}

/// The active tab's code with `code` added at its end, on a line of its own.
pub fn append_code(current: &str, code: &str) -> String {
    let mut out = current.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(code);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// What "Run in console" does for a read-only recipe: open a new tab, load the recipe into it, and run
/// it. The user's own tab is left as it was.
pub fn run_in_console(code: &str) -> Vec<ConsoleMsg> {
    vec![
        ConsoleMsg::StudioNewTab,
        ConsoleMsg::StudioEditCode(code.to_owned()),
        ConsoleMsg::StudioRunScript,
    ]
}
