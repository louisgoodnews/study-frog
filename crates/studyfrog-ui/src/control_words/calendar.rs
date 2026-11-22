//! `calendar view WORD`, `calendar period DATE`, `calendar homework DATE` and `reminders due`.

use studyfrog_core::civil::{Date, parse_date};

use crate::calendar_mode::CalendarViewMode;
use crate::control_cmd::Command;
use crate::control_cmd::calendar::CalendarCmd;
use crate::control_words_items::need;

/// Parse the words after `calendar`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let words: Vec<&str> = rest.iter().map(String::as_str).collect();
    let cmd = match words.first().copied() {
        Some("view") => {
            let word = need(&words, 1, "a view: month, week, day or timetable")?;
            CalendarCmd::View(view_named(word)?)
        }
        Some("period") => CalendarCmd::Period(date(need(&words, 1, "a date")?)?),
        Some("homework") => CalendarCmd::Homework(date(need(&words, 1, "a date")?)?),
        Some(other) => return Err(format!("calendar has view, period, homework (not {other})")),
        None => return Err("calendar needs view, period or homework".to_owned()),
    };
    Ok(Command::Calendar(cmd))
}

/// Parse the words after `reminders`: only `due`.
pub fn reminders(rest: &[String]) -> Result<Command, String> {
    match rest.first().map(String::as_str) {
        Some("due") => Ok(Command::Calendar(CalendarCmd::RemindersDue)),
        _ => Err("reminders has one command: due".to_owned()),
    }
}

fn view_named(word: &str) -> Result<CalendarViewMode, String> {
    match word {
        "month" => Ok(CalendarViewMode::Month),
        "week" => Ok(CalendarViewMode::Week),
        "day" => Ok(CalendarViewMode::Day),
        "timetable" => Ok(CalendarViewMode::Timetable),
        other => Err(format!(
            "unknown view {other} (month, week, day or timetable)"
        )),
    }
}

fn date(word: &str) -> Result<Date, String> {
    parse_date(word).ok_or_else(|| format!("{word} is not a date (YYYY-MM-DD)"))
}
