//! The keyboard focus of the search fields as the protocol sees it: a request (`search --focus`) and the
//! name of the field that took it.

use crate::chrome::ChromeMsg;
use crate::effect::Effect;
use crate::state::State;

/// Record a focus request, or the field that took the keyboard (and clear the request).
pub fn update(mut state: State, msg: ChromeMsg) -> (State, Vec<Effect>) {
    match msg {
        ChromeMsg::SearchFocus => state.chrome.search_focus = true,
        ChromeMsg::SearchFocusTaken(name) => {
            state.chrome.search_focus = false;
            state.chrome.focused = Some(name);
        }
        _ => {}
    }
    (state, Vec::new())
}
