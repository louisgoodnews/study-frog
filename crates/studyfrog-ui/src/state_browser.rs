//! Stack browser state and the new-card draft, as plain data. Re-exported from `state`.

use studyfrog_store::overview::ItemRow;

/// What the library screen shows on its right-hand side.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Selection {
    /// Nothing chosen yet.
    #[default]
    Nothing,
    /// The stack in this vault-relative folder.
    Stack(String),
    /// Items outside every stack.
    Inbox,
}

/// The card being written in the "new card" form.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardDraft {
    /// Optional title (derived from the front when empty).
    pub title: String,
    /// Prompt side.
    pub front: String,
    /// Answer side.
    pub back: String,
}

/// Which field of the new-card form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    /// The optional title.
    Title,
    /// The front.
    Front,
    /// The back.
    Back,
}

/// State of the stack browser.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Browser {
    /// Which stack (or the inbox) is open.
    pub selection: Selection,
    /// Items of the selection, as last loaded.
    pub items: Vec<ItemRow>,
    /// How many items of the stack, with its sub-stacks, are in each status, as last loaded.
    pub status_counts: Vec<(studyfrog_core::status::Status, u32)>,
    /// Title filter.
    pub search: String,
    /// Card being written.
    pub draft: CardDraft,
    /// The card type of the new-card form (`None`: an ordinary front and back).
    pub card_type: Option<String>,
    /// The file to import cards from, as typed.
    pub import_path: String,
}
