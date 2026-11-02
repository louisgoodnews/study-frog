//! The words, strings and symbols of a query, each with the column it starts at.

use crate::query::QueryError;

/// A piece of a query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tok {
    /// A bare word (letters, digits and `-_./%`): a name, a keyword, a number or a date.
    Word(String),
    /// A string in double quotes.
    Str(String),
    /// A symbol: `(`, `)`, `,`, `=`, `!=`, `<`, `<=`, `>`, `>=`, `~`, `!~`.
    Sym(&'static str),
}

/// A token and where it starts and ends (columns from 1; `end` is the column after its last character).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// What it is.
    pub tok: Tok,
    /// First column.
    pub col: usize,
    /// The column after the last.
    pub end: usize,
}

fn error(column: usize, message: String) -> QueryError {
    QueryError { column, message }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '%')
}

fn string(chars: &[char], start: usize) -> Result<(String, usize), QueryError> {
    let (mut out, mut i) = (String::new(), start + 1);
    while i < chars.len() {
        match chars[i] {
            '"' => return Ok((out, i + 1)),
            '\\' if i + 1 < chars.len() && matches!(chars[i + 1], '"' | '\\') => {
                out.push(chars[i + 1]);
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    Err(error(
        start + 1,
        format!(
            "The string that starts at column {} is not closed",
            start + 1
        ),
    ))
}

fn symbol(chars: &[char], i: usize) -> Option<(&'static str, usize)> {
    let next = chars.get(i + 1).copied();
    Some(match (chars[i], next) {
        ('(', _) => ("(", 1),
        (')', _) => (")", 1),
        (',', _) => (",", 1),
        ('=', _) => ("=", 1),
        ('~', _) => ("~", 1),
        ('!', Some('=')) => ("!=", 2),
        ('!', Some('~')) => ("!~", 2),
        ('<', Some('=')) => ("<=", 2),
        ('<', _) => ("<", 1),
        ('>', Some('=')) => (">=", 2),
        ('>', _) => (">", 1),
        _ => return None,
    })
}

/// Split a query into tokens.
pub fn lex(text: &str) -> Result<Vec<Token>, QueryError> {
    let chars: Vec<char> = text.chars().collect();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c == '"' {
            let (s, next) = string(&chars, i)?;
            out.push(Token {
                tok: Tok::Str(s),
                col: i + 1,
                end: next + 1,
            });
            i = next;
        } else if let Some((sym, len)) = symbol(&chars, i) {
            out.push(Token {
                tok: Tok::Sym(sym),
                col: i + 1,
                end: i + len + 1,
            });
            i += len;
        } else if is_word_char(c) {
            let end = (i..chars.len())
                .find(|j| !is_word_char(chars[*j]))
                .unwrap_or(chars.len());
            out.push(Token {
                tok: Tok::Word(chars[i..end].iter().collect()),
                col: i + 1,
                end: end + 1,
            });
            i = end;
        } else {
            return Err(error(
                i + 1,
                format!("Unexpected character `{c}` at column {}", i + 1),
            ));
        }
    }
    Ok(out)
}
