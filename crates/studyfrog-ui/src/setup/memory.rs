//! Memory mode on the rehearse screen: the game's state and its pure update. The engine
//! ([`studyfrog_rehearsal::memory`]) decides the rules; this module keeps the session on screen,
//! starts it from the open stack, and feeds it the clock. No egui, no I/O.

use studyfrog_rehearsal::memory::{
    MemorySession, flip_tile, new_memory_session, tick_memory_timer,
};

use crate::effect::Effect;
use crate::state::{Selection, State};

/// The most pairs in one game (a 4 x 4 grid of cards).
pub const MAX_PAIRS: usize = 8;
/// How long a mismatched pair stays face up, in milliseconds.
pub const MISMATCH_MS: u64 = 900;

/// Messages of the memory game.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryMsg {
    /// Start a game from the open stack.
    Start,
    /// The cards of the stack, as (front, back), or why they could not be read.
    Loaded(Result<Vec<(String, String)>, String>),
    /// A card was pressed.
    Flip(usize),
    /// A frame passed: move the clock on.
    Clock,
    /// The game is closed.
    Close,
}

/// The memory game on screen. `session` is `None` while the cards load or when loading failed.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MemoryUi {
    /// The game in progress or finished.
    pub session: Option<MemorySession>,
    /// The cards are being read.
    pub loading: bool,
    /// Why the game could not start.
    pub error: Option<String>,
    /// The wall clock (Unix ms) when the game started; the clock counts from here.
    pub started_ms: i64,
}

/// Handle a memory message.
pub fn update(state: State, msg: MemoryMsg) -> (State, Vec<Effect>) {
    match msg {
        MemoryMsg::Start => start(state),
        MemoryMsg::Loaded(result) => loaded(state, result),
        MemoryMsg::Flip(idx) => (flip(state, idx), Vec::new()),
        MemoryMsg::Clock => (clock(state), Vec::new()),
        MemoryMsg::Close => (with_memory(state, None), Vec::new()),
    }
}

/// Advance the clock of a running game (called on every tick).
pub fn tick(state: State) -> (State, Vec<Effect>) {
    (clock(state), Vec::new())
}

fn with_memory(state: State, memory: Option<MemoryUi>) -> State {
    State {
        rehearse: crate::setup::Rehearse {
            memory,
            ..state.rehearse.clone()
        },
        ..state
    }
}

fn start(state: State) -> (State, Vec<Effect>) {
    let started_ms = state.wall_ms;
    let folder = match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        Selection::Inbox => None,
        Selection::Nothing => {
            let memory = MemoryUi {
                error: Some("Open a stack first, then start the memory game.".to_owned()),
                started_ms,
                ..MemoryUi::default()
            };
            return (with_memory(state, Some(memory)), Vec::new());
        }
    };
    let memory = MemoryUi {
        loading: true,
        started_ms,
        ..MemoryUi::default()
    };
    (
        with_memory(state, Some(memory)),
        vec![Effect::MemoryDeck(folder)],
    )
}

fn loaded(state: State, result: Result<Vec<(String, String)>, String>) -> (State, Vec<Effect>) {
    let Some(memory) = state.rehearse.memory.clone() else {
        return (state, Vec::new());
    };
    let seed = memory.started_ms.unsigned_abs();
    let memory = match result {
        Err(e) => MemoryUi {
            loading: false,
            error: Some(e),
            ..memory
        },
        Ok(cards) => {
            let usable = cards
                .iter()
                .filter(|(f, b)| !f.trim().is_empty() && !b.trim().is_empty())
                .count();
            if usable < 2 {
                MemoryUi {
                    loading: false,
                    error: Some("This stack needs at least two cards to play.".to_owned()),
                    ..memory
                }
            } else {
                let session =
                    new_memory_session(&cards, usable.min(MAX_PAIRS), seed, MISMATCH_MS, None);
                MemoryUi {
                    session: Some(session),
                    loading: false,
                    error: None,
                    ..memory
                }
            }
        }
    };
    (with_memory(state, Some(memory)), Vec::new())
}

fn flip(state: State, idx: usize) -> State {
    let Some(memory) = state.rehearse.memory.clone() else {
        return state;
    };
    let Some(session) = &memory.session else {
        return state;
    };
    match flip_tile(session, idx) {
        Ok(next) => with_memory(
            state,
            Some(MemoryUi {
                session: Some(next),
                ..memory
            }),
        ),
        Err(_) => state,
    }
}

fn clock(state: State) -> State {
    let Some(memory) = state.rehearse.memory.clone() else {
        return state;
    };
    let Some(session) = &memory.session else {
        return state;
    };
    let now = (state.wall_ms - memory.started_ms).max(0) as u64;
    let next = tick_memory_timer(session, now);
    with_memory(
        state,
        Some(MemoryUi {
            session: Some(next),
            ..memory
        }),
    )
}
