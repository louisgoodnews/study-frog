//! The board screen: state and update logic. Pure; every change is a [`BoardOp`] that the executor
//! applies through the service, so the rules (WIP limits, ordering) live in `studyfrog-board`.

use studyfrog_board::model::{Board, MoveMode, find_card};
use studyfrog_board::ops::BoardOp;
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

pub use crate::kanban_msg::{BoardMsg, CardEdit};

/// What the board screen shows.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Kanban {
    /// The stored board, once loaded.
    pub board: Option<Board>,
    /// Moves may exceed WIP limits.
    pub force: bool,
    /// Title for a card about to be added.
    pub new_title: String,
    /// The card open for editing.
    pub edit: Option<CardEdit>,
}

fn mode(k: &Kanban) -> MoveMode {
    if k.force {
        MoveMode::Force
    } else {
        MoveMode::Strict
    }
}

fn with(state: State, kanban: Kanban) -> (State, Vec<Effect>) {
    (State { kanban, ..state }, Vec::new())
}

fn command(state: State, op: BoardOp) -> (State, Vec<Effect>) {
    (state, vec![Effect::BoardCommand(op)])
}

fn loaded(state: State, result: Result<Board, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(board) => {
            let edit = state
                .kanban
                .edit
                .clone()
                .filter(|e| find_card(&board, e.card).is_some());
            with(
                state.clone(),
                Kanban {
                    board: Some(board),
                    edit,
                    ..state.kanban
                },
            )
        }
        Err(text) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text,
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

fn add(state: State) -> (State, Vec<Effect>) {
    let title = state.kanban.new_title.trim().to_owned();
    let first = state
        .kanban
        .board
        .as_ref()
        .and_then(|b| b.columns.first().map(|c| c.id));
    match first {
        Some(column) if !title.is_empty() => {
            let kanban = Kanban {
                new_title: String::new(),
                ..state.kanban.clone()
            };
            let op = BoardOp::AddCard { column, title };
            command(State { kanban, ..state }, op)
        }
        _ => (state, Vec::new()),
    }
}

fn open(state: State, id: Uid) -> (State, Vec<Effect>) {
    let card = state
        .kanban
        .board
        .as_ref()
        .and_then(|b| b.columns.iter().flat_map(|c| &c.cards).find(|c| c.id == id));
    let edit = card.map(|c| CardEdit {
        card: c.id,
        title: c.title.clone(),
        notes: c.notes.clone(),
        stack: c.stack,
        points: c.points.map(|p| p.to_string()).unwrap_or_default(),
    });
    with(
        state.clone(),
        Kanban {
            edit,
            ..state.kanban
        },
    )
}

fn save_card(state: State) -> (State, Vec<Effect>) {
    let Some(e) = state.kanban.edit.clone() else {
        return (state, Vec::new());
    };
    let points = match e.points.trim() {
        "" => Ok(None),
        text => text.parse::<u32>().map(Some),
    };
    let Ok(points) = points else {
        let notice = Some(Notice {
            kind: NoticeKind::Error,
            text: "The estimate must be a whole number.".to_owned(),
        });
        return (State { notice, ..state }, Vec::new());
    };
    let op = BoardOp::Update {
        card: e.card,
        title: e.title,
        notes: e.notes,
        stack: e.stack,
        points,
    };
    let kanban = Kanban {
        edit: None,
        ..state.kanban.clone()
    };
    command(State { kanban, ..state }, op)
}

fn edit_field(state: State, f: impl FnOnce(CardEdit) -> CardEdit) -> (State, Vec<Effect>) {
    let edit = state.kanban.edit.clone().map(f);
    with(
        state.clone(),
        Kanban {
            edit,
            ..state.kanban
        },
    )
}

/// Handle the board messages.
pub fn update(state: State, msg: BoardMsg) -> (State, Vec<Effect>) {
    let m = mode(&state.kanban);
    match msg {
        BoardMsg::Loaded(result) => loaded(state, result),
        BoardMsg::Force(force) => with(
            state.clone(),
            Kanban {
                force,
                ..state.kanban
            },
        ),
        BoardMsg::NewTitle(new_title) => with(
            state.clone(),
            Kanban {
                new_title,
                ..state.kanban
            },
        ),
        BoardMsg::Add => add(state),
        BoardMsg::Move { card, to, index } => command(
            state,
            BoardOp::Move {
                card,
                to,
                index,
                mode: m,
            },
        ),
        BoardMsg::Push(id) => command(state, BoardOp::Push(id, m)),
        BoardMsg::Pull(id) => command(state, BoardOp::Pull(id, m)),
        BoardMsg::Shift(id, by) => command(state, BoardOp::Shift(id, by)),
        BoardMsg::Remove(id) => command(state, BoardOp::Remove(id)),
        BoardMsg::Open(id) => open(state, id),
        BoardMsg::CloseCard => with(
            state.clone(),
            Kanban {
                edit: None,
                ..state.kanban
            },
        ),
        BoardMsg::EditTitle(title) => edit_field(state, |e| CardEdit { title, ..e }),
        BoardMsg::EditNotes(notes) => edit_field(state, |e| CardEdit { notes, ..e }),
        BoardMsg::EditStack(stack) => edit_field(state, |e| CardEdit { stack, ..e }),
        BoardMsg::EditPoints(points) => edit_field(state, |e| CardEdit { points, ..e }),
        BoardMsg::SaveCard => save_card(state),
    }
}
