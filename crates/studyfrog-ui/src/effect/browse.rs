//! Stack browser, search and item lookup effects.

/// Stack browser, search and item lookup effects.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectBrowse {
    /// List the items of a stack (or the inbox).
    LoadItems(crate::state::Selection),
    /// Count the items of a stack and its sub-stacks by status.
    LoadStatusCounts(crate::state::Selection),
    /// Read every item of the vault as a row of the library list.
    LoadLibrary,
    /// Read the cards of a stack (the inbox when `None`) for a memory game.
    MemoryDeck(Option<String>),
    /// Read the case studies of a stack (the inbox when `None`).
    LoadCases(Option<String>),
    /// Find what the item page shows besides the file: the items that link to it (by title) and its familiarity.
    LoadBacklinks(studyfrog_core::uid::Uid, String),
    /// Find items whose title contains this text (the global search).
    SearchItems(String),
}
