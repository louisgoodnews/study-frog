//! `board`, `board list`, `board column NAME` and `board add TITLE [--column NAME]`.

use crate::control_cmd::Command;
use crate::control_cmd::board::BoardCmd;
use crate::control_words_items::{flag, split_flags};

/// Parse the words after `board`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let (flags, pos) = split_flags(rest, &[])?;
    let text = |from: usize| pos.get(from..).unwrap_or(&[]).join(" ");
    let cmd = match pos.first().copied() {
        None | Some("list") => BoardCmd::List,
        Some("column") => {
            let name = text(1);
            if name.is_empty() {
                return Err("board column needs a column name".to_owned());
            }
            BoardCmd::Column(name)
        }
        Some("add") => {
            let title = text(1);
            if title.is_empty() {
                return Err("board add needs a title".to_owned());
            }
            BoardCmd::Add {
                title,
                column: flag(&flags, "column").map(str::to_owned),
            }
        }
        Some(other) => {
            return Err(format!(
                "board has list, column NAME and add TITLE (not {other})"
            ));
        }
    };
    Ok(Command::Board(cmd))
}
