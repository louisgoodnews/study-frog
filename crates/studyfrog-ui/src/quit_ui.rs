//! Leaving StudyFrog from its own red button: at once, or after asking when the editor holds unsaved changes. Pure.

use crate::chrome::Chrome;
use crate::effect::Effect;
use crate::state::State;

/// What the user did about leaving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitMsg {
    /// The Exit button was pressed.
    Ask,
    /// 'Exit anyway' was chosen.
    Confirm,
    /// 'Keep working' was chosen.
    Cancel,
}

fn with(state: State, quit: bool, confirm_quit: bool) -> (State, Vec<Effect>) {
    let chrome = Chrome {
        quit,
        confirm_quit,
        ..state.chrome.clone()
    };
    (State { chrome, ..state }, Vec::new())
}

/// Handle the Exit button and its question.
pub fn update(state: State, msg: QuitMsg) -> (State, Vec<Effect>) {
    match msg {
        QuitMsg::Ask if crate::editor::has_unsaved_changes(&state) => with(state, false, true),
        QuitMsg::Ask | QuitMsg::Confirm => with(state, true, false),
        QuitMsg::Cancel => with(state, false, false),
    }
}
