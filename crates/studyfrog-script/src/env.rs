//! Execution scope and symbol environment for FrogScript variables and functions.

use crate::val::Val;
use std::collections::HashMap;

/// An environment frame storing variable bindings and an optional parent scope.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Env {
    bindings: HashMap<String, Val>,
    parent: Option<Box<Env>>,
}

/// Create a new top-level environment pre-populated with standard built-in functions.
pub fn new_global_env() -> Env {
    let mut bindings = HashMap::new();
    bindings.insert("print".into(), Val::NativeFn("print".into()));
    bindings.insert("log_info".into(), Val::NativeFn("log_info".into()));
    bindings.insert("log_warn".into(), Val::NativeFn("log_warn".into()));
    bindings.insert("log_error".into(), Val::NativeFn("log_error".into()));
    bindings.insert("len".into(), Val::NativeFn("len".into()));
    bindings.insert("now".into(), Val::NativeFn("now".into()));
    bindings.insert("today".into(), Val::NativeFn("today".into()));
    bindings.insert("days_ago".into(), Val::NativeFn("days_ago".into()));
    bindings.insert("due_today".into(), Val::NativeFn("due_today".into()));
    bindings.insert("tag".into(), Val::NativeFn("tag".into()));
    bindings.insert("type".into(), Val::NativeFn("type".into()));
    bindings.insert("difficulty".into(), Val::NativeFn("difficulty".into()));
    bindings.insert("priority".into(), Val::NativeFn("priority".into()));
    Env {
        bindings,
        parent: None,
    }
}

/// Create a nested child scope with a reference to the enclosing parent environment.
pub fn new_child_env(parent: Env) -> Env {
    Env {
        bindings: HashMap::new(),
        parent: Some(Box::new(parent)),
    }
}

/// Look up a variable in the environment hierarchy.
pub fn env_get(env: &Env, name: &str) -> Option<Val> {
    if let Some(val) = env.bindings.get(name) {
        Some(val.clone())
    } else if let Some(parent) = &env.parent {
        env_get(parent, name)
    } else {
        None
    }
}

/// Bind a variable directly in the current scope.
pub fn env_define(env: &mut Env, name: &str, val: Val) {
    env.bindings.insert(name.to_string(), val);
}

/// Update an existing variable in the scope hierarchy, or define it locally if not found.
pub fn env_set(env: &mut Env, name: &str, val: Val) {
    if env.bindings.contains_key(name) {
        env.bindings.insert(name.to_string(), val);
    } else if let Some(parent) = &mut env.parent {
        env_set(parent, name, val);
    } else {
        env.bindings.insert(name.to_string(), val);
    }
}
