//! The place the parser is at in the tokens, and the small helpers it uses to look at them.

use crate::query::QueryError;
use crate::query_lex::{Tok, Token};

/// Where the parser is.
pub(crate) struct Cursor<'a> {
    pub toks: &'a [Token],
    pub pos: usize,
    pub depth: usize,
}

pub(crate) type Parsed<T> = Result<T, QueryError>;

pub(crate) fn fail<T>(column: usize, message: String) -> Parsed<T> {
    Err(QueryError { column, message })
}

pub(crate) fn peek<'a>(c: &Cursor<'a>) -> Option<&'a Token> {
    c.toks.get(c.pos)
}

pub(crate) fn take<'a>(c: &mut Cursor<'a>) -> Option<&'a Token> {
    let t = c.toks.get(c.pos);
    c.pos += usize::from(t.is_some());
    t
}

pub(crate) fn is_word(t: Option<&Token>, kw: &str) -> bool {
    matches!(t, Some(Token { tok: Tok::Word(w), .. }) if w.eq_ignore_ascii_case(kw))
}

pub(crate) fn is_sym(t: Option<&Token>, s: &str) -> bool {
    matches!(t, Some(Token { tok: Tok::Sym(x), .. }) if *x == s)
}

pub(crate) fn shown(t: &Token) -> String {
    match &t.tok {
        Tok::Word(w) => w.clone(),
        Tok::Str(s) => format!("\"{s}\""),
        Tok::Sym(s) => (*s).to_owned(),
    }
}

/// The column after the token before the cursor, where something is missing.
pub(crate) fn missing_at(c: &Cursor) -> usize {
    c.pos
        .checked_sub(1)
        .and_then(|i| c.toks.get(i))
        .map_or(1, |t| t.end)
}
