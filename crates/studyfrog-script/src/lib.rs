//! FrogScript domain-specific language engine for StudyFrog.
//!
//! FrogScript provides study automation, card query and filtering pipelines (`|`),
//! active-recall queue steering, closure transformations, and dedicated execution logging.

pub mod ast;
pub mod env;
pub mod error;
pub mod eval;
pub mod eval_expr;
pub mod eval_pipe;
pub mod eval_stdlib;
pub mod expr_parser;
pub mod lexer;
pub mod lexer_scan;
pub mod log_stream;
pub mod parser;
pub mod parser_state;
pub mod pipe_parser;
pub mod primary_parser;
pub mod token;
pub mod val;

use ast::Program;
use env::{env_define, new_global_env};
use error::{LexError, ParseError, RuntimeError};
use eval::eval_program;
use eval_expr::EvalContext;
use log_stream::LogBuffer;
use val::{ScriptItem, Val};

/// Unified error enum for FrogScript compilation, parsing, and execution.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ScriptError {
    /// Error during tokenization.
    #[error(transparent)]
    Lex(#[from] LexError),
    /// Error during AST parsing.
    #[error(transparent)]
    Parse(#[from] ParseError),
    /// Error during runtime evaluation.
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}

/// Tokenize and parse FrogScript source text into an AST Program.
pub fn parse_frogscript(source: &str) -> Result<Program, ScriptError> {
    let tokens = lexer::tokenize(source)?;
    let program = parser::parse(&tokens)?;
    Ok(program)
}

/// Parse and execute FrogScript code against an optional sequence of vault items.
pub fn run_frogscript(
    source: &str,
    items: Vec<ScriptItem>,
    now_ms: i64,
) -> Result<(Val, LogBuffer), ScriptError> {
    let program = parse_frogscript(source)?;
    let mut env = new_global_env();

    // Inject initial items into environment
    if !items.is_empty() {
        let val_items = items.into_iter().map(Val::Item).collect();
        env_define(&mut env, "__vault_items", Val::List(val_items));
    }

    let mut logs = LogBuffer::default();
    let mut ctx = EvalContext {
        env: &mut env,
        logs: &mut logs,
        now_ms,
        fuel: 50_000,
    };

    let result = eval_program(&program, &mut ctx)?;
    Ok((result, logs))
}
