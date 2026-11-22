//! Memory mode on the rehearse screen: the game's state and its pure update. The engine
//! ([`studyfrog_rehearsal::memory`]) decides the rules; this module keeps the session on screen,
//! starts it from the open stack with the chosen timing, and feeds it the clock. No egui, no I/O.

use studyfrog_core::settings::{memory_best_of, with_memory_best};
use studyfrog_rehearsal::memory::{MemoryOutcome, MemorySession, flip_tile, tick_memory_timer};
use studyfrog_rehearsal::memory_summary::{MemoryTimer, victory_summary};

use crate::effect::Effect;
use crate::modal_types::Modal;
use crate::setup::stack_source::{StackSource, stack_source};
use crate::state::{Selection, State};

/// The most pairs in one game (a 4 x 4 grid of cards).
pub const MAX_PAIRS: usize = 8;
/// How long a mismatched pair stays face up, in milliseconds.
pub const MISMATCH_MS: u64 = 900;

/// Messages of the memory game.
#[derive(Debug, Clone, PartialEq)]
pub enum MemoryMsg {
    /// Start a game from the stack ticked in the What step, or else the open one, with the chosen timing.
    Start,
    /// Start a game from this stack (the stack page's action).
    StartStack(String),
    /// Play the same stack again (the end of a game).
    Again,
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
    /// The stack this game is played with (empty for the inbox), for its record.
    pub stack: String,
    /// The best time before this game started, in milliseconds.
    pub prior_best_ms: Option<u64>,
    /// Whether this game's time has been counted towards the best.
    pub recorded: bool,
}

/// Handle a memory message.
pub fn update(state: State, msg: MemoryMsg) -> (State, Vec<Effect>) {
    match msg {
        MemoryMsg::Start => {
            let source = stack_source(&state);
            start(state, source)
        }
        MemoryMsg::StartStack(folder) => {
            let state = State {
                browser: crate::state::Browser {
                    selection: Selection::Stack(folder.clone()),
                    ..state.browser.clone()
                },
                ..state
            };
            start(state, StackSource::Stack(folder))
        }
        MemoryMsg::Again => {
            let source = match state.rehearse.memory.as_ref().map(|m| m.stack.clone()) {
                Some(stack) if stack.is_empty() => StackSource::Inbox,
                Some(stack) => StackSource::Stack(stack),
                None => StackSource::Nothing,
            };
            start(state, source)
        }
        MemoryMsg::Mode(timer) => (with_timer(state, timer), Vec::new()),
        MemoryMsg::Loaded(result) => super::memory_deck::loaded(state, result),
        MemoryMsg::Flip(idx) => flip(state, idx),
        MemoryMsg::Clock => clock(state),
        MemoryMsg::Close => (with_memory(state, None), Vec::new()),
    }
}

/// Advance the clock of a running game (called on every tick).
pub fn tick(state: State) -> (State, Vec<Effect>) {
    clock(state)
}

/// The engine's time limit for a timing, in milliseconds.
pub fn limit_ms(timer: MemoryTimer) -> Option<u64> {
    match timer {
        MemoryTimer::Stopwatch => None,
        MemoryTimer::TimeAttack(sec) => Some(u64::from(sec) * 1000),
    }
}

pub(super) fn with_memory(state: State, memory: Option<MemoryUi>) -> State {
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

fn start(state: State, source: StackSource) -> (State, Vec<Effect>) {
    let started_ms = state.wall_ms;
    let timer = state.rehearse.memory_timer;
    let vault = state.app.vault.clone().unwrap_or_default();
    let folder = match source {
        StackSource::Stack(f) => Some(f),
        StackSource::Inbox => Some(String::new()),
        StackSource::Nothing => None,
    };
    let stack = folder.clone().unwrap_or_default();
    let prior_best_ms = memory_best_of(&state.app, &vault, &stack);
    let base = MemoryUi {
        started_ms,
        timer,
        stack,
        prior_best_ms,
        ..MemoryUi::default()
    };
    let Some(folder) = folder else {
        let memory = MemoryUi {
            error: Some(
                "Tick a stack in the What step, or open one in the library, then start the memory game."
                    .to_owned(),
            ),
            ..base
        };
        return (with_memory(state, Some(memory)), Vec::new());
    };
    let memory = MemoryUi {
        loading: true,
        ..base
    };
    let stack_arg = if folder.is_empty() {
        None
    } else {
        Some(folder)
    };
    (
        with_memory(state, Some(memory)),
        vec![Effect::MemoryDeck(stack_arg)],
    )
}

fn flip(state: State, idx: usize) -> (State, Vec<Effect>) {
    let Some(memory) = state.rehearse.memory.clone() else {
        return (state, Vec::new());
    };
    let Some(session) = &memory.session else {
        return (state, Vec::new());
    };
    match flip_tile(session, idx) {
        Ok(next) => put(state, memory, next),
        Err(_) => (state, Vec::new()),
    }
}

fn clock(state: State) -> (State, Vec<Effect>) {
    let Some(memory) = state.rehearse.memory.clone() else {
        return (state, Vec::new());
    };
    let Some(session) = &memory.session else {
        return (state, Vec::new());
    };
    let now = (state.wall_ms - memory.started_ms).max(0) as u64;
    let next = tick_memory_timer(session, now);
    put(state, memory, next)
}

/// Store the game's new session. A completed game's time becomes the stack's best if it beats the
/// stored one, and the settings are saved.
fn put(state: State, memory: MemoryUi, session: MemorySession) -> (State, Vec<Effect>) {
    let mut memory = MemoryUi {
        session: Some(session.clone()),
        ..memory
    };
    let mut app = state.app.clone();
    let mut effects = Vec::new();
    let mut just_completed = false;
    if let Some(MemoryOutcome::Completed { elapsed_ms }) = session.outcome
        && !memory.recorded
    {
        memory.recorded = true;
        just_completed = true;
        let vault = state.app.vault.clone().unwrap_or_default();
        app = with_memory_best(&state.app, &vault, &memory.stack, elapsed_ms);
        effects.push(Effect::SaveAppSettings(app.clone()));
    }
    // The summary opens once, when the game is completed; later frames must not reopen it.
    let modal = just_completed
        .then(|| victory_summary(&session, memory.timer, memory.prior_best_ms))
        .flatten()
        .map(|v| Modal::MemorySummary(Box::new(v), memory.prior_best_ms));
    let next = State {
        app,
        modal: modal.or(state.modal.clone()),
        rehearse: crate::setup::Rehearse {
            memory: Some(memory),
            ..state.rehearse.clone()
        },
        ..state
    };
    (next, effects)
}
