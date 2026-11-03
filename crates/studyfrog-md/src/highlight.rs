//! Syntax highlighting for fenced code blocks: a small tokenizer driven by a table per language
//! ([`crate::highlight_langs`]). It knows five kinds of token, runs once when the text is parsed and never fails;
//! an unknown or missing language gives no tokens, so the code is drawn plain.
//!
//! Build or buy (P10): `syntect` brings hundreds of grammars, a regex engine and large data files; for eleven
//! languages and five kinds of token a table-driven scanner is a few hundred lines, starts instantly and adds no
//! dependency, so it is built here.

use crate::highlight_langs::{Flavour, Lang, lang_of};

/// What a stretch of code is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Anything else.
    Plain,
    /// A keyword, a tag or a section header.
    Keyword,
    /// A string.
    String,
    /// A comment.
    Comment,
    /// A number.
    Number,
}

/// A stretch of code of one kind: its length in bytes. The tokens cover the whole text in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind.
    pub kind: Kind,
    /// Length in bytes.
    pub len: usize,
}

struct Scan<'a> {
    lang: &'a Lang,
    code: &'a str,
    tokens: Vec<Token>,
    in_tag: bool,
}

fn word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// How many bytes of `rest` the text up to and excluding the first of `stops` takes, or all of it.
fn until(rest: &str, stops: &[char]) -> usize {
    rest.find(stops).unwrap_or(rest.len())
}

/// A string that starts with the quote at the start of `rest`: up to and including the closing quote, not past a
/// line end.
fn string_len(rest: &str) -> usize {
    let mut chars = rest.char_indices();
    let Some((_, quote)) = chars.next() else {
        return 0;
    };
    let mut escaped = false;
    for (i, c) in chars {
        match c {
            '\n' => return i,
            _ if escaped => escaped = false,
            '\\' => escaped = true,
            c if c == quote => return i + c.len_utf8(),
            _ => {}
        }
    }
    rest.len()
}

fn triple_len(rest: &str, quote: &str) -> usize {
    rest[3..].find(quote).map_or(rest.len(), |i| i + 6)
}

fn push(s: &mut Scan, kind: Kind, len: usize) {
    match s.tokens.last_mut() {
        Some(last) if last.kind == kind => last.len += len,
        _ => s.tokens.push(Token { kind, len }),
    }
}

fn at_line_start(s: &Scan, i: usize) -> bool {
    i == 0 || s.code.as_bytes()[i - 1] == b'\n'
}

/// The kind and length of the token at byte `i`.
fn next(s: &mut Scan, i: usize) -> (Kind, usize) {
    let rest = &s.code[i..];
    let lang = s.lang;
    match lang.flavour {
        Flavour::Table if i == 0 => return (Kind::Keyword, until(rest, &['\n'])),
        Flavour::Sections if at_line_start(s, i) && rest.trim_start().starts_with('[') => {
            return (Kind::Keyword, until(rest, &['\n']));
        }
        Flavour::Markup => return markup(s, rest),
        _ => {}
    }
    generic(s, rest, i)
}

fn markup(s: &mut Scan, rest: &str) -> (Kind, usize) {
    let quoted = rest.starts_with(['"', '\'']);
    if s.in_tag {
        return match rest.chars().next() {
            Some('>') => {
                s.in_tag = false;
                (Kind::Keyword, 1)
            }
            _ if quoted => (Kind::String, string_len(rest)),
            _ => (Kind::Keyword, until(rest, &['"', '\'', '>']).max(1)),
        };
    }
    if let Some((open, close)) = s.lang.block_comment
        && rest.starts_with(open)
    {
        let end = rest.find(close).map_or(rest.len(), |i| i + close.len());
        return (Kind::Comment, end);
    }
    let mut chars = rest.chars();
    let tag = chars.next() == Some('<')
        && chars
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '/' || c == '!');
    if tag {
        s.in_tag = true;
        let name = rest[1..]
            .find(|c: char| c.is_whitespace() || c == '>')
            .unwrap_or(rest.len() - 1);
        return (Kind::Keyword, 1 + name);
    }
    let first = rest.chars().next().map_or(0, char::len_utf8);
    (Kind::Plain, first + until(&rest[first..], &['<']))
}

fn generic(s: &Scan, rest: &str, i: usize) -> (Kind, usize) {
    let lang = s.lang;
    if let Some((open, close)) = lang.block_comment
        && rest.starts_with(open)
    {
        return (
            Kind::Comment,
            rest[open.len()..]
                .find(close)
                .map_or(rest.len(), |e| e + open.len() + close.len()),
        );
    }
    if lang.line_comments.iter().any(|c| rest.starts_with(c)) {
        return (Kind::Comment, until(rest, &['\n']));
    }
    if lang.triple_quotes {
        for quote in ["\"\"\"", "'''"] {
            if rest.starts_with(quote) {
                return (Kind::String, triple_len(rest, quote));
            }
        }
    }
    let first = rest.chars().next().unwrap_or(' ');
    if lang.quotes.contains(&first) {
        return (Kind::String, string_len(rest));
    }
    let after_word = i > 0 && s.code[..i].chars().next_back().is_some_and(word_char);
    if lang.numbers && first.is_ascii_digit() && !after_word {
        let n = rest
            .find(|c: char| !(c.is_alphanumeric() || c == '.' || c == '_'))
            .unwrap_or(rest.len());
        return (Kind::Number, n);
    }
    if first.is_alphabetic() || first == '_' {
        let n = rest.find(|c: char| !word_char(c)).unwrap_or(rest.len());
        let kind = if lang.keywords.contains(&&rest[..n]) {
            Kind::Keyword
        } else {
            Kind::Plain
        };
        return (kind, n);
    }
    (Kind::Plain, first.len_utf8())
}

/// The tokens of `code` in the language `info` names (the first word of a code fence). Empty when the language is
/// unknown, so the caller draws the code plain.
pub fn highlight(info: &str, code: &str) -> Vec<Token> {
    let Some(lang) = lang_of(info) else {
        return Vec::new();
    };
    let mut scan = Scan {
        lang,
        code,
        tokens: Vec::new(),
        in_tag: false,
    };
    let mut i = 0;
    while i < code.len() {
        let (kind, len) = next(&mut scan, i);
        let len = len.max(1);
        let len = (i + len..=code.len())
            .find(|e| code.is_char_boundary(*e))
            .map_or(code.len() - i, |e| e - i);
        push(&mut scan, kind, len);
        i += len;
    }
    scan.tokens
}
