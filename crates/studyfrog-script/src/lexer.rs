//! Tokenizer / lexical analyzer for the FrogScript language.

use crate::error::{LexError, SourcePos, Span};
use crate::lexer_scan::{match_symbol, scan_number, scan_string};
use crate::token::{SpannedToken, Token, lookup_keyword};

/// Lexes source code into a sequence of spanned tokens.
pub fn tokenize(source: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut tokens: Vec<SpannedToken> = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let (mut i, mut line, mut col) = (0, 1, 1);
    let mut bracket_depth: usize = 0;

    while i < len {
        let b = bytes[i];
        let start_byte = i;
        let pos = SourcePos { line, col };

        if b == b'\n' {
            i += 1;
            line += 1;
            col = 1;
            if bracket_depth == 0
                && let Some(last) = tokens.last()
            {
                let pipe_or_sep = matches!(
                    last.token,
                    Token::Newline | Token::Pipe | Token::Comma | Token::Arrow | Token::Colon
                );
                if !pipe_or_sep {
                    tokens.push(make_token(Token::Newline, start_byte, i, pos));
                }
            }
            continue;
        }

        if b == b'\r' {
            i += 1;
            continue;
        }

        if b == b' ' || b == b'\t' {
            i += 1;
            col += 1;
            continue;
        }

        if b == b'#' || (b == b'/' && i + 1 < len && bytes[i + 1] == b'/') {
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        if (b as char).is_ascii_alphabetic() || b == b'_' {
            if b == b'f' && i + 1 < len && (bytes[i + 1] == b'"' || bytes[i + 1] == b'\'') {
                i += 1;
                col += 1;
                let (s, ni, nc) = scan_string(bytes, i, len, col, bytes[i], pos)?;
                tokens.push(make_token(Token::FormatStr(s), start_byte, ni, pos));
                (i, col) = (ni, nc);
                continue;
            }
            let (ident, ni, nc) = scan_ident(bytes, i, len, col);
            let tok = lookup_keyword(&ident).unwrap_or(Token::Ident(ident));
            tokens.push(make_token(tok, start_byte, ni, pos));
            (i, col) = (ni, nc);
            continue;
        }

        if b == b'"' || b == b'\'' {
            let (s, ni, nc) = scan_string(bytes, i, len, col, b, pos)?;
            tokens.push(make_token(Token::Str(s), start_byte, ni, pos));
            (i, col) = (ni, nc);
            continue;
        }

        if (b as char).is_ascii_digit() {
            let (tok, ni, nc) = scan_number(bytes, i, len, col, pos)?;
            tokens.push(make_token(tok, start_byte, ni, pos));
            (i, col) = (ni, nc);
            continue;
        }

        let (tok, consumed) = match_symbol(bytes, i, len, pos)?;
        match &tok {
            Token::LParen | Token::LBracket => bracket_depth += 1,
            Token::RParen | Token::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        tokens.push(make_token(tok, start_byte, i + consumed, pos));
        i += consumed;
        col += consumed;
    }

    tokens.push(make_token(Token::Eof, len, len, SourcePos { line, col }));
    Ok(tokens)
}

fn scan_ident(bytes: &[u8], mut i: usize, len: usize, mut col: usize) -> (String, usize, usize) {
    let start = i;
    while i < len && ((bytes[i] as char).is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
        col += 1;
    }
    let ident = String::from_utf8_lossy(&bytes[start..i]).into_owned();
    (ident, i, col)
}

fn make_token(token: Token, start: usize, end: usize, pos: SourcePos) -> SpannedToken {
    SpannedToken {
        token,
        span: Span { start, end, pos },
    }
}
