//! Pipeline stream query, transformation, and mutation evaluation.

use crate::ast::{Expr, PipeStage};
use crate::env::{env_define, new_child_env};
use crate::error::RuntimeError;
use crate::eval_expr::{EvalContext, eval_expr};
use crate::val::{ScriptItem, Val, is_truthy, set_item_field};

/// Evaluate a `from stack ...` or `from vault` query.
pub fn eval_from(target: Option<&Expr>, ctx: &mut EvalContext<'_>) -> Result<Val, RuntimeError> {
    let stack_filter = if let Some(t) = target {
        let v = eval_expr(t, ctx, None)?;
        match v {
            Val::Str(s) => Some(s),
            _ => None,
        }
    } else {
        None
    };

    // If an item sequence already exists in the environment under `__vault_items`, use it
    let items_val = crate::env::env_get(ctx.env, "__vault_items");
    let all_items = if let Some(Val::List(list)) = items_val {
        list.into_iter()
            .filter_map(|v| match v {
                Val::Item(it) => Some(it),
                _ => None,
            })
            .collect::<Vec<ScriptItem>>()
    } else {
        Vec::new()
    };

    let filtered = all_items
        .into_iter()
        .filter(|it| {
            if let Some(target_stack) = &stack_filter {
                if target_stack == "*" {
                    true
                } else {
                    it.stack == *target_stack || it.stack.starts_with(&format!("{target_stack}/"))
                }
            } else {
                true
            }
        })
        .map(Val::Item)
        .collect();

    Ok(Val::List(filtered))
}

/// Evaluate a pipeline chain `source | stage1 | stage2`.
pub fn eval_pipeline(
    source_expr: &Expr,
    stages: &[PipeStage],
    ctx: &mut EvalContext<'_>,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    let initial = eval_expr(source_expr, ctx, current_item)?;
    let mut current_items = match initial {
        Val::List(l) => l
            .into_iter()
            .filter_map(|v| match v {
                Val::Item(it) => Some(it),
                _ => None,
            })
            .collect::<Vec<ScriptItem>>(),
        Val::Item(it) => vec![it],
        _ => Vec::new(),
    };

    for stage in stages {
        match stage {
            PipeStage::Where(cond_expr) => {
                let mut kept = Vec::new();
                for it in current_items {
                    let res = eval_expr(cond_expr, ctx, Some(&it))?;
                    if is_truthy(&res) {
                        kept.push(it);
                    }
                }
                current_items = kept;
            }
            PipeStage::Set { field, value } => {
                for it in &mut current_items {
                    let v = eval_expr(value, ctx, Some(it))?;
                    set_item_field(it, field, &v);
                }
            }
            PipeStage::Tag(tag_expr) => {
                for it in &mut current_items {
                    let tag_val = eval_expr(tag_expr, ctx, Some(it))?;
                    let tag_str = tag_val.to_string();
                    if !it.tags.contains(&tag_str) {
                        it.tags.push(tag_str);
                    }
                }
            }
            PipeStage::Untag(tag_expr) => {
                for it in &mut current_items {
                    let tag_val = eval_expr(tag_expr, ctx, Some(it))?;
                    let tag_str = tag_val.to_string();
                    it.tags.retain(|t| t != &tag_str);
                }
            }
            PipeStage::Limit(limit_expr) => {
                let lim_val = eval_expr(limit_expr, ctx, None)?;
                if let Val::Int(n) = lim_val {
                    current_items.truncate((n.max(0)) as usize);
                }
            }
            PipeStage::Sort { field, descending } => {
                sort_items(&mut current_items, field, *descending);
            }
            PipeStage::Closure(closure_expr) => {
                execute_pipe_closure(closure_expr, &mut current_items, ctx)?;
            }
            _ => {}
        }
    }

    Ok(Val::List(
        current_items.into_iter().map(Val::Item).collect(),
    ))
}

fn sort_items(items: &mut [ScriptItem], field: &str, descending: bool) {
    items.sort_by(|a, b| {
        let ord = match field {
            "familiarity" => a
                .familiarity
                .partial_cmp(&b.familiarity)
                .unwrap_or(std::cmp::Ordering::Equal),
            "due" | "due_day" => a.due_day.cmp(&b.due_day),
            "title" => a.title.cmp(&b.title),
            _ => std::cmp::Ordering::Equal,
        };
        if descending { ord.reverse() } else { ord }
    });
}

fn execute_pipe_closure(
    closure_expr: &Expr,
    items: &mut [ScriptItem],
    ctx: &mut EvalContext<'_>,
) -> Result<(), RuntimeError> {
    let Expr::Closure { params, body, .. } = closure_expr else {
        return Ok(());
    };
    let param_name = params.first().cloned().unwrap_or_else(|| "it".into());

    for it in items {
        let mut child_env = new_child_env(ctx.env.clone());
        env_define(&mut child_env, &param_name, Val::Item(it.clone()));

        let mut child_ctx = EvalContext {
            env: &mut child_env,
            logs: ctx.logs,
            now_ms: ctx.now_ms,
            fuel: ctx.fuel,
        };

        for stmt in body {
            crate::eval::eval_stmt(stmt, &mut child_ctx, Some(it))?;
        }
        ctx.fuel = child_ctx.fuel;
    }
    Ok(())
}
