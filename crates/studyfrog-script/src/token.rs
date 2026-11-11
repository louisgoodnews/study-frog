//! Token definitions and classifications for FrogScript lexical analysis.

use crate::error::Span;
use std::fmt;

/// Distinct syntactic tokens produced by the FrogScript lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    /// `from`
    From,
    /// `stack`
    Stack,
    /// `where`
    Where,
    /// `type`
    Type,
    /// `set`
    Set,
    /// `tag`
    Tag,
    /// `untag`
    Untag,
    /// `sort`
    Sort,
    /// `limit`
    Limit,
    /// `select`
    Select,
    /// `export`
    Export,
    /// `if`
    If,
    /// `elif`
    Elif,
    /// `else`
    Else,
    /// `for`
    For,
    /// `in`
    In,
    /// `def`
    Def,
    /// `return`
    Return,
    /// `and`
    And,
    /// `or`
    Or,
    /// `not`
    Not,
    /// `true`
    True,
    /// `false`
    False,
    /// `null` / `None`
    Null,

    // Literals
    /// Integer number literal.
    Int(i64),
    /// Floating point number literal.
    Float(f64),
    /// Duration literal normalized to seconds (e.g. 10m -> 600).
    DurationSecs(i64),
    /// Quoted string literal.
    Str(String),
    /// Formatted interpolated string (`f"..."`).
    FormatStr(String),
    /// Variable or function identifier.
    Ident(String),

    // Symbols & Operators
    /// Pipeline operator `|`
    Pipe,
    /// Closure arrow `->`
    Arrow,
    /// Semicolon `;`
    Semicolon,
    /// Comma `,`
    Comma,
    /// Colon `:`
    Colon,
    /// Dot `.`
    Dot,
    /// Assignment `=`
    Assign,
    /// Equal `==`
    Eq,
    /// Not equal `!=`
    NotEq,
    /// Less than `<`
    Lt,
    /// Less than or equal `<=`
    LtEq,
    /// Greater than `>`
    Gt,
    /// Greater than or equal `>=`
    GtEq,
    /// Plus `+`
    Plus,
    /// Minus `-`
    Minus,
    /// Asterisk `*` (multiplication or wildcard)
    Star,
    /// Slash `/`
    Slash,
    /// Percent `%`
    Percent,

    // Brackets
    /// Left parenthesis `(`
    LParen,
    /// Right parenthesis `)`
    RParen,
    /// Left square bracket `[`
    LBracket,
    /// Right square bracket `]`
    RBracket,
    /// Left curly brace `{`
    LBrace,
    /// Right curly brace `}`
    RBrace,

    // Layout
    /// Significant statement newline.
    Newline,
    /// End of file token.
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::From => write!(f, "from"),
            Self::Stack => write!(f, "stack"),
            Self::Where => write!(f, "where"),
            Self::Type => write!(f, "type"),
            Self::Set => write!(f, "set"),
            Self::Tag => write!(f, "tag"),
            Self::Untag => write!(f, "untag"),
            Self::Sort => write!(f, "sort"),
            Self::Limit => write!(f, "limit"),
            Self::Select => write!(f, "select"),
            Self::Export => write!(f, "export"),
            Self::If => write!(f, "if"),
            Self::Elif => write!(f, "elif"),
            Self::Else => write!(f, "else"),
            Self::For => write!(f, "for"),
            Self::In => write!(f, "in"),
            Self::Def => write!(f, "def"),
            Self::Return => write!(f, "return"),
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
            Self::Not => write!(f, "not"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n}"),
            Self::DurationSecs(s) => write!(f, "{s}s"),
            Self::Str(s) => write!(f, "\"{s}\""),
            Self::FormatStr(s) => write!(f, "f\"{s}\""),
            Self::Ident(id) => write!(f, "{id}"),
            Self::Pipe => write!(f, "|"),
            Self::Arrow => write!(f, "->"),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Dot => write!(f, "."),
            Self::Assign => write!(f, "="),
            Self::Eq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::LBrace => write!(f, "{{"),
            Self::RBrace => write!(f, "}}"),
            Self::Newline => write!(f, "<newline>"),
            Self::Eof => write!(f, "<eof>"),
        }
    }
}

/// A token bundled with its source code span.
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    /// Token kind.
    pub token: Token,
    /// Source span.
    pub span: Span,
}

/// Look up if an identifier string is a reserved keyword.
pub fn lookup_keyword(ident: &str) -> Option<Token> {
    match ident {
        "from" => Some(Token::From),
        "stack" => Some(Token::Stack),
        "where" => Some(Token::Where),
        "type" => Some(Token::Type),
        "set" => Some(Token::Set),
        "tag" => Some(Token::Tag),
        "untag" => Some(Token::Untag),
        "sort" => Some(Token::Sort),
        "limit" => Some(Token::Limit),
        "select" => Some(Token::Select),
        "export" => Some(Token::Export),
        "if" => Some(Token::If),
        "elif" => Some(Token::Elif),
        "else" => Some(Token::Else),
        "for" => Some(Token::For),
        "in" => Some(Token::In),
        "def" => Some(Token::Def),
        "return" => Some(Token::Return),
        "and" => Some(Token::And),
        "or" => Some(Token::Or),
        "not" => Some(Token::Not),
        "true" => Some(Token::True),
        "false" => Some(Token::False),
        "null" | "None" => Some(Token::Null),
        _ => None,
    }
}
