//! Parser token stream cursor and helper methods.

use crate::error::{ParseError, Span};
use crate::token::{SpannedToken, Token};

/// Cursor tracking the current position within the token stream.
#[derive(Debug, Clone)]
pub struct ParserState<'a> {
    tokens: &'a [SpannedToken],
    pos: usize,
}

/// Create a new parser state over a token slice.
pub fn new_state(tokens: &[SpannedToken]) -> ParserState<'_> {
    ParserState { tokens, pos: 0 }
}

/// Peek at the current token without consuming it.
pub fn peek<'a>(p: &'a ParserState<'a>) -> &'a Token {
    p.tokens
        .get(p.pos)
        .map(|st| &st.token)
        .unwrap_or(&Token::Eof)
}

/// Peek at the next token (lookahead 1) without consuming it.
pub fn peek_next<'a>(p: &'a ParserState<'a>) -> &'a Token {
    p.tokens
        .get(p.pos + 1)
        .map(|st| &st.token)
        .unwrap_or(&Token::Eof)
}

/// Peek at the current token's source span.
pub fn current_span(p: &ParserState<'_>) -> Span {
    p.tokens.get(p.pos).map(|st| st.span).unwrap_or_default()
}

/// Check if the parser has reached the end of the input.
pub fn is_at_end(p: &ParserState<'_>) -> bool {
    matches!(peek(p), Token::Eof)
}

/// Advance to the next token, returning the consumed token.
pub fn advance<'a>(p: &mut ParserState<'a>) -> &'a SpannedToken {
    let tok = &p.tokens[p.pos];
    if p.pos + 1 < p.tokens.len() {
        p.pos += 1;
    }
    tok
}

/// Consume the current token if it matches the predicate, else return None.
pub fn match_token<'a, F>(p: &mut ParserState<'a>, pred: F) -> Option<&'a SpannedToken>
where
    F: FnOnce(&Token) -> bool,
{
    if pred(peek(p)) {
        Some(advance(p))
    } else {
        None
    }
}

/// Expect and consume a specific token, or return a ParseError.
pub fn expect<'a>(
    p: &mut ParserState<'a>,
    expected: &Token,
    desc: &str,
) -> Result<&'a SpannedToken, ParseError> {
    let current = peek(p);
    if current == expected {
        Ok(advance(p))
    } else {
        let span = current_span(p);
        Err(ParseError::ExpectedToken {
            expected: desc.to_string(),
            found: current.to_string(),
            pos: span.pos,
        })
    }
}

/// Expect and consume an identifier token, returning its string value.
pub fn expect_ident(p: &mut ParserState<'_>) -> Result<(String, Span), ParseError> {
    let span = current_span(p);
    match peek(p) {
        Token::Ident(name) => {
            let name = name.clone();
            advance(p);
            Ok((name, span))
        }
        Token::Type => {
            advance(p);
            Ok(("type".into(), span))
        }
        Token::Tag => {
            advance(p);
            Ok(("tag".into(), span))
        }
        Token::Stack => {
            advance(p);
            Ok(("stack".into(), span))
        }
        Token::Set => {
            advance(p);
            Ok(("set".into(), span))
        }
        Token::Where => {
            advance(p);
            Ok(("where".into(), span))
        }
        found => Err(ParseError::ExpectedToken {
            expected: "identifier".to_string(),
            found: found.to_string(),
            pos: span.pos,
        }),
    }
}

/// Skip any sequence of newline or semicolon tokens.
pub fn skip_separators(p: &mut ParserState<'_>) {
    while matches!(peek(p), Token::Newline | Token::Semicolon) {
        advance(p);
    }
}

/// Check if the current token is a statement delimiter (newline, semicolon, or EOF).
pub fn is_stmt_delimiter(p: &ParserState<'_>) -> bool {
    matches!(
        peek(p),
        Token::Newline | Token::Semicolon | Token::Eof | Token::RBrace
    )
}

/// Construct a parse error for an unexpected token at current position.
pub fn unexpected_token_error(p: &ParserState<'_>) -> ParseError {
    let span = current_span(p);
    ParseError::UnexpectedToken {
        token: peek(p).to_string(),
        pos: span.pos,
    }
}
