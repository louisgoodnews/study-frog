//! Leaving StudyFrog from its own red button. Unsaved edits in the editor are saved first, then the window closes. Pure.

use crate::chrome::Chrome;
use crate::effect::Effect;
use crate::state::State;

/// What the user did about leaving.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuitMsg {
    /// The Exit button was pressed.
    Ask,
}

/// Handle the Exit button: close at once, or after the editor's edits are on disk.
pub fn update(state: State, msg: QuitMsg) -> (State, Vec<Effect>) {
    let QuitMsg::Ask = msg;
    let then = crate::msg::Msg::Chrome(crate::chrome::ChromeMsg::Quit(QuitMsg::Ask));
    match crate::editor_save::flush_before(state, then) {
        (state, Some(effects)) => (state, effects),
        (state, None) => {
            let chrome = Chrome {
                quit: true,
                ..state.chrome.clone()
            };
            (State { chrome, ..state }, Vec::new())
        }
    }
}
