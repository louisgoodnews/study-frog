//! Scanning helpers for FrogScript strings, numbers, and symbols.

use crate::error::{LexError, SourcePos};
use crate::token::Token;

pub(crate) fn scan_string(
    bytes: &[u8],
    mut i: usize,
    len: usize,
    mut col: usize,
    quote: u8,
    pos: SourcePos,
) -> Result<(String, usize, usize), LexError> {
    i += 1;
    col += 1;
    let mut out = String::new();
    while i < len && bytes[i] != quote {
        if bytes[i] == b'\n' {
            return Err(LexError::UnclosedString { pos });
        }
        if bytes[i] == b'\\' && i + 1 < len {
            i += 1;
            col += 1;
            match bytes[i] {
                b'n' => out.push('\n'),
                b't' => out.push('\t'),
                b'r' => out.push('\r'),
                b'\\' => out.push('\\'),
                b'"' => out.push('"'),
                b'\'' => out.push('\''),
                c => {
                    out.push('\\');
                    out.push(c as char);
                }
            }
        } else {
            out.push(bytes[i] as char);
        }
        i += 1;
        col += 1;
    }
    if i >= len {
        return Err(LexError::UnclosedString { pos });
    }
    i += 1;
    col += 1;
    Ok((out, i, col))
}

pub(crate) fn scan_number(
    bytes: &[u8],
    mut i: usize,
    len: usize,
    mut col: usize,
    pos: SourcePos,
) -> Result<(Token, usize, usize), LexError> {
    let start = i;
    let mut is_float = false;

    while i < len && (bytes[i] as char).is_ascii_digit() {
        i += 1;
        col += 1;
    }
    if i < len && bytes[i] == b'.' && i + 1 < len && (bytes[i + 1] as char).is_ascii_digit() {
        is_float = true;
        i += 1;
        col += 1;
        while i < len && (bytes[i] as char).is_ascii_digit() {
            i += 1;
            col += 1;
        }
    }

    if !is_float && i < len {
        let mult = match bytes[i] {
            b's' => Some(1),
            b'm' => Some(60),
            b'h' => Some(3600),
            b'd' => Some(86400),
            _ => None,
        };
        if let Some(m) = mult {
            let s = String::from_utf8_lossy(&bytes[start..i]);
            let n: i64 = s.parse().map_err(|_| LexError::InvalidNumber {
                text: s.into_owned(),
                pos,
            })?;
            return Ok((Token::DurationSecs(n * m), i + 1, col + 1));
        }
    }

    let raw = String::from_utf8_lossy(&bytes[start..i]);
    if is_float {
        let f: f64 = raw.parse().map_err(|_| LexError::InvalidNumber {
            text: raw.into_owned(),
            pos,
        })?;
        Ok((Token::Float(f), i, col))
    } else {
        let n: i64 = raw.parse().map_err(|_| LexError::InvalidNumber {
            text: raw.into_owned(),
            pos,
        })?;
        Ok((Token::Int(n), i, col))
    }
}

pub(crate) fn match_symbol(
    bytes: &[u8],
    i: usize,
    len: usize,
    pos: SourcePos,
) -> Result<(Token, usize), LexError> {
    let b = bytes[i];
    let next = if i + 1 < len {
        Some(bytes[i + 1])
    } else {
        None
    };

    if b == b'-' && next == Some(b'>') {
        return Ok((Token::Arrow, 2));
    }
    if b == b'=' && next == Some(b'=') {
        return Ok((Token::Eq, 2));
    }
    if b == b'!' && next == Some(b'=') {
        return Ok((Token::NotEq, 2));
    }
    if b == b'<' && next == Some(b'=') {
        return Ok((Token::LtEq, 2));
    }
    if b == b'>' && next == Some(b'=') {
        return Ok((Token::GtEq, 2));
    }

    let tok = match b {
        b'|' => Token::Pipe,
        b';' => Token::Semicolon,
        b',' => Token::Comma,
        b':' => Token::Colon,
        b'.' => Token::Dot,
        b'=' => Token::Assign,
        b'<' => Token::Lt,
        b'>' => Token::Gt,
        b'+' => Token::Plus,
        b'-' => Token::Minus,
        b'*' => Token::Star,
        b'/' => Token::Slash,
        b'%' => Token::Percent,
        b'(' => Token::LParen,
        b')' => Token::RParen,
        b'[' => Token::LBracket,
        b']' => Token::RBracket,
        b'{' => Token::LBrace,
        b'}' => Token::RBrace,
        ch => {
            return Err(LexError::UnexpectedChar {
                ch: ch as char,
                pos,
            });
        }
    };
    Ok((tok, 1))
}
