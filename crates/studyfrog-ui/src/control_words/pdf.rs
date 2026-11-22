//! `pdf find TEXT [--file PATH]`: set the find field of the PDF viewer, opening the file first when given.

use crate::control_cmd::Command;
use crate::control_words_items::{flag, need, split_flags};

/// Parse the words after `pdf`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let (flags, pos) = split_flags(rest, &[])?;
    let verb = need(&pos, 0, "find")?;
    if verb != "find" {
        return Err(format!("pdf has one command: find (not {verb})"));
    }
    let text = pos[1..].join(" ");
    if text.trim().is_empty() {
        return Err("pdf find needs the text to look for".to_owned());
    }
    Ok(Command::PdfFind(
        flag(&flags, "file").map(str::to_owned),
        text,
    ))
}
