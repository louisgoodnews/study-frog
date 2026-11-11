//! Parser helpers for pipeline stages (`| where ...`, `| set ...`, etc.).

use crate::ast::{Expr, PipeStage};
use crate::error::ParseError;
use crate::parser_state::{ParserState, advance, expect_ident, peek};
use crate::token::Token;

/// Parse a single pipeline stage following a pipe symbol `|`.
pub fn parse_pipe_stage(
    p: &mut ParserState<'_>,
    parse_expr_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
    parse_closure_fn: fn(&mut ParserState<'_>) -> Result<Expr, ParseError>,
) -> Result<PipeStage, ParseError> {
    match peek(p) {
        Token::Where => {
            advance(p);
            let cond = parse_expr_fn(p)?;
            Ok(PipeStage::Where(cond))
        }
        Token::Set => {
            advance(p);
            let (field, _) = expect_ident(p)?;
            let value = parse_expr_fn(p)?;
            Ok(PipeStage::Set { field, value })
        }
        Token::Tag => {
            advance(p);
            let tag = parse_expr_fn(p)?;
            Ok(PipeStage::Tag(tag))
        }
        Token::Untag => {
            advance(p);
            let tag = parse_expr_fn(p)?;
            Ok(PipeStage::Untag(tag))
        }
        Token::Sort => {
            advance(p);
            let (field, _) = expect_ident(p)?;
            let descending = if matches!(peek(p), Token::Ident(id) if id == "desc") {
                advance(p);
                true
            } else if matches!(peek(p), Token::Ident(id) if id == "asc") {
                advance(p);
                false
            } else {
                false
            };
            Ok(PipeStage::Sort { field, descending })
        }
        Token::Limit => {
            advance(p);
            let count = parse_expr_fn(p)?;
            Ok(PipeStage::Limit(count))
        }
        Token::Select => {
            advance(p);
            let mut fields = Vec::new();
            let (first, _) = expect_ident(p)?;
            fields.push(first);
            while matches!(peek(p), Token::Comma) {
                advance(p);
                let (next, _) = expect_ident(p)?;
                fields.push(next);
            }
            Ok(PipeStage::Select(fields))
        }
        Token::Export => {
            advance(p);
            let (kind, _) = expect_ident(p)?;
            let path = parse_expr_fn(p)?;
            Ok(PipeStage::Export { kind, path })
        }
        Token::LBrace => {
            let closure = parse_closure_fn(p)?;
            Ok(PipeStage::Closure(closure))
        }
        _ => {
            let expr = parse_expr_fn(p)?;
            Ok(PipeStage::Custom(expr))
        }
    }
}
