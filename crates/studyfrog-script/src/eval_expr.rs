//! Expression evaluation for FrogScript.

use crate::ast::{BinOp, Expr, Literal, UnOp};
use crate::env::{Env, env_get};
use crate::error::RuntimeError;
use crate::eval_stdlib::call_native_fn;
use crate::log_stream::LogBuffer;
use crate::val::{ScriptItem, Val, get_item_field, is_truthy};

/// Runtime context holding variable environment, logs, and current time.
pub struct EvalContext<'a> {
    /// Active variable scope.
    pub env: &'a mut Env,
    /// Destination log buffer for stdout and stderr.
    pub logs: &'a mut LogBuffer,
    /// Current system time in epoch milliseconds.
    pub now_ms: i64,
    /// Remaining execution steps before fuel exhaustion.
    pub fuel: usize,
}

/// Consume one step of execution fuel, returning an error if exhausted.
pub fn consume_fuel(ctx: &mut EvalContext<'_>) -> Result<(), RuntimeError> {
    if ctx.fuel == 0 {
        return Err(RuntimeError::OutOfFuel { steps: 50_000 });
    }
    ctx.fuel -= 1;
    Ok(())
}

/// Evaluate an AST expression producing a dynamic runtime value.
pub fn eval_expr(
    expr: &Expr,
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    consume_fuel(ctx)?;
    match expr {
        Expr::Literal(lit, _) => Ok(eval_literal(lit)),
        Expr::Ident(name, _) => eval_ident(name, ctx, current_item),
        Expr::Binary {
            op, left, right, ..
        } => eval_binary(*op, left, right, ctx, current_item),
        Expr::Unary { op, expr, .. } => eval_unary(*op, expr, ctx, current_item),
        Expr::Call { callee, args, .. } => eval_call(callee, args, ctx, current_item),
        Expr::FieldAccess { target, field, .. } => {
            let obj = eval_expr(target, ctx, current_item)?;
            match obj {
                Val::Item(it) => {
                    get_item_field(&it, field).ok_or_else(|| RuntimeError::TypeError {
                        message: format!("item has no property '{field}'"),
                    })
                }
                Val::Dict(d) => Ok(d.get(field).cloned().unwrap_or(Val::Null)),
                other => Err(RuntimeError::TypeError {
                    message: format!("cannot access property '{field}' on type '{}'", other),
                }),
            }
        }
        Expr::List(items, _) => {
            let mut vals = Vec::new();
            for it in items {
                vals.push(eval_expr(it, ctx, current_item)?);
            }
            Ok(Val::List(vals))
        }
        Expr::Dict(pairs, _) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in pairs {
                let k_val = eval_expr(k, ctx, current_item)?;
                let v_val = eval_expr(v, ctx, current_item)?;
                map.insert(k_val.to_string(), v_val);
            }
            Ok(Val::Dict(map))
        }
        Expr::Pipeline { source, stages, .. } => {
            crate::eval_pipe::eval_pipeline(source, stages, ctx, current_item)
        }
        Expr::From { target, .. } => crate::eval_pipe::eval_from(target.as_deref(), ctx),
        Expr::Closure { params, .. } => {
            // Evaluates to a callable closure reference (represented in Val)
            Ok(Val::NativeFn(format!("closure_{}", params.join(","))))
        }
    }
}

fn eval_literal(lit: &Literal) -> Val {
    match lit {
        Literal::Int(n) => Val::Int(*n),
        Literal::Float(f) => Val::Float(*f),
        Literal::DurationSecs(s) => Val::DurationSecs(*s),
        Literal::Str(s) => Val::Str(s.clone()),
        Literal::FormatStr(s) => Val::Str(s.clone()),
        Literal::Bool(b) => Val::Bool(*b),
        Literal::Null => Val::Null,
        Literal::Wildcard => Val::Str("*".into()),
    }
}

fn eval_ident(
    name: &str,
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    if let Some(val) = env_get(ctx.env, name) {
        return Ok(val);
    }
    // Check if current item exposes this as property
    if let Some(it) = current_item
        && let Some(field_val) = get_item_field(it, name)
    {
        return Ok(field_val);
    }
    Err(RuntimeError::UndefinedVariable {
        name: name.to_string(),
    })
}

fn eval_binary(
    op: BinOp,
    left_expr: &Expr,
    right_expr: &Expr,
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    let left = eval_expr(left_expr, ctx, current_item)?;

    // Short-circuit logical operators
    if op == BinOp::And {
        return if !is_truthy(&left) {
            Ok(Val::Bool(false))
        } else {
            let right = eval_expr(right_expr, ctx, current_item)?;
            Ok(Val::Bool(is_truthy(&right)))
        };
    }
    if op == BinOp::Or {
        return if is_truthy(&left) {
            Ok(Val::Bool(true))
        } else {
            let right = eval_expr(right_expr, ctx, current_item)?;
            Ok(Val::Bool(is_truthy(&right)))
        };
    }

    let right = eval_expr(right_expr, ctx, current_item)?;
    match (op, &left, &right) {
        (BinOp::Add, Val::Int(a), Val::Int(b)) => Ok(Val::Int(a + b)),
        (BinOp::Add, Val::Float(a), Val::Float(b)) => Ok(Val::Float(a + b)),
        (BinOp::Add, Val::Str(a), b) => Ok(Val::Str(format!("{a}{b}"))),
        (BinOp::Sub, Val::Int(a), Val::Int(b)) => Ok(Val::Int(a - b)),
        (BinOp::Sub, Val::Float(a), Val::Float(b)) => Ok(Val::Float(a - b)),
        (BinOp::Mul, Val::Int(a), Val::Int(b)) => Ok(Val::Int(a * b)),
        (BinOp::Mul, Val::Float(a), Val::Float(b)) => Ok(Val::Float(a * b)),
        (BinOp::Div, _, Val::Int(0)) => Err(RuntimeError::DivisionByZero),
        (BinOp::Div, Val::Int(a), Val::Int(b)) => Ok(Val::Int(a / b)),
        (BinOp::Div, Val::Float(a), Val::Float(b)) => {
            if *b == 0.0 {
                Err(RuntimeError::DivisionByZero)
            } else {
                Ok(Val::Float(a / b))
            }
        }
        (BinOp::Eq, a, b) => Ok(Val::Bool(a == b)),
        (BinOp::NotEq, a, b) => Ok(Val::Bool(a != b)),
        (BinOp::Lt, Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a < b)),
        (BinOp::Lt, Val::Float(a), Val::Float(b)) => Ok(Val::Bool(a < b)),
        (BinOp::Lt, Val::Float(a), Val::Int(b)) => Ok(Val::Bool(*a < *b as f64)),
        (BinOp::Lt, Val::Int(a), Val::Float(b)) => Ok(Val::Bool((*a as f64) < *b)),
        (BinOp::LtEq, Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a <= b)),
        (BinOp::LtEq, Val::Float(a), Val::Float(b)) => Ok(Val::Bool(a <= b)),
        (BinOp::Gt, Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a > b)),
        (BinOp::Gt, Val::Float(a), Val::Float(b)) => Ok(Val::Bool(a > b)),
        (BinOp::Gt, Val::Float(a), Val::Int(b)) => Ok(Val::Bool(*a > *b as f64)),
        (BinOp::Gt, Val::Int(a), Val::Float(b)) => Ok(Val::Bool((*a as f64) > *b)),
        (BinOp::GtEq, Val::Int(a), Val::Int(b)) => Ok(Val::Bool(a >= b)),
        (BinOp::GtEq, Val::Float(a), Val::Float(b)) => Ok(Val::Bool(a >= b)),
        _ => Err(RuntimeError::TypeError {
            message: format!(
                "unsupported binary operation {:?} on {} and {}",
                op, left, right
            ),
        }),
    }
}

fn eval_unary(
    op: UnOp,
    expr: &Expr,
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    let val = eval_expr(expr, ctx, current_item)?;
    match (op, val) {
        (UnOp::Not, v) => Ok(Val::Bool(!is_truthy(&v))),
        (UnOp::Neg, Val::Int(n)) => Ok(Val::Int(-n)),
        (UnOp::Neg, Val::Float(f)) => Ok(Val::Float(-f)),
        _ => Err(RuntimeError::TypeError {
            message: format!("unsupported unary operation {:?}", op),
        }),
    }
}

fn eval_call(
    callee: &Expr,
    args: &[Expr],
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    let mut evaluated_args = Vec::new();
    for a in args {
        evaluated_args.push(eval_expr(a, ctx, current_item)?);
    }

    if let Expr::Ident(func_name, _) = callee {
        return call_native_fn(
            func_name,
            &evaluated_args,
            ctx.logs,
            ctx.now_ms,
            current_item,
        );
    }
    let target = eval_expr(callee, ctx, current_item)?;
    if let Val::NativeFn(func_name) = target {
        return call_native_fn(
            &func_name,
            &evaluated_args,
            ctx.logs,
            ctx.now_ms,
            current_item,
        );
    }
    Err(RuntimeError::TypeError {
        message: format!("cannot invoke non-callable value '{}'", target),
    })
}
