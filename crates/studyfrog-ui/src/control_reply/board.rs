//! The answers of the board commands: the columns with their cards (a card's status is its column),
//! one column, and the card a new one became. Pure.

use serde_json::{Value, json};
use studyfrog_board::model::{Board, Column};

use crate::control_cmd::board::BoardCmd;
use crate::control_state::ulid;
use crate::state::State;

/// One column with its cards in order; each card's `status` is this column's name.
pub fn column_json(c: &Column) -> Value {
    let cards: Vec<Value> = c
        .cards
        .iter()
        .map(|card| {
            json!({
                "uid": ulid(card.id), "title": card.title, "notes": card.notes,
                "stack": card.stack.map(ulid), "points": card.points, "status": c.name,
            })
        })
        .collect();
    json!({"uid": ulid(c.id), "name": c.name, "wip_limit": c.wip_limit, "cards": cards})
}

fn board_of(state: &State) -> Result<&Board, String> {
    state
        .kanban
        .board
        .as_ref()
        .ok_or_else(|| "the board is not loaded (open a vault first)".to_owned())
}

fn find_column<'a>(board: &'a Board, name: &str) -> Result<&'a Column, String> {
    board
        .columns
        .iter()
        .find(|c| c.name.eq_ignore_ascii_case(name.trim()))
        .ok_or_else(|| format!("no column named {}", name.trim()))
}

/// The answer to a `board` command once the board has loaded.
pub fn answer(state: &State, cmd: &BoardCmd) -> Result<Value, String> {
    let board = board_of(state)?;
    match cmd {
        BoardCmd::List => {
            let columns: Vec<Value> = board.columns.iter().map(column_json).collect();
            Ok(json!({"columns": columns}))
        }
        BoardCmd::Column(name) => Ok(column_json(find_column(board, name)?)),
        BoardCmd::Add { title, column } => added(board, title, column.as_deref()),
    }
}

/// The card titled `title` in its column (the first column when none was named).
fn added(board: &Board, title: &str, column: Option<&str>) -> Result<Value, String> {
    let col = match column {
        Some(name) => find_column(board, name)?,
        None => board.columns.first().ok_or("the board has no column")?,
    };
    let card = col
        .cards
        .iter()
        .find(|c| c.title == title.trim())
        .ok_or_else(|| "the card was not added".to_owned())?;
    Ok(json!({
        "column": col.name,
        "status": col.name,
        "card": {"uid": ulid(card.id), "title": card.title, "notes": card.notes, "points": card.points},
    }))
}
