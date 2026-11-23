//! `pdf find TEXT [--file PATH]` sets the find field of the PDF viewer, opening the file first when
//! given; `pdf open PATH` opens a vault PDF and answers its page count and find results.

use crate::control_cmd::Command;
use crate::control_cmd::screens::ScreenCmd;
use crate::control_words_items::{flag, need, split_flags};

/// Parse the words after `pdf`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let (flags, pos) = split_flags(rest, &[])?;
    match need(&pos, 0, "find or open")? {
        "find" => {
            let text = pos[1..].join(" ");
            if text.trim().is_empty() {
                return Err("pdf find needs the text to look for".to_owned());
            }
            Ok(Command::PdfFind(
                flag(&flags, "file").map(str::to_owned),
                text,
            ))
        }
        "open" => Ok(Command::Screens(ScreenCmd::PdfOpen(
            need(&pos, 1, "PDF path in the vault")?.to_owned(),
        ))),
        other => Err(format!("pdf has find and open (not {other})")),
    }
}
