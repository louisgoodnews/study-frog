//! The answer of `wizard status` and of `wizard step` (STORY-0212): whether the wizard is open, the step it is on and
//! each step's status (current, done, skipped or pending). Pure.

use serde_json::{Value, json};
use studyfrog_config::wizard::{self as flow, STEPS, Status};

use crate::state::State;
use crate::view::wizard::installed::installed_of;
use crate::view::wizard::types::{flow_of, name_of, title_of};

/// The wizard's state: `open`, the current step and every step with its status.
pub fn status_json(state: &State) -> Result<Value, String> {
    let installed = installed_of(state);
    let current = flow_of(&state.wizard);
    let steps: Vec<Value> = STEPS
        .into_iter()
        .map(|step| {
            json!({
                "step": name_of(step),
                "title": title_of(step),
                "status": status_word(flow::status(step, &installed, &current)),
            })
        })
        .collect();
    studyfrog_log::log_debug!(
        "ui.control",
        "wizard status answered, open={}",
        state.wizard.open
    );
    Ok(json!({
        "open": state.wizard.open,
        "current": name_of(flow::current(&current)),
        "steps": steps,
    }))
}

fn status_word(status: Status) -> &'static str {
    match status {
        Status::Current => "current",
        Status::Done => "done",
        Status::Skipped => "skipped",
        Status::Pending => "pending",
    }
}
