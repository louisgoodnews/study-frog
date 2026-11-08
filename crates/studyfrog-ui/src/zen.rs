//! Zen mode (STORY-0116, `docs/specs/ui-colour-and-type.md`): the sidebar and the status bar step aside so a run or a
//! note has the window to itself. Pure: whether it is on.

use crate::state::{Screen, State};

/// Whether a rehearsal or exam run is on screen and not yet over.
fn in_run(state: &State) -> bool {
    state.screen == Screen::Rehearse
        && state
            .rehearse
            .run
            .as_ref()
            .is_some_and(|r| !crate::run_update::is_over(r))
}

/// What the setting alone asks for: on during a run when "Zen mode during runs" is chosen.
pub fn automatic(state: &State) -> bool {
    state.app.appearance.zen_in_runs && in_run(state)
}

/// Whether Zen mode is on: the user's own choice (the shortcut or the button), else the automatic one.
pub fn active(state: &State) -> bool {
    state.chrome.zen.unwrap_or_else(|| automatic(state))
}
