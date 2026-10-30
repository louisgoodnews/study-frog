//! Board commands as data, so a UI can ask for a change and a service can apply and store it.

use studyfrog_core::uid::Uid;

use crate::edit::{
    add_card, add_column, edit_column, move_column, remove_card, remove_column, update_card,
};
use crate::model::{Board, BoardError, MoveMode, card_new};
use crate::moves::{move_card, pull, push, shift};

/// One change to the board.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardOp {
    /// Add a card to the end of a column (the new id is supplied to [`apply`]).
    AddCard {
        /// Column.
        column: Uid,
        /// Title.
        title: String,
    },
    /// Move a card to a column and position.
    Move {
        /// Card.
        card: Uid,
        /// Target column.
        to: Uid,
        /// Position in the column.
        index: usize,
        /// Whether WIP limits are enforced.
        mode: MoveMode,
    },
    /// Push a card to the next column.
    Push(Uid, MoveMode),
    /// Pull a card back to the previous column.
    Pull(Uid, MoveMode),
    /// Move a card up or down inside its column.
    Shift(Uid, i32),
    /// Delete a card.
    Remove(Uid),
    /// Change a card's text, stack link and estimate.
    Update {
        /// Card.
        card: Uid,
        /// New title.
        title: String,
        /// New notes.
        notes: String,
        /// New stack link.
        stack: Option<Uid>,
        /// New estimate.
        points: Option<u32>,
    },
    /// Add a column (the new id is supplied to [`apply`]).
    AddColumn {
        /// Name.
        name: String,
        /// WIP limit.
        wip_limit: Option<u32>,
        /// Position.
        index: usize,
    },
    /// Rename a column and change its limit.
    EditColumn {
        /// Column.
        column: Uid,
        /// New name.
        name: String,
        /// New limit.
        wip_limit: Option<u32>,
    },
    /// Remove a column; its cards go to a neighbour.
    RemoveColumn(Uid),
    /// Move a column.
    MoveColumn(Uid, usize),
}

/// Apply a command. `new_id` is used by the commands that create something.
pub fn apply(b: &Board, op: &BoardOp, new_id: Uid) -> Result<Board, BoardError> {
    match op {
        BoardOp::AddCard { column, title } => {
            add_card(b, *column, card_new(new_id, title.trim()), MoveMode::Strict)
        }
        BoardOp::Move {
            card,
            to,
            index,
            mode,
        } => move_card(b, *card, *to, *index, *mode),
        BoardOp::Push(card, mode) => push(b, *card, *mode),
        BoardOp::Pull(card, mode) => pull(b, *card, *mode),
        BoardOp::Shift(card, delta) => shift(b, *card, *delta),
        BoardOp::Remove(card) => remove_card(b, *card),
        BoardOp::Update {
            card,
            title,
            notes,
            stack,
            points,
        } => update_card(b, *card, |c| {
            (c.title, c.notes, c.stack, c.points) =
                (title.trim().to_owned(), notes.clone(), *stack, *points);
        }),
        BoardOp::AddColumn {
            name,
            wip_limit,
            index,
        } => add_column(b, new_id, name.trim(), *wip_limit, *index),
        BoardOp::EditColumn {
            column,
            name,
            wip_limit,
        } => edit_column(b, *column, name.trim(), *wip_limit),
        BoardOp::RemoveColumn(column) => remove_column(b, *column),
        BoardOp::MoveColumn(column, index) => move_column(b, *column, *index),
    }
}
