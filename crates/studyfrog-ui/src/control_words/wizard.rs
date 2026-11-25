//! `wizard`, `wizard status`, `wizard step next|previous|skip|goto <step>`: the first-run setup wizard (STORY-0212).

use studyfrog_config::wizard::Step;

use crate::control_cmd::Command;
use crate::control_cmd::wizard::{StepOp, WizardCmd};

/// Parse the words after `wizard`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let cmd = match rest.first().map(String::as_str) {
        None | Some("status") => WizardCmd::Status,
        Some("step") => WizardCmd::Step(step_op(&rest[1..])?),
        Some(other) => return Err(format!("wizard has status and step (not {other})")),
    };
    Ok(Command::Wizard(cmd))
}

fn step_op(words: &[String]) -> Result<StepOp, String> {
    Ok(match words.first().map(String::as_str) {
        Some("next") => StepOp::Next,
        Some("previous" | "back") => StepOp::Previous,
        Some("skip") => StepOp::Skip,
        Some("goto") => StepOp::Goto(step_named(words.get(1).map_or("", String::as_str))?),
        Some(other) => {
            return Err(format!(
                "wizard step has next, previous, skip and goto (not {other})"
            ));
        }
        None => return Err("wizard step needs next, previous, skip or goto <step>".to_owned()),
    })
}

/// The step a name stands for: local_model, remote_provider, minilm or summary.
fn step_named(word: &str) -> Result<Step, String> {
    Ok(match word {
        "local_model" | "local" => Step::LocalModel,
        "remote_provider" | "remote" => Step::RemoteProvider,
        "minilm" => Step::Minilm,
        "summary" => Step::Summary,
        "vault" => return Err("the vault is chosen before the wizard starts".to_owned()),
        other => {
            return Err(format!(
                "unknown step {other} (local_model, remote_provider, minilm, summary)"
            ));
        }
    })
}
