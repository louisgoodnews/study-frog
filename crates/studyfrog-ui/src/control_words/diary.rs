//! `diary`, `diary list [DATE [DATE]]`, `diary open ULID` and
//! `diary add DATE TITLE [--text T] [--minutes N] [--tags T] [--stacks S]`.

use studyfrog_core::civil::parse_date;

use crate::control_cmd::Command;
use crate::control_cmd::diary::{DiaryCmd, DiaryNew};
use crate::control_words_items::{flag, split_flags};

/// Parse the words after `diary`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let (flags, pos) = split_flags(rest, &[])?;
    let cmd = match pos.first().copied() {
        None | Some("list") => list(&pos[pos.len().min(1)..])?,
        Some("open") => DiaryCmd::Open(entry_ulid(pos.get(1).copied())?),
        Some("add") => add(&flags, &pos[1..])?,
        Some(other) => {
            return Err(format!("diary has list, open and add (not {other})"));
        }
    };
    Ok(Command::Diary(cmd))
}

fn date(word: &str) -> Result<studyfrog_core::civil::Date, String> {
    parse_date(word).ok_or_else(|| format!("{word} is not a date (YYYY-MM-DD)"))
}

fn list(days: &[&str]) -> Result<DiaryCmd, String> {
    let from = days.first().map(|d| date(d)).transpose()?;
    let to = days.get(1).map(|d| date(d)).transpose()?;
    if days.len() > 2 {
        return Err("diary list takes one day or a first and a last day".to_owned());
    }
    Ok(DiaryCmd::List { from, to })
}

fn entry_ulid(word: Option<&str>) -> Result<studyfrog_core::uid::Uid, String> {
    let word = word.ok_or("diary open needs the entry's ULID")?;
    studyfrog_md::ulid::parse_ulid(word).ok_or_else(|| format!("{word} is not a ULID"))
}

fn add(flags: &[(&str, &str)], words: &[&str]) -> Result<DiaryCmd, String> {
    let day = date(words.first().ok_or("diary add needs a date (YYYY-MM-DD)")?)?;
    let title = words.get(1..).unwrap_or(&[]).join(" ");
    if title.is_empty() {
        return Err("diary add needs a title".to_owned());
    }
    let minutes = flag(flags, "minutes")
        .map(|m| {
            m.parse::<u32>()
                .map_err(|_| format!("--minutes {m}: not a number"))
        })
        .transpose()?;
    Ok(DiaryCmd::Add(DiaryNew {
        date: day,
        title,
        text: flag(flags, "text").unwrap_or("").to_owned(),
        minutes,
        tags: flag(flags, "tags").unwrap_or("").to_owned(),
        stacks: flag(flags, "stacks").unwrap_or("").to_owned(),
    }))
}
