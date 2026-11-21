//! Memory mode on the rehearse screen: the game's state and its pure update. The engine
//! ([`studyfrog_rehearsal::memory`]) decides the rules; this module keeps the session on screen,
//! starts it from the open stack with the chosen timing, and feeds it the clock. No egui, no I/O.

use studyfrog_rehearsal::memory::{
    MemoryOutcome, MemorySession, flip_tile, new_memory_session, tick_memory_timer,
};
use studyfrog_rehearsal::memory_summary::MemoryTimer;

use crate::effect::Effect;
use crate::state::{Selection, State};

/// The most pairs in one game (a 4 x 4 grid of cards).
pub const MAX_PAIRS: usize = 8;
/// How long a mismatched pair stays face up, in milliseconds.
pub const MISMATCH_MS: u64 = 900;

/// Messages of the memory game.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryMsg {
    /// Start a game from the open stack, with the chosen timing.
    Start,
    /// Start a game from this stack (the stack page's action).
    StartStack(String),
    /// Choose the timing of the next game.
    Mode(MemoryTimer),
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
    /// The timing of this game.
    pub timer: MemoryTimer,
    /// The best time before this game started, in milliseconds.
    pub prior_best_ms: Option<u64>,
    /// Whether this game's time has been counted towards the best.
    pub recorded: bool,
}

/// Handle a memory message.
pub fn update(state: State, msg: MemoryMsg) -> (State, Vec<Effect>) {
    match msg {
        MemoryMsg::Start => start(state),
        MemoryMsg::StartStack(folder) => start(State {
            browser: crate::state::Browser {
                selection: Selection::Stack(folder),
                ..state.browser.clone()
            },
            ..state
        }),
        MemoryMsg::Mode(timer) => (with_timer(state, timer), Vec::new()),
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

/// The engine's time limit for a timing, in milliseconds.
pub fn limit_ms(timer: MemoryTimer) -> Option<u64> {
    match timer {
        MemoryTimer::Stopwatch => None,
        MemoryTimer::TimeAttack(sec) => Some(u64::from(sec) * 1000),
    }
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

fn with_timer(state: State, timer: MemoryTimer) -> State {
    State {
        rehearse: crate::setup::Rehearse {
            memory_timer: timer,
            ..state.rehearse.clone()
        },
        ..state
    }
}

fn start(state: State) -> (State, Vec<Effect>) {
    let started_ms = state.wall_ms;
    let timer = state.rehearse.memory_timer;
    let prior_best_ms = state.rehearse.memory_best_ms;
    let base = MemoryUi {
        started_ms,
        timer,
        prior_best_ms,
        ..MemoryUi::default()
    };
    let folder = match &state.browser.selection {
        Selection::Stack(f) => Some(f.clone()),
        Selection::Inbox => None,
        Selection::Nothing => {
            let memory = MemoryUi {
                error: Some("Open a stack first, then start the memory game.".to_owned()),
                ..base
            };
            return (with_memory(state, Some(memory)), Vec::new());
        }
    };
    let memory = MemoryUi {
        loading: true,
        ..base
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
                let session = new_memory_session(
                    &cards,
                    usable.min(MAX_PAIRS),
                    seed,
                    MISMATCH_MS,
                    limit_ms(memory.timer),
                );
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
        Ok(next) => put(state, memory, next),
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
    put(state, memory, next)
}

/// Store the game's new session, and count a completed game's time towards the session's best.
fn put(state: State, memory: MemoryUi, session: MemorySession) -> State {
    let mut memory = MemoryUi {
        session: Some(session.clone()),
        ..memory
    };
    let mut best = state.rehearse.memory_best_ms;
    if let Some(MemoryOutcome::Completed { elapsed_ms }) = session.outcome
        && !memory.recorded
    {
        best = Some(best.map_or(elapsed_ms, |b| b.min(elapsed_ms)));
        memory.recorded = true;
    }
    State {
        rehearse: crate::setup::Rehearse {
            memory: Some(memory),
            memory_best_ms: best,
            ..state.rehearse.clone()
        },
        ..state
    }
}
