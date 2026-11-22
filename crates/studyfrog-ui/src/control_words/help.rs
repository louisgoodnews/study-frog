//! `help` (the table of commands) and `help tooltip NAME` (the explanation of a control, with its shortcut).

use crate::control_cmd::Command;

/// Parse the words after `help`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    match rest.first().map(String::as_str) {
        None => Ok(Command::Help(None)),
        Some("tooltip") => {
            let name = rest[1..].join(" ");
            if name.is_empty() {
                return Err("help tooltip needs the name of the control".to_owned());
            }
            Ok(Command::Help(Some(name)))
        }
        Some(other) => Err(format!("help has one subject: tooltip NAME (not {other})")),
    }
}
