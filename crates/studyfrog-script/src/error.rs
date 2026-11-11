//! Error types for FrogScript lexing, parsing, and execution.

use std::fmt;

/// Source position represented by line and column numbers (1-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SourcePos {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number.
    pub col: usize,
}

impl fmt::Display for SourcePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// A span of source code between two byte positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    /// Starting byte index.
    pub start: usize,
    /// Ending byte index (exclusive).
    pub end: usize,
    /// Human-readable source position of start.
    pub pos: SourcePos,
}

/// Error encountered during tokenization / lexical analysis.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LexError {
    /// An unexpected character was encountered.
    #[error("unexpected character '{ch}' at {pos}")]
    UnexpectedChar {
        /// The unexpected character.
        ch: char,
        /// Position in source.
        pos: SourcePos,
    },
    /// A string literal was unclosed before newline or EOF.
    #[error("unclosed string literal starting at {pos}")]
    UnclosedString {
        /// Start of string.
        pos: SourcePos,
    },
    /// Invalid numeric or duration format.
    #[error("invalid number or duration format '{text}' at {pos}")]
    InvalidNumber {
        /// The invalid literal string.
        text: String,
        /// Position in source.
        pos: SourcePos,
    },
}

/// Error encountered during syntactic analysis / AST parsing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// An expected token was missing.
    #[error("expected {expected}, found {found} at {pos}")]
    ExpectedToken {
        /// Expected token description.
        expected: String,
        /// Actual token found.
        found: String,
        /// Position in source.
        pos: SourcePos,
    },
    /// Unexpected end of input.
    #[error("unexpected end of file while parsing {context} (expected {expected})")]
    UnexpectedEof {
        /// The syntactic context being parsed.
        context: String,
        /// Expected token description.
        expected: String,
    },
    /// Invalid pipeline stage syntax.
    #[error("invalid pipeline stage '{stage}' at {pos}")]
    InvalidPipelineStage {
        /// The name or token of the invalid stage.
        stage: String,
        /// Position in source.
        pos: SourcePos,
    },
    /// An unexpected token was encountered.
    #[error("unexpected token '{token}' at {pos}")]
    UnexpectedToken {
        /// Token text.
        token: String,
        /// Position in source.
        pos: SourcePos,
    },
}

/// Error encountered during program evaluation / runtime execution.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RuntimeError {
    /// Referenced variable was not found in any enclosing scope.
    #[error("undefined variable '{name}'")]
    UndefinedVariable {
        /// Name of variable.
        name: String,
    },
    /// An operation was attempted on incompatible types.
    #[error("type error: {message}")]
    TypeError {
        /// Details.
        message: String,
    },
    /// Division or modulo by zero.
    #[error("division by zero")]
    DivisionByZero,
    /// Script exceeded maximum allowed execution step limit.
    #[error("execution timed out (exceeded {steps} steps)")]
    OutOfFuel {
        /// Limit that was exceeded.
        steps: usize,
    },
    /// Unknown function invocation.
    #[error("unknown function '{name}'")]
    UnknownFunction {
        /// Function name.
        name: String,
    },
}
