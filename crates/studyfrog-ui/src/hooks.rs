//! What runs after every update: the notification list, the timing of the item page and the rows of the library.

use crate::effect::Effect;
use crate::state::{Notice, State};

/// What one update does: the next state and the work it asks for.
pub type Stepped = (State, Vec<Effect>);

/// Run `step` for `msg` and follow it with what the change makes necessary; the place the user was at before is
/// noted for the history.
pub fn around(
    state: State,
    msg: crate::msg::Msg,
    step: fn(State, crate::msg::Msg) -> Stepped,
) -> Stepped {
    let (mut state, mut msg) = (state, msg);
    let mut all = Vec::new();
    loop {
        let before = state.notice.clone();
        let there = crate::history::place_of(&state);
        let reload = crate::library_ui::wants_reload(&msg);
        let visit = crate::editor_leave::visit(&state);
        let (next, effects) = step(state, msg);
        let next = crate::editor_leave::after(visit, next);
        let (next, after) = after(next, &before, reload);
        let next = crate::history::record(next, there);
        let (mut next, last) = crate::appearance_ui::remember_screen(next);
        all.extend([effects, after, last].concat());
        match next.replay.take() {
            Some(again) => (state, msg) = (next, *again),
            None => return (next, all),
        }
    }
}

/// Follow a state change with what it makes necessary. `reload` says the message changed what the library shows.
pub fn after(state: State, before: &Option<Notice>, reload: bool) -> (State, Vec<Effect>) {
    let (state, mut effects) = crate::notify::track(before, state);
    let (state, recorded) = crate::view_time::sync(state);
    effects.extend(recorded);
    let (state, rows) = crate::library_ui::sync(state, reload);
    effects.extend(rows);
    let (state, summary) = crate::stack_view_ui::sync(state);
    effects.extend(summary);
    let (state, goals) = crate::stack_view_ui::sync_goals(state);
    effects.extend(goals);
    let (state, data) = crate::stack_view_ui::sync_data(state);
    effects.extend(data);
    let state = crate::tutorial::sync(state);
    (state, effects)
}
