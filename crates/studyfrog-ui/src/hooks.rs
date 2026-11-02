//! What runs after every update: the notification list, the timing of the item page and the rows of the library.

use crate::effect::Effect;
use crate::state::{Notice, State};

/// Follow a state change with what it makes necessary. `reload` says the message changed what the library shows.
pub fn after(state: State, before: &Option<Notice>, reload: bool) -> (State, Vec<Effect>) {
    let (state, mut effects) = crate::notify::track(before, state);
    let (state, recorded) = crate::view_time::sync(state);
    effects.extend(recorded);
    let (state, rows) = crate::library_ui::sync(state, reload);
    effects.extend(rows);
    let (state, summary) = crate::stack_view_ui::sync(state);
    effects.extend(summary);
    (state, effects)
}
