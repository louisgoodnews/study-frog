//! Suggestions while a query is typed: field names, operators, values and keywords that fit what comes next.

use crate::query::Field;
use crate::query_fields::{FIELDS, can_be_empty, field_by_name, ops_allowed, takes_in};
use crate::query_lex::{Tok, Token, lex};

/// A suggestion: what to put where the word being typed starts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completion {
    /// The character index in the query where the word being typed starts.
    pub from: usize,
    /// The text to put there instead of the word.
    pub text: String,
}

const KINDS: [&str; 3] = ["note", "card", "question"];
const STATUSES: [&str; 5] = ["new", "learning", "reviewing", "done", "paused"];
const PRIORITIES: [&str; 5] = ["lowest", "low", "medium", "high", "highest"];
const DIFFICULTIES: [&str; 3] = ["easy", "medium", "difficult"];

fn values_of(f: Field) -> Vec<&'static str> {
    match f {
        Field::Type => KINDS.to_vec(),
        Field::Status => STATUSES.to_vec(),
        Field::Priority => PRIORITIES.to_vec(),
        Field::Difficulty => DIFFICULTIES.to_vec(),
        Field::Familiarity => vec!["0.4", "40%", "90%"],
        Field::Due | Field::Created | Field::Updated => vec!["today", "7d", "-14d", "2w", "1m"],
        _ => Vec::new(),
    }
}

fn is_kw(t: &Token, kw: &str) -> bool {
    matches!(&t.tok, Tok::Word(w) if w.eq_ignore_ascii_case(kw))
}

fn field_before(toks: &[Token]) -> Option<Field> {
    toks.iter().rev().find_map(|t| match &t.tok {
        Tok::Word(w) => field_by_name(w),
        _ => None,
    })
}

/// What fits after the tokens `before`.
fn candidates(before: &[Token]) -> Vec<String> {
    let words = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let fields = || {
        FIELDS
            .iter()
            .map(|(_, n)| (*n).to_owned())
            .collect::<Vec<_>>()
    };
    let Some(last) = before.last() else {
        return fields();
    };
    let field = field_before(before);
    match &last.tok {
        Tok::Sym("(") => {
            let in_list = before.len() >= 2 && is_kw(&before[before.len() - 2], "IN");
            if in_list {
                field.map(|f| words(&values_of(f))).unwrap_or_default()
            } else {
                fields()
            }
        }
        Tok::Sym(",") => {
            let in_order = before.iter().any(|t| is_kw(t, "BY"));
            if in_order {
                fields()
            } else {
                field.map(|f| words(&values_of(f))).unwrap_or_default()
            }
        }
        Tok::Sym(")") => words(&["AND", "OR", "ORDER BY"]),
        Tok::Sym(_) => field.map(|f| words(&values_of(f))).unwrap_or_default(),
        Tok::Word(w)
            if ["AND", "OR", "NOT", "BY"]
                .iter()
                .any(|k| w.eq_ignore_ascii_case(k)) =>
        {
            fields()
        }
        Tok::Word(w) if w.eq_ignore_ascii_case("IN") => words(&["("]),
        Tok::Word(w) if w.eq_ignore_ascii_case("IS") => words(&["EMPTY", "NOT"]),
        Tok::Word(w) if w.eq_ignore_ascii_case("ORDER") => words(&["BY"]),
        Tok::Word(w) if field_by_name(w).is_some() && !before.iter().any(|t| is_kw(t, "BY")) => {
            let f = field_by_name(w).unwrap_or(Field::Title);
            let mut ops: Vec<String> = ops_allowed(f)
                .iter()
                .map(|o| crate::query_fields::op_symbol(*o).to_owned())
                .collect();
            if takes_in(f) {
                ops.push("IN".to_owned());
            }
            if can_be_empty(f) {
                ops.push("IS".to_owned());
            }
            ops
        }
        _ => words(&["AND", "OR", "ORDER BY"]),
    }
}

/// The suggestions for the end of `text`: those that begin with the word being typed (all that fit when it is empty).
pub fn complete(text: &str) -> Vec<Completion> {
    let Ok(toks) = lex(text) else {
        return Vec::new();
    };
    let chars = text.chars().count();
    let typing = toks
        .last()
        .filter(|t| matches!(t.tok, Tok::Word(_)) && t.end == chars + 1);
    let (before, prefix, from) = match typing {
        Some(t) => (
            &toks[..toks.len() - 1],
            if let Tok::Word(w) = &t.tok {
                w.clone()
            } else {
                String::new()
            },
            t.col - 1,
        ),
        None => (&toks[..], String::new(), chars),
    };
    let lower = prefix.to_lowercase();
    let mut found: Vec<Completion> = candidates(before)
        .into_iter()
        .filter(|c| c.to_lowercase().starts_with(&lower) && c.to_lowercase() != lower)
        .map(|text| Completion { from, text })
        .collect();
    found.truncate(8);
    found
}

/// The text with the completion applied.
pub fn apply(text: &str, c: &Completion) -> String {
    let head: String = text.chars().take(c.from).collect();
    let space = if c.text == "(" || head.is_empty() || head.ends_with(' ') || head.ends_with('(') {
        ""
    } else {
        " "
    };
    let tail = if c.text == "(" { "" } else { " " };
    format!("{head}{space}{}{tail}", c.text)
}
