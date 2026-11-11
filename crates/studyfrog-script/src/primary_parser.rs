//! Parsing of primary expressions, calls, literals, and closures.

use crate::ast::{Expr, Literal, Stmt};
use crate::error::ParseError;
use crate::parser_state::{
    ParserState, advance, current_span, expect, expect_ident, peek, unexpected_token_error,
};
use crate::token::Token;

/// Parse primary expressions and postfix operations (calls, field access).
pub fn parse_postfix(
    p: &mut ParserState<'_>,
    parse_expr_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
) -> Result<Expr, ParseError> {
    let mut target = parse_primary(p, parse_expr_fn)?;
    loop {
        if matches!(peek(p), Token::LParen) {
            advance(p);
            let mut args = Vec::new();
            if !matches!(peek(p), Token::RParen) {
                args.push(parse_expr_fn(p)?);
                while matches!(peek(p), Token::Comma) {
                    advance(p);
                    if matches!(peek(p), Token::RParen) {
                        break;
                    }
                    args.push(parse_expr_fn(p)?);
                }
            }
            let end = expect(p, &Token::RParen, "closing ')'")?.span;
            target = Expr::Call {
                callee: Box::new(target),
                args,
                span: end,
            };
        } else if matches!(peek(p), Token::Dot) {
            advance(p);
            let (field, span) = expect_ident(p)?;
            target = Expr::FieldAccess {
                target: Box::new(target),
                field,
                span,
            };
        } else if matches!(peek(p), Token::Str(_) | Token::FormatStr(_)) {
            let arg = parse_primary(p, parse_expr_fn)?;
            let span = current_span(p);
            target = Expr::Call {
                callee: Box::new(target),
                args: vec![arg],
                span,
            };
        } else {
            break;
        }
    }
    Ok(target)
}

fn parse_primary(
    p: &mut ParserState<'_>,
    parse_expr_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
) -> Result<Expr, ParseError> {
    let span = current_span(p);
    match peek(p) {
        Token::From => {
            advance(p);
            let target = if matches!(peek(p), Token::Stack) {
                advance(p);
                Some(Box::new(parse_primary(p, parse_expr_fn)?))
            } else if matches!(peek(p), Token::Ident(id) if id == "vault") {
                advance(p);
                None
            } else {
                Some(Box::new(parse_primary(p, parse_expr_fn)?))
            };
            Ok(Expr::From { target, span })
        }
        Token::Int(n) => {
            let n = *n;
            advance(p);
            Ok(Expr::Literal(Literal::Int(n), span))
        }
        Token::Float(f) => {
            let f = *f;
            advance(p);
            Ok(Expr::Literal(Literal::Float(f), span))
        }
        Token::DurationSecs(s) => {
            let s = *s;
            advance(p);
            Ok(Expr::Literal(Literal::DurationSecs(s), span))
        }
        Token::Str(s) => {
            let s = s.clone();
            advance(p);
            Ok(Expr::Literal(Literal::Str(s), span))
        }
        Token::FormatStr(s) => {
            let s = s.clone();
            advance(p);
            Ok(Expr::Literal(Literal::FormatStr(s), span))
        }
        Token::True => {
            advance(p);
            Ok(Expr::Literal(Literal::Bool(true), span))
        }
        Token::False => {
            advance(p);
            Ok(Expr::Literal(Literal::Bool(false), span))
        }
        Token::Null => {
            advance(p);
            Ok(Expr::Literal(Literal::Null, span))
        }
        Token::Star => {
            advance(p);
            Ok(Expr::Literal(Literal::Wildcard, span))
        }
        Token::Ident(name) => {
            let name = name.clone();
            advance(p);
            Ok(Expr::Ident(name, span))
        }
        Token::Type => {
            advance(p);
            Ok(Expr::Ident("type".into(), span))
        }
        Token::Tag => {
            advance(p);
            Ok(Expr::Ident("tag".into(), span))
        }
        Token::Stack => {
            advance(p);
            Ok(Expr::Ident("stack".into(), span))
        }
        Token::Set => {
            advance(p);
            Ok(Expr::Ident("set".into(), span))
        }
        Token::LParen => {
            advance(p);
            let inner = parse_expr_fn(p)?;
            expect(p, &Token::RParen, "closing ')'")?;
            Ok(inner)
        }
        Token::LBracket => {
            advance(p);
            let mut items = Vec::new();
            if !matches!(peek(p), Token::RBracket) {
                items.push(parse_expr_fn(p)?);
                while matches!(peek(p), Token::Comma) {
                    advance(p);
                    if matches!(peek(p), Token::RBracket) {
                        break;
                    }
                    items.push(parse_expr_fn(p)?);
                }
            }
            expect(p, &Token::RBracket, "closing ']'")?;
            Ok(Expr::List(items, span))
        }
        Token::LBrace => parse_closure(p, parse_expr_fn),
        _ => Err(unexpected_token_error(p)),
    }
}

pub(crate) fn parse_closure(
    p: &mut ParserState<'_>,
    parse_expr_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
) -> Result<Expr, ParseError> {
    let span = current_span(p);
    expect(p, &Token::LBrace, "opening '{'")?;
    while matches!(peek(p), Token::Newline) {
        advance(p);
    }

    let mut params = Vec::new();
    if matches!(peek(p), Token::LParen) {
        advance(p);
        let (first, _) = expect_ident(p)?;
        params.push(first);
        while matches!(peek(p), Token::Comma) {
            advance(p);
            let (next, _) = expect_ident(p)?;
            params.push(next);
        }
        expect(p, &Token::RParen, "closing ')'")?;
        expect(p, &Token::Arrow, "closure '->'")?;
    } else if let Token::Ident(name) = peek(p) {
        let name = name.clone();
        advance(p);
        if matches!(peek(p), Token::Arrow) {
            advance(p);
            params.push(name);
        } else {
            let id_expr = Expr::Ident(name, span);
            let mut stmts = vec![Stmt::Expr(id_expr)];
            finish_closure_body(p, &mut stmts, parse_expr_fn)?;
            return Ok(Expr::Closure {
                params: vec!["it".into()],
                body: stmts,
                span,
            });
        }
    }

    let mut body = Vec::new();
    finish_closure_body(p, &mut body, parse_expr_fn)?;
    Ok(Expr::Closure { params, body, span })
}

fn finish_closure_body(
    p: &mut ParserState<'_>,
    body: &mut Vec<Stmt>,
    _parse_expr_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
) -> Result<(), ParseError> {
    while !matches!(peek(p), Token::RBrace | Token::Eof) {
        while matches!(peek(p), Token::Newline | Token::Semicolon) {
            advance(p);
        }
        if matches!(peek(p), Token::RBrace | Token::Eof) {
            break;
        }
        let stmt = crate::parser::parse_stmt(p)?;
        body.push(stmt);
        while matches!(peek(p), Token::Newline | Token::Semicolon) {
            advance(p);
        }
    }
    expect(p, &Token::RBrace, "closing '}'")?;
    Ok(())
}
