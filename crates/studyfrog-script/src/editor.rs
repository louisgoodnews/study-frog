//! Editor help for FrogScript (STORY-0150): the colour of each piece of text, the bracket that matches the one at the
//! cursor, the first error with its line, and the names that can complete the word being typed. Pure: the editor draws
//! what these give.

use std::ops::Range;

use crate::error::{LexError, ParseError};
use crate::lexer::tokenize;
use crate::token::Token;
use crate::{ScriptError, parse_frogscript};

pub mod complete;
pub use complete::{completions, step, with_completion, word_at};

/// The built-in functions a script can call (see `eval_stdlib`).
pub const BUILTINS: &[&str] = &[
    "print",
    "log_info",
    "log_warn",
    "log_error",
    "len",
    "now",
    "today",
    "days_ago",
    "tag",
    "type",
    "difficulty",
    "priority",
];

/// The members of the `vault` object a script can use after `vault.`.
pub const VAULT_MEMBERS: &[&str] = &["stacks", "has_stack", "stack", "create_card", "tags"];

/// The reserved words of the language.
pub const KEYWORDS: &[&str] = &[
    "from", "stack", "where", "type", "set", "tag", "untag", "sort", "limit", "select", "export",
    "if", "elif", "else", "for", "in", "def", "return", "and", "or", "not", "true", "false",
    "null",
];

/// What a piece of the text is, for its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Part {
    /// A reserved word.
    Keyword,
    /// A built-in function's name.
    Builtin,
    /// A string literal.
    Text,
    /// A number or a duration.
    Number,
    /// A `#` comment, to the end of its line.
    Comment,
    /// An operator or punctuation.
    Operator,
    /// Anything else (names of the script's own, whitespace).
    Plain,
}

/// The coloured pieces of `src`, in order, covering the whole text. A text that does not lex is all `Plain`.
pub fn highlight(src: &str) -> Vec<(Range<usize>, Part)> {
    let Ok(tokens) = tokenize(src) else {
        return vec![(0..src.len(), Part::Plain)];
    };
    let mut out = Vec::new();
    let mut at = 0;
    for t in &tokens {
        let (start, end) = (t.span.start.min(src.len()), t.span.end.min(src.len()));
        if start > at {
            gap(src, at, start, &mut out);
        }
        if end > start {
            out.push((start..end, part_of(&t.token)));
        }
        at = at.max(end);
    }
    if at < src.len() {
        gap(src, at, src.len(), &mut out);
    }
    out
}

/// Between two tokens there is whitespace, and maybe a comment (which runs to the line's end).
fn gap(src: &str, from: usize, to: usize, out: &mut Vec<(Range<usize>, Part)>) {
    let text = &src[from..to];
    match text.find('#') {
        Some(hash) => {
            let at = from + hash;
            if at > from {
                out.push((from..at, Part::Plain));
            }
            let end = src[at..]
                .find('\n')
                .map_or(src.len(), |n| at + n)
                .min(to.max(at));
            out.push((at..end, Part::Comment));
            if end < to {
                out.push((end..to, Part::Plain));
            }
        }
        None => out.push((from..to, Part::Plain)),
    }
}

fn part_of(t: &Token) -> Part {
    match t {
        Token::Ident(name) if BUILTINS.contains(&name.as_str()) => Part::Builtin,
        Token::Ident(_) => Part::Plain,
        Token::Str(_) | Token::FormatStr(_) => Part::Text,
        Token::Int(_) | Token::Float(_) | Token::DurationSecs(_) => Part::Number,
        Token::Newline => Part::Plain,
        Token::From
        | Token::Stack
        | Token::Where
        | Token::Type
        | Token::Set
        | Token::Tag
        | Token::Untag
        | Token::Sort
        | Token::Limit
        | Token::Select
        | Token::Export
        | Token::If
        | Token::Elif
        | Token::Else
        | Token::For
        | Token::In
        | Token::Def
        | Token::Return
        | Token::And
        | Token::Or
        | Token::Not
        | Token::True
        | Token::False
        | Token::Null => Part::Keyword,
        Token::Eof => Part::Plain,
        _ => Part::Operator,
    }
}

/// The byte index of the bracket that matches the one at `at` (the bracket just before the cursor, or the one under
/// it), skipping string literals. `None` when there is no bracket there or no match.
pub fn matching_bracket(src: &str, at: usize) -> Option<usize> {
    let bytes = src.as_bytes();
    let here = [at, at.saturating_sub(1)]
        .into_iter()
        .find(|&i| i < bytes.len() && b"()[]{}".contains(&bytes[i]))?;
    let (open, close, forward) = match bytes[here] {
        b'(' => (b'(', b')', true),
        b'[' => (b'[', b']', true),
        b'{' => (b'{', b'}', true),
        b')' => (b'(', b')', false),
        b']' => (b'[', b']', false),
        _ => (b'{', b'}', false),
    };
    let mut depth = 0i32;
    let mut in_text = false;
    let mut i = here;
    loop {
        let b = bytes[i];
        if b == b'"' {
            in_text = !in_text;
        } else if !in_text {
            if b == open {
                depth += if forward { 1 } else { -1 };
            } else if b == close {
                depth += if forward { -1 } else { 1 };
            }
            if depth == 0 {
                return Some(i);
            }
        }
        if forward {
            i += 1;
            if i >= bytes.len() {
                return None;
            }
        } else {
            if i == 0 {
                return None;
            }
            i -= 1;
        }
    }
}

/// Where the first error is: its line (`None` when the error is at the end of the text) and what it says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorAt {
    /// The 1-based line, when known.
    pub line: Option<usize>,
    /// The message, as the script reports it.
    pub message: String,
}

/// The first error in `src` (lexing or parsing; a run-time error has no place in the text). `None` for a valid script.
pub fn error_of(src: &str) -> Option<ErrorAt> {
    let e = match parse_frogscript(src) {
        Ok(_) => return None,
        Err(e) => e,
    };
    let line = match &e {
        ScriptError::Lex(LexError::UnexpectedChar { pos, .. })
        | ScriptError::Lex(LexError::UnclosedString { pos })
        | ScriptError::Lex(LexError::InvalidNumber { pos, .. })
        | ScriptError::Parse(ParseError::ExpectedToken { pos, .. })
        | ScriptError::Parse(ParseError::UnexpectedToken { pos, .. })
        | ScriptError::Parse(ParseError::InvalidPipelineStage { pos, .. }) => Some(pos.line),
        _ => None,
    };
    Some(ErrorAt {
        line,
        message: e.to_string(),
    })
}

/// The byte range of a 1-based line in `src` (without its newline).
pub fn line_range(src: &str, line: usize) -> Option<Range<usize>> {
    let mut start = 0;
    for (n, text) in src.split('\n').enumerate() {
        if n + 1 == line {
            return Some(start..start + text.len());
        }
        start += text.len() + 1;
    }
    None
}
