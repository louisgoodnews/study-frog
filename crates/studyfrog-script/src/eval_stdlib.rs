//! Standard library routines and built-in functions for FrogScript.

use crate::error::RuntimeError;
use crate::log_stream::{LogBuffer, log_error, log_info, log_warn};
use crate::val::{ScriptItem, Val};

/// Dispatch a call to a built-in standard library function.
pub fn call_native_fn(
    name: &str,
    args: &[Val],
    logs: &mut LogBuffer,
    now_ms: i64,
    current_item: Option<&ScriptItem>,
) -> Result<Val, RuntimeError> {
    match name {
        "print" => {
            let out = args
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            log_info(logs, &out, now_ms);
            Ok(Val::Void)
        }
        "log_info" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            log_info(logs, &msg, now_ms);
            Ok(Val::Void)
        }
        "log_warn" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            log_warn(logs, &msg, now_ms);
            Ok(Val::Void)
        }
        "log_error" => {
            let msg = args.first().map(|v| v.to_string()).unwrap_or_default();
            log_error(logs, &msg, now_ms);
            Ok(Val::Void)
        }
        "len" => {
            let Some(first) = args.first() else {
                return Err(RuntimeError::TypeError {
                    message: "len() takes exactly 1 argument".into(),
                });
            };
            let count = match first {
                Val::List(l) => l.len() as i64,
                Val::Dict(d) => d.len() as i64,
                Val::Str(s) => s.len() as i64,
                _ => {
                    return Err(RuntimeError::TypeError {
                        message: format!("object of type '{}' has no len()", first),
                    });
                }
            };
            Ok(Val::Int(count))
        }
        "now" => Ok(Val::Int(now_ms)),
        "today" => Ok(Val::Int(now_ms / 86_400_000)),
        "days_ago" => {
            let days = args
                .first()
                .and_then(|v| match v {
                    Val::Int(n) => Some(*n),
                    _ => None,
                })
                .unwrap_or(0);
            let today = now_ms / 86_400_000;
            Ok(Val::Int(today - days))
        }
        "tag" => {
            let tag_name = args.first().map(|v| v.to_string()).unwrap_or_default();
            let matched = current_item.is_some_and(|it| it.tags.contains(&tag_name));
            Ok(Val::Bool(matched))
        }
        "type" => {
            let kind = args.first().map(|v| v.to_string()).unwrap_or_default();
            let matched = current_item.is_some_and(|it| it.kind == kind);
            Ok(Val::Bool(matched))
        }
        "difficulty" => {
            let diff = args.first().map(|v| v.to_string()).unwrap_or_default();
            let matched = current_item.is_some_and(|it| it.difficulty.eq_ignore_ascii_case(&diff));
            Ok(Val::Bool(matched))
        }
        "priority" => {
            let prio = args.first().map(|v| v.to_string()).unwrap_or_default();
            let matched = current_item.is_some_and(|it| it.priority.eq_ignore_ascii_case(&prio));
            Ok(Val::Bool(matched))
        }
        unknown => Err(RuntimeError::UnknownFunction {
            name: unknown.to_string(),
        }),
    }
}
