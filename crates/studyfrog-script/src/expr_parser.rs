//! Recursive descent expression and pipeline parsing for FrogScript.

use crate::ast::{BinOp, Expr, UnOp};
use crate::error::ParseError;
use crate::parser_state::{ParserState, advance, current_span, peek};
use crate::pipe_parser::parse_pipe_stage;
use crate::primary_parser::{parse_closure, parse_postfix};
use crate::token::Token;

/// Parse any top-level expression (including pipelines).
pub fn parse_expr(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    parse_pipeline(p)
}

fn parse_pipeline(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let start_span = current_span(p);
    let mut left = parse_logical_or(p)?;

    if matches!(peek(p), Token::Pipe) {
        let mut stages = Vec::new();
        while matches!(peek(p), Token::Pipe) {
            advance(p);
            while matches!(peek(p), Token::Newline) {
                advance(p);
            }
            let stage = parse_pipe_stage(p, parse_logical_or, |p| parse_closure(p, parse_expr))?;
            stages.push(stage);
        }
        left = Expr::Pipeline {
            source: Box::new(left),
            stages,
            span: start_span,
        };
    }
    Ok(left)
}

pub(crate) fn parse_logical_or(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_logical_and(p)?;
    while matches!(peek(p), Token::Or) {
        advance(p);
        let right = parse_logical_and(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op: BinOp::Or,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_logical_and(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_equality(p)?;
    while matches!(peek(p), Token::And) {
        advance(p);
        let right = parse_equality(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op: BinOp::And,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_equality(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_relational(p)?;
    while matches!(peek(p), Token::Eq | Token::NotEq) {
        let op = if matches!(peek(p), Token::Eq) {
            BinOp::Eq
        } else {
            BinOp::NotEq
        };
        advance(p);
        let right = parse_relational(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_relational(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_term(p)?;
    while matches!(peek(p), Token::Lt | Token::LtEq | Token::Gt | Token::GtEq) {
        let op = match peek(p) {
            Token::Lt => BinOp::Lt,
            Token::LtEq => BinOp::LtEq,
            Token::Gt => BinOp::Gt,
            _ => BinOp::GtEq,
        };
        advance(p);
        let right = parse_term(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_term(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_factor(p)?;
    while matches!(peek(p), Token::Plus | Token::Minus) {
        let op = if matches!(peek(p), Token::Plus) {
            BinOp::Add
        } else {
            BinOp::Sub
        };
        advance(p);
        let right = parse_factor(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_factor(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    let mut left = parse_unary(p)?;
    while matches!(peek(p), Token::Star | Token::Slash | Token::Percent) {
        let op = match peek(p) {
            Token::Star => BinOp::Mul,
            Token::Slash => BinOp::Div,
            _ => BinOp::Mod,
        };
        advance(p);
        let right = parse_unary(p)?;
        let span = current_span(p);
        left = Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span,
        };
    }
    Ok(left)
}

fn parse_unary(p: &mut ParserState<'_>) -> Result<Expr, ParseError> {
    if matches!(peek(p), Token::Not) {
        advance(p);
        let span = current_span(p);
        let expr = parse_unary(p)?;
        return Ok(Expr::Unary {
            op: UnOp::Not,
            expr: Box::new(expr),
            span,
        });
    }
    if matches!(peek(p), Token::Minus) {
        advance(p);
        let span = current_span(p);
        let expr = parse_unary(p)?;
        return Ok(Expr::Unary {
            op: UnOp::Neg,
            expr: Box::new(expr),
            span,
        });
    }
    parse_postfix(p, parse_expr)
}
