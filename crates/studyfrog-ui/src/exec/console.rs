//! Writing the Console's transcript to the vault's log file (STORY-0152). Answers with the console's message.

use std::fs;

use crate::console_ui::ConsoleMsg;
use crate::exec::Worker;
use crate::msg::Msg;

/// The file the transcript is written to, inside the vault.
pub const LOG_FILE: &str = "logs/script_runs.log";

/// Write `text` to the vault's log file, replacing the last export.
pub fn export(w: &mut Worker, text: &str) -> Msg {
    let result = match &w.session {
        None => Err("no vault is open".to_owned()),
        Some(session) => {
            let path = session.root.join(LOG_FILE);
            path.parent()
                .map_or(Ok(()), fs::create_dir_all)
                .and_then(|_| fs::write(&path, text))
                .map(|_| path.display().to_string())
                .map_err(|e| e.to_string())
        }
    };
    Msg::Console(ConsoleMsg::Exported(result))
}
