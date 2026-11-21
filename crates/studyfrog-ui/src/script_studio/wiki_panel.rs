//! The FrogScript reference panel beside ScriptStudio (STORY-0171): whether it is open, what it is
//! searched for, and which topic it shows. Pure.

use crate::console_ui::ConsoleState;

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
