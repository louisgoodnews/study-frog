//! The words of `ui ...`: `ui` or `ui tree`, `ui find LABEL`, `ui click LABEL`.

use crate::control_cmd::Command;
use crate::control_cmd::ui::UiCmd;

/// The label the words after the verb name (the rest of the line, so quoting is optional).
fn label(words: &[String], verb: &str) -> Result<String, String> {
    let text = words.get(1..).unwrap_or_default().join(" ");
    if text.trim().is_empty() {
        return Err(format!("ui {verb} needs the label of a widget"));
    }
    Ok(text)
}

/// Parse the words after `ui`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let cmd = match rest.first().map(String::as_str) {
        None | Some("tree") => UiCmd::Tree,
        Some("find") => UiCmd::Find(label(rest, "find")?),
        Some("click") => UiCmd::Click(label(rest, "click")?),
        Some(other) => {
            return Err(format!(
                "ui has tree, find LABEL and click LABEL (not {other})"
            ));
        }
    };
    Ok(Command::Ui(cmd))
}
