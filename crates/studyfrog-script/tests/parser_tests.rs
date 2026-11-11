#![allow(clippy::unwrap_used)]
//! Tests for the FrogScript AST parser.

use studyfrog_script::ast::{BinOp, Expr, Literal, PipeStage, Stmt};
use studyfrog_script::parse_frogscript;

#[test]
fn parses_user_pipeline_assignment() {
    let src = r#"cards = from stack "Neuroscience/Synapses" | where tag "cram-session""#;
    let prog = parse_frogscript(src).unwrap();

    assert_eq!(prog.statements.len(), 1);
    let Stmt::Assign { target, value, .. } = &prog.statements[0] else {
        panic!("expected assignment statement");
    };
    assert_eq!(target, "cards");

    let Expr::Pipeline { source, stages, .. } = value else {
        panic!("expected pipeline expression");
    };
    let Expr::From {
        target: stack_target,
        ..
    } = &**source
    else {
        panic!("expected from expression");
    };
    assert!(stack_target.is_some());
    assert_eq!(stages.len(), 1);
}

#[test]
fn parses_user_pipeline_mutation_and_filtering() {
    let src = "cards | set priority \"high\" | where familiarity < 0.7";
    let prog = parse_frogscript(src).unwrap();

    assert_eq!(prog.statements.len(), 1);
    let Stmt::Expr(Expr::Pipeline { source, stages, .. }) = &prog.statements[0] else {
        panic!("expected pipeline statement");
    };
    let Expr::Ident(name, _) = &**source else {
        panic!("expected identifier source");
    };
    assert_eq!(name, "cards");
    assert_eq!(stages.len(), 2);

    match &stages[0] {
        PipeStage::Set { field, .. } => assert_eq!(field, "priority"),
        _ => panic!("expected Set stage"),
    }
    match &stages[1] {
        PipeStage::Where(Expr::Binary { op, .. }) => assert_eq!(*op, BinOp::Lt),
        _ => panic!("expected Where stage with <"),
    }
}

#[test]
fn parses_semicolon_chained_lines() {
    let src = "a = 10; b = 20; log_info(a + b)";
    let prog = parse_frogscript(src).unwrap();

    assert_eq!(prog.statements.len(), 3);
    assert!(matches!(&prog.statements[0], Stmt::Assign { target, .. } if target == "a"));
    assert!(matches!(&prog.statements[1], Stmt::Assign { target, .. } if target == "b"));
    assert!(matches!(&prog.statements[2], Stmt::Expr(Expr::Call { .. })));
}

#[test]
fn parses_closure_pipelines() {
    let src = r#"
        from stack * | where type "card" | {
            card -> if card.difficulty == "low": card | set priority "low"
        }
    "#;
    let prog = parse_frogscript(src).unwrap();

    assert_eq!(prog.statements.len(), 1);
    let Stmt::Expr(Expr::Pipeline { stages, .. }) = &prog.statements[0] else {
        panic!("expected pipeline");
    };
    assert_eq!(stages.len(), 2);
    let PipeStage::Closure(Expr::Closure { params, body, .. }) = &stages[1] else {
        panic!("expected closure stage");
    };
    assert_eq!(params, &["card".to_string()]);
    assert!(!body.is_empty());
}

#[test]
fn parses_native_logging_calls() {
    let src = r#"log_error("Something went wrong!")"#;
    let prog = parse_frogscript(src).unwrap();

    assert_eq!(prog.statements.len(), 1);
    let Stmt::Expr(Expr::Call { callee, args, .. }) = &prog.statements[0] else {
        panic!("expected call");
    };
    let Expr::Ident(func, _) = &**callee else {
        panic!("expected ident");
    };
    assert_eq!(func, "log_error");
    assert_eq!(args.len(), 1);
    assert!(matches!(&args[0], Expr::Literal(Literal::Str(s), _) if s == "Something went wrong!"));
}
