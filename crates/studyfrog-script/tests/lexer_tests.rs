#![allow(clippy::unwrap_used)]
//! Tests for the FrogScript lexer and tokenizer.

use studyfrog_script::lexer::tokenize;
use studyfrog_script::token::Token;

#[test]
fn lexes_keywords_and_symbols() {
    let src =
        "from stack where type set tag untag sort limit select export | -> ; = == != < <= > >=";
    let tokens = tokenize(src).unwrap();
    let kinds: Vec<Token> = tokens.into_iter().map(|st| st.token).collect();

    assert_eq!(
        kinds,
        vec![
            Token::From,
            Token::Stack,
            Token::Where,
            Token::Type,
            Token::Set,
            Token::Tag,
            Token::Untag,
            Token::Sort,
            Token::Limit,
            Token::Select,
            Token::Export,
            Token::Pipe,
            Token::Arrow,
            Token::Semicolon,
            Token::Assign,
            Token::Eq,
            Token::NotEq,
            Token::Lt,
            Token::LtEq,
            Token::Gt,
            Token::GtEq,
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_numbers_and_duration_suffixes() {
    let src = "100 42.5 30s 15m 2h 7d";
    let tokens = tokenize(src).unwrap();
    let kinds: Vec<Token> = tokens.into_iter().map(|st| st.token).collect();

    assert_eq!(
        kinds,
        vec![
            Token::Int(100),
            Token::Float(42.5),
            Token::DurationSecs(30),
            Token::DurationSecs(900),
            Token::DurationSecs(7200),
            Token::DurationSecs(604800),
            Token::Eof,
        ]
    );
}

#[test]
fn lexes_strings_and_formatted_strings() {
    let src = r#" "hello \"world\"" 'single' f"count: {total}" "#;
    let tokens = tokenize(src).unwrap();
    let kinds: Vec<Token> = tokens.into_iter().map(|st| st.token).collect();

    assert_eq!(
        kinds,
        vec![
            Token::Str("hello \"world\"".into()),
            Token::Str("single".into()),
            Token::FormatStr("count: {total}".into()),
            Token::Eof,
        ]
    );
}

#[test]
fn ignores_comments_and_normalizes_newlines() {
    let src = "# python style\n// c style\nx = 1\n\n\ny = 2";
    let tokens = tokenize(src).unwrap();
    let kinds: Vec<Token> = tokens.into_iter().map(|st| st.token).collect();

    assert_eq!(
        kinds,
        vec![
            Token::Ident("x".into()),
            Token::Assign,
            Token::Int(1),
            Token::Newline,
            Token::Ident("y".into()),
            Token::Assign,
            Token::Int(2),
            Token::Eof,
        ]
    );
}
