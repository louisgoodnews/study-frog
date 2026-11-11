//! Main statement and program parsing for FrogScript.

use crate::ast::{Program, Stmt};
use crate::error::ParseError;
use crate::expr_parser::parse_expr;
use crate::parser_state::{
    ParserState, advance, current_span, expect, expect_ident, is_at_end, new_state, peek,
    skip_separators,
};
use crate::token::{SpannedToken, Token};

/// Parses a token stream into an AST Program.
pub fn parse(tokens: &[SpannedToken]) -> Result<Program, ParseError> {
    let mut state = new_state(tokens);
    parse_program(&mut state)
}

fn parse_program(p: &mut ParserState<'_>) -> Result<Program, ParseError> {
    let mut statements = Vec::new();
    skip_separators(p);

    while !is_at_end(p) {
        let stmt = parse_stmt(p)?;
        statements.push(stmt);
        skip_separators(p);
    }

    Ok(Program { statements })
}

pub(crate) fn parse_stmt(p: &mut ParserState<'_>) -> Result<Stmt, ParseError> {
    match peek(p) {
        Token::If => parse_if(p),
        Token::For => parse_for(p),
        Token::Def => parse_def(p),
        Token::Return => parse_return(p),
        Token::Ident(_) if matches!(crate::parser_state::peek_next(p), Token::Assign) => {
            let (target, span) = expect_ident(p)?;
            expect(p, &Token::Assign, "=")?;
            let value = parse_expr(p)?;
            Ok(Stmt::Assign {
                target,
                value,
                span,
            })
        }
        _ => {
            let expr = parse_expr(p)?;
            Ok(Stmt::Expr(expr))
        }
    }
}

fn parse_if(p: &mut ParserState<'_>) -> Result<Stmt, ParseError> {
    let span = current_span(p);
    expect(p, &Token::If, "if")?;
    let condition = parse_expr(p)?;
    if matches!(peek(p), Token::Colon) {
        advance(p);
    }
    let then_body = parse_block(p)?;

    let mut elif_branches = Vec::new();
    while matches!(peek(p), Token::Elif) {
        advance(p);
        let elif_cond = parse_expr(p)?;
        if matches!(peek(p), Token::Colon) {
            advance(p);
        }
        let elif_body = parse_block(p)?;
        elif_branches.push((elif_cond, elif_body));
    }

    let mut else_body = None;
    if matches!(peek(p), Token::Else) {
        advance(p);
        if matches!(peek(p), Token::Colon) {
            advance(p);
        }
        else_body = Some(parse_block(p)?);
    }

    Ok(Stmt::If {
        condition,
        then_body,
        elif_branches,
        else_body,
        span,
    })
}

fn parse_for(p: &mut ParserState<'_>) -> Result<Stmt, ParseError> {
    let span = current_span(p);
    expect(p, &Token::For, "for")?;
    let (item_var, _) = expect_ident(p)?;
    let mut index_var = None;
    if matches!(peek(p), Token::Comma) {
        advance(p);
        let (idx, _) = expect_ident(p)?;
        index_var = Some(idx);
    }
    expect(p, &Token::In, "in")?;
    let iter = parse_expr(p)?;
    if matches!(peek(p), Token::Colon) {
        advance(p);
    }
    let body = parse_block(p)?;

    Ok(Stmt::For {
        item_var,
        index_var,
        iter,
        body,
        span,
    })
}

fn parse_def(p: &mut ParserState<'_>) -> Result<Stmt, ParseError> {
    let span = current_span(p);
    expect(p, &Token::Def, "def")?;
    let (name, _) = expect_ident(p)?;
    expect(p, &Token::LParen, "(")?;
    let mut params = Vec::new();
    if !matches!(peek(p), Token::RParen) {
        let (first, _) = expect_ident(p)?;
        params.push(first);
        while matches!(peek(p), Token::Comma) {
            advance(p);
            if matches!(peek(p), Token::RParen) {
                break;
            }
            let (next, _) = expect_ident(p)?;
            params.push(next);
        }
    }
    expect(p, &Token::RParen, ")")?;
    if matches!(peek(p), Token::Colon) {
        advance(p);
    }
    let body = parse_block(p)?;

    Ok(Stmt::Def {
        name,
        params,
        body,
        span,
    })
}

fn parse_return(p: &mut ParserState<'_>) -> Result<Stmt, ParseError> {
    let span = current_span(p);
    expect(p, &Token::Return, "return")?;
    let value = if matches!(
        peek(p),
        Token::Newline | Token::Semicolon | Token::Eof | Token::RBrace
    ) {
        None
    } else {
        Some(parse_expr(p)?)
    };
    Ok(Stmt::Return { value, span })
}

fn parse_block(p: &mut ParserState<'_>) -> Result<Vec<Stmt>, ParseError> {
    skip_separators(p);
    if matches!(peek(p), Token::LBrace) {
        advance(p);
        let mut stmts = Vec::new();
        while !matches!(peek(p), Token::RBrace | Token::Eof) {
            skip_separators(p);
            if matches!(peek(p), Token::RBrace | Token::Eof) {
                break;
            }
            stmts.push(parse_stmt(p)?);
            skip_separators(p);
        }
        expect(p, &Token::RBrace, "}")?;
        Ok(stmts)
    } else {
        // Single statement body
        let stmt = parse_stmt(p)?;
        Ok(vec![stmt])
    }
}
