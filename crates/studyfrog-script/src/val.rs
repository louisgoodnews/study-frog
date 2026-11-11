//! Dynamic runtime values in the FrogScript execution engine.

use std::collections::HashMap;
use std::fmt;

/// An in-memory study item representation exposed to FrogScript.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScriptItem {
    /// Item UID if persisted in vault.
    pub uid: Option<String>,
    /// Item title.
    pub title: String,
    /// Front prompt text.
    pub prompt: String,
    /// Back answer text.
    pub answer: String,
    /// Item kind ("card", "note", "question").
    pub kind: String,
    /// Enclosing stack folder / path.
    pub stack: String,
    /// Tags attached to the item.
    pub tags: Vec<String>,
    /// Scheduling or study priority ("low", "medium", "high").
    pub priority: String,
    /// Difficulty tier ("low", "medium", "hard").
    pub difficulty: String,
    /// Familiarity measure from 0.0 to 1.0.
    pub familiarity: f64,
    /// Due epoch day number, if scheduled.
    pub due_day: Option<i64>,
}

/// Dynamic runtime value types in FrogScript.
#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    /// Void / empty value (statements producing no return).
    Void,
    /// Null / None literal value.
    Null,
    /// Boolean flag.
    Bool(bool),
    /// 64-bit integer.
    Int(i64),
    /// 64-bit floating point number.
    Float(f64),
    /// Duration in seconds.
    DurationSecs(i64),
    /// UTF-8 string.
    Str(String),
    /// Sequence / list of dynamic values.
    List(Vec<Val>),
    /// Map / dictionary of key-value pairs.
    Dict(HashMap<String, Val>),
    /// Domain study item (card, note, question).
    Item(ScriptItem),
    /// Built-in native function name.
    NativeFn(String),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, ""),
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::DurationSecs(s) => write!(f, "{s}s"),
            Self::Str(s) => write!(f, "{s}"),
            Self::List(l) => {
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Self::Dict(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{k}\": {v}")?;
                }
                write!(f, "}}")
            }
            Self::Item(it) => write!(f, "<item '{}' ({})>", it.title, it.kind),
            Self::NativeFn(name) => write!(f, "<function {name}>"),
        }
    }
}

/// Check if a runtime value is truthy in conditional checks.
pub fn is_truthy(v: &Val) -> bool {
    match v {
        Val::Void | Val::Null => false,
        Val::Bool(b) => *b,
        Val::Int(n) => *n != 0,
        Val::Float(f) => *f != 0.0 && !f.is_nan(),
        Val::DurationSecs(s) => *s > 0,
        Val::Str(s) => !s.is_empty(),
        Val::List(l) => !l.is_empty(),
        Val::Dict(d) => !d.is_empty(),
        Val::Item(_) | Val::NativeFn(_) => true,
    }
}

/// Retrieve a field value from a ScriptItem.
pub fn get_item_field(it: &ScriptItem, field: &str) -> Option<Val> {
    match field {
        "uid" => Some(it.uid.clone().map_or(Val::Null, Val::Str)),
        "title" => Some(Val::Str(it.title.clone())),
        "prompt" | "front" => Some(Val::Str(it.prompt.clone())),
        "answer" | "back" => Some(Val::Str(it.answer.clone())),
        "type" | "kind" => Some(Val::Str(it.kind.clone())),
        "stack" => Some(Val::Str(it.stack.clone())),
        "tags" => Some(Val::List(it.tags.iter().cloned().map(Val::Str).collect())),
        "priority" => Some(Val::Str(it.priority.clone())),
        "difficulty" => Some(Val::Str(it.difficulty.clone())),
        "familiarity" => Some(Val::Float(it.familiarity)),
        "due" | "due_day" => Some(it.due_day.map_or(Val::Null, Val::Int)),
        _ => None,
    }
}

/// Set a property on a ScriptItem.
pub fn set_item_field(it: &mut ScriptItem, field: &str, val: &Val) {
    match field {
        "title" => it.title = val.to_string(),
        "prompt" | "front" => it.prompt = val.to_string(),
        "answer" | "back" => it.answer = val.to_string(),
        "priority" => it.priority = val.to_string(),
        "difficulty" => it.difficulty = val.to_string(),
        "familiarity" => {
            if let Val::Float(f) = val {
                it.familiarity = *f;
            } else if let Val::Int(n) = val {
                it.familiarity = *n as f64;
            }
        }
        _ => {}
    }
}
