//! How a value in a query is read for the field it is compared with.

use crate::civil::{date_to_days, parse_date};
use crate::event::difficulty_from_name;
use crate::plan::item_kind_from_name;
use crate::priority::priority_from_name;
use crate::query::{DateUnit, DateVal, Field, QueryError, Val};
use crate::query_lex::{Tok, Token};
use crate::status::status_from_name;

fn error(t: &Token, message: String) -> QueryError {
    QueryError {
        column: t.col,
        message,
    }
}

fn text_of(t: &Token) -> Option<&str> {
    match &t.tok {
        Tok::Word(w) | Tok::Str(w) => Some(w),
        Tok::Sym(_) => None,
    }
}

/// "status" -> "statuses", "type" -> "types".
fn plural(what: &str) -> String {
    if what.ends_with('s') {
        format!("{what}es")
    } else {
        format!("{what}s")
    }
}

fn choice<T>(t: &Token, what: &str, names: &[&str], found: Option<T>) -> Result<T, QueryError> {
    let word = text_of(t).unwrap_or_default();
    found.ok_or_else(|| {
        error(
            t,
            format!(
                "Unknown {what} `{word}` at column {}. The {} are {}",
                t.col,
                plural(what),
                names.join(", ")
            ),
        )
    })
}

fn number(t: &Token) -> Result<f64, QueryError> {
    let word = text_of(t)
        .filter(|_| matches!(t.tok, Tok::Word(_)))
        .unwrap_or_default();
    let (digits, scale) = word.strip_suffix('%').map_or((word, 1.0), |d| (d, 0.01));
    let n = digits.parse::<f64>().ok().filter(|n| n.is_finite());
    let n = n.ok_or_else(|| {
        error(
            t,
            format!(
                "familiarity needs a number between 0 and 1, or a percent, at column {}",
                t.col
            ),
        )
    })?;
    let n = n * scale;
    if (0.0..=1.0).contains(&n) {
        Ok(n)
    } else {
        Err(error(
            t,
            format!("familiarity is between 0 and 1 (column {})", t.col),
        ))
    }
}

fn relative(word: &str) -> Option<DateVal> {
    let unit = match word.chars().last()?.to_ascii_lowercase() {
        'd' => DateUnit::Day,
        'w' => DateUnit::Week,
        'm' => DateUnit::Month,
        'y' => DateUnit::Year,
        _ => return None,
    };
    let count = word[..word.len() - 1]
        .parse::<i64>()
        .ok()
        .filter(|n| n.abs() <= 100_000)?;
    Some(DateVal::Relative(count, unit))
}

fn date(t: &Token) -> Result<DateVal, QueryError> {
    let word = text_of(t)
        .filter(|_| matches!(t.tok, Tok::Word(_)))
        .unwrap_or_default();
    if word.eq_ignore_ascii_case("today") {
        return Ok(DateVal::Relative(0, DateUnit::Day));
    }
    if let Some(d) = parse_date(word) {
        return Ok(DateVal::Day(date_to_days(d)));
    }
    relative(word).ok_or_else(|| {
        error(
            t,
            format!(
                "`{word}` is not a date at column {}. Use 2026-11-18, today, 7d or -14d",
                t.col
            ),
        )
    })
}

fn whole(t: &Token) -> Result<i64, QueryError> {
    let word = text_of(t).unwrap_or_default();
    word.parse::<i64>().ok().filter(|n| *n >= 0).ok_or_else(|| {
        error(
            t,
            format!("reviews needs a whole number at column {}", t.col),
        )
    })
}

const KINDS: [&str; 3] = ["note", "card", "question"];
const STATUSES: [&str; 5] = ["new", "learning", "reviewing", "done", "paused"];
const PRIORITIES: [&str; 5] = ["lowest", "low", "medium", "high", "highest"];
const DIFFICULTIES: [&str; 3] = ["easy", "medium", "difficult"];

/// Read a value for a field from a word or string token.
pub fn value_of(field: Field, t: &Token) -> Result<Val, QueryError> {
    let Some(text) = text_of(t) else {
        return Err(error(t, format!("Expected a value at column {}", t.col)));
    };
    let lower = text.to_lowercase();
    match field {
        Field::Type => choice(t, "type", &KINDS, item_kind_from_name(&lower)).map(Val::Kind),
        Field::Status => choice(t, "status", &STATUSES, status_from_name(&lower)).map(Val::Status),
        Field::Priority => {
            choice(t, "priority", &PRIORITIES, priority_from_name(&lower)).map(Val::Priority)
        }
        Field::Difficulty => choice(t, "difficulty", &DIFFICULTIES, difficulty_from_name(&lower))
            .map(Val::Difficulty),
        Field::Familiarity => number(t).map(Val::Num),
        Field::Reviews => whole(t).map(Val::Int),
        Field::Due | Field::Created | Field::Updated => date(t).map(Val::Date),
        _ => Ok(Val::Text(text.to_owned())),
    }
}
