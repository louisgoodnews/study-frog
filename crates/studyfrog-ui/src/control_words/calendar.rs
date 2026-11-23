//! `calendar view WORD`, `calendar period DATE`, `calendar homework DATE`, `calendar session add`
//! and `reminders due`.

use studyfrog_core::civil::{Date, LocalTime, parse_date, parse_minute};

use crate::calendar_mode::CalendarViewMode;
use crate::control_cmd::Command;
use crate::control_cmd::calendar::{CalendarCmd, SessionAdd};
use crate::control_words_items::{flag, need, split_flags};

/// Parse the words after `calendar`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let words: Vec<&str> = rest.iter().map(String::as_str).collect();
    let cmd = match words.first().copied() {
        Some("view") if words.len() == 1 => CalendarCmd::Show,
        Some("period") if words.len() == 1 => CalendarCmd::Show,
        Some("view") => {
            let word = need(&words, 1, "a view: month, week, day or timetable")?;
            CalendarCmd::View(view_named(word)?)
        }
        Some("period") => CalendarCmd::Period(date(need(&words, 1, "a date")?)?),
        Some("homework") => CalendarCmd::Homework(date(need(&words, 1, "a date")?)?),
        Some("session") => CalendarCmd::SessionAdd(session_add(&rest[1..])?),
        Some(other) => {
            return Err(format!(
                "calendar has view, period, homework, session (not {other})"
            ));
        }
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

/// The words after `calendar session`: `add TITLE --start "YYYY-MM-DD HH:MM" --minutes N`.
fn session_add(words: &[String]) -> Result<SessionAdd, String> {
    let (flags, rest) = split_flags(words, &[])?;
    match rest.first().copied() {
        Some("add") => {}
        _ => return Err("calendar session has one command: add".to_owned()),
    }
    let title = need(&rest, 1, "a title")?.to_owned();
    let start =
        start_of(flag(&flags, "start").ok_or("session add needs --start \"YYYY-MM-DD HH:MM\"")?)?;
    let minutes = flag(&flags, "minutes")
        .ok_or("session add needs --minutes N")?
        .parse::<u16>()
        .ok()
        .filter(|m| *m > 0)
        .ok_or("--minutes must be a whole number above 0")?;
    Ok(SessionAdd {
        title,
        start,
        minutes,
    })
}

/// A local start from `YYYY-MM-DD HH:MM`.
fn start_of(text: &str) -> Result<LocalTime, String> {
    let (day, time) = text
        .trim()
        .split_once(' ')
        .ok_or_else(|| format!("{text} is not a start (YYYY-MM-DD HH:MM)"))?;
    Ok(LocalTime {
        date: date(day)?,
        minute: parse_minute(time.trim()).ok_or_else(|| format!("{time} is not a time (HH:MM)"))?,
    })
}

fn date(word: &str) -> Result<Date, String> {
    parse_date(word).ok_or_else(|| format!("{word} is not a date (YYYY-MM-DD)"))
}
