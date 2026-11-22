//! The plan of `search`: the top-bar search text, and with `--focus` the search field's keyboard focus.
//! Pure.

use crate::chrome::ChromeMsg;
use crate::control_plan::Plan;
use crate::msg::Msg;

/// Set the search text (unless only the focus was asked for), and ask for the keyboard first with `focus`.
pub fn plan(text: &str, focus: bool) -> Result<Plan, String> {
    let mut msgs = Vec::new();
    if focus {
        msgs.push(Msg::Chrome(ChromeMsg::SearchFocus));
    }
    if !focus || !text.is_empty() {
        msgs.push(Msg::Chrome(ChromeMsg::SearchEdited(text.to_owned())));
    }
    Ok(vec![msgs])
}
