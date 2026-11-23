//! `ollama`, `ollama status`, `ollama suggest`: the local Ollama service and the suggestions.

use crate::control_cmd::Command;
use crate::control_cmd::ollama::OllamaCmd;

/// Parse the words after `ollama`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let cmd = match rest.first().map(String::as_str) {
        None | Some("status") => OllamaCmd::Status,
        Some("suggest") => OllamaCmd::Suggest,
        Some(other) => return Err(format!("ollama has status and suggest (not {other})")),
    };
    Ok(Command::Ollama(cmd))
}
