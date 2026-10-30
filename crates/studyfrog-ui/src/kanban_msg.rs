//! Messages and the card editor data of the board screen.

use studyfrog_board::model::Board;
use studyfrog_core::uid::Uid;

/// A card being edited.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardEdit {
    /// The card.
    pub card: Uid,
    /// Title as typed.
    pub title: String,
    /// Notes as typed.
    pub notes: String,
    /// Linked stack.
    pub stack: Option<Uid>,
    /// Estimate as typed (empty: none).
    pub points: String,
}

/// Everything the board screen can be asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardMsg {
    /// The board was read, or a command was applied; carries the stored board.
    Loaded(Result<Board, String>),
    /// Allow or forbid exceeding WIP limits.
    Force(bool),
    /// The new-card title changed.
    NewTitle(String),
    /// "Add card" was activated.
    Add,
    /// A card was dropped on a column at a position.
    Move {
        /// The card.
        card: Uid,
        /// Target column.
        to: Uid,
        /// Position in the column.
        index: usize,
    },
    /// Push a card to the next column.
    Push(Uid),
    /// Pull a card to the previous column.
    Pull(Uid),
    /// Move a card up (-1) or down (+1) inside its column.
    Shift(Uid, i32),
    /// Delete a card.
    Remove(Uid),
    /// Open a card for editing.
    Open(Uid),
    /// Close the card editor without saving.
    CloseCard,
    /// The card's title changed.
    EditTitle(String),
    /// The card's notes changed.
    EditNotes(String),
    /// The card's stack link changed.
    EditStack(Option<Uid>),
    /// The card's estimate changed.
    EditPoints(String),
    /// "Save card" was activated.
    SaveCard,
}
