//! Memory game: turning the cards of a stack into a game once they are read. Pure.

use studyfrog_rehearsal::memory::new_memory_session;

use super::memory::{MAX_PAIRS, MISMATCH_MS, MemoryUi, limit_ms, with_memory};
use crate::effect::Effect;
use crate::state::State;

pub(super) fn loaded(
    state: State,
    result: Result<Vec<(String, String)>, String>,
) -> (State, Vec<Effect>) {
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
