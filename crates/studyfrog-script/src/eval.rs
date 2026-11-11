//! Main statement evaluation and execution loop for FrogScript AST.

use crate::ast::{Program, Stmt};
use crate::env::{env_define, env_set, new_child_env};
use crate::error::RuntimeError;
use crate::eval_expr::{EvalContext, eval_expr};
use crate::val::{ScriptItem, Val, is_truthy};

/// Execute a complete AST Program, returning the value of the last evaluated statement.
pub fn eval_program(program: &Program, ctx: &mut EvalContext<'_>) -> Result<Val, RuntimeError> {
    let mut last_val = Val::Void;
    for stmt in &program.statements {
        last_val = eval_stmt(stmt, ctx, None)?;
    }
    Ok(last_val)
}

/// Execute a single statement in the given context.
pub fn eval_stmt(
    stmt: &Stmt,
    ctx: &mut EvalContext<'_>,
    current_item: Option<&mut ScriptItem>,
) -> Result<Val, RuntimeError> {
    match stmt {
        Stmt::Expr(expr) => eval_expr(expr, ctx, current_item.as_deref()),
        Stmt::Assign { target, value, .. } => {
            let evaluated = eval_expr(value, ctx, current_item.as_deref())?;
            env_set(ctx.env, target, evaluated.clone());
            Ok(evaluated)
        }
        Stmt::If {
            condition,
            then_body,
            elif_branches,
            else_body,
            ..
        } => {
            let cond_val = eval_expr(condition, ctx, current_item.as_deref())?;
            if is_truthy(&cond_val) {
                return eval_stmt_list(then_body, ctx, current_item);
            }
            for (elif_cond, elif_stmts) in elif_branches {
                let v = eval_expr(elif_cond, ctx, current_item.as_deref())?;
                if is_truthy(&v) {
                    return eval_stmt_list(elif_stmts, ctx, current_item);
                }
            }
            if let Some(else_stmts) = else_body {
                return eval_stmt_list(else_stmts, ctx, current_item);
            }
            Ok(Val::Void)
        }
        Stmt::For {
            item_var,
            iter,
            body,
            ..
        } => {
            let iter_val = eval_expr(iter, ctx, current_item.as_deref())?;
            let items = match iter_val {
                Val::List(l) => l,
                Val::Str(s) => s.chars().map(|c| Val::Str(c.to_string())).collect(),
                other => vec![other],
            };
            let mut last = Val::Void;
            for item in items {
                let mut loop_env = new_child_env(ctx.env.clone());
                env_define(&mut loop_env, item_var, item);
                let mut loop_ctx = EvalContext {
                    env: &mut loop_env,
                    logs: ctx.logs,
                    now_ms: ctx.now_ms,
                    fuel: ctx.fuel,
                };
                for s in body {
                    last = eval_stmt(s, &mut loop_ctx, None)?;
                }
                ctx.fuel = loop_ctx.fuel;
            }
            Ok(last)
        }
        Stmt::Def { .. } => Ok(Val::Void),
        Stmt::Return { value, .. } => {
            if let Some(expr) = value {
                eval_expr(expr, ctx, current_item.as_deref())
            } else {
                Ok(Val::Void)
            }
        }
    }
}

fn eval_stmt_list(
    stmts: &[Stmt],
    ctx: &mut EvalContext<'_>,
    mut current_item: Option<&mut ScriptItem>,
) -> Result<Val, RuntimeError> {
    let mut last = Val::Void;
    for s in stmts {
        last = eval_stmt(s, ctx, current_item.as_deref_mut())?;
    }
    Ok(last)
}
