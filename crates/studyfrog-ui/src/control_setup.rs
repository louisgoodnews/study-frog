//! `rehearse setup`: the messages that fill in the run setup the way the form would.

use crate::control_cmd::SetupArgs;
use crate::control_plan::resolve_stack;
use crate::msg::Msg;
use crate::setup::{Preset, SetupEdit};
use crate::state::{Screen, State};

/// Go to the rehearse screen and set what `args` names: first the preset (which resets the form;
/// without one the setup starts from Spaced, BUG-0054), then the mode, the limit and the stacks.
pub fn setup_msgs(state: &State, args: &SetupArgs) -> Result<Vec<Msg>, String> {
    let preset = args.preset.unwrap_or(Preset::Spaced);
    let mut msgs = vec![
        Msg::Navigate(Screen::Rehearse),
        Msg::Setup(SetupEdit::Preset(preset)),
    ];
    msgs.extend(args.mode.map(|m| Msg::Setup(SetupEdit::Mode(m))));
    msgs.extend(args.limit.clone().map(|l| Msg::Setup(SetupEdit::Limit(l))));
    for folder in &args.stacks {
        let uid = resolve_stack(state, folder)?;
        msgs.push(Msg::Setup(SetupEdit::Stack(uid, true)));
    }
    Ok(msgs)
}
