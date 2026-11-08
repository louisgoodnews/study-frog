//! Reading a control request: one line, either JSON (`{"id": 1, "cmd": "nav", "args": ["home"]}`,
//! or `{"cmd": "nav home"}`) or plain words (`nav home`). Pure; the words become commands in
//! `control_words`.

use serde_json::Value;

use crate::control_cmd::{Command, Request};
use crate::control_words::command_from_words;

/// Split a line into words: spaces separate, double quotes group, a backslash escapes the next
/// character. An unclosed quote is an error.
pub fn tokenize(line: &str) -> Result<Vec<String>, String> {
    let (mut words, mut word, mut quoted, mut started) = (Vec::new(), String::new(), false, false);
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                word.extend(chars.next());
                started = true;
            }
            '"' => (quoted, started) = (!quoted, true),
            c if c.is_whitespace() && !quoted => {
                if started {
                    words.push(std::mem::take(&mut word));
                }
                started = false;
            }
            c => {
                word.push(c);
                started = true;
            }
        }
    }
    if quoted {
        return Err("a quote is not closed".into());
    }
    if started {
        words.push(word);
    }
    Ok(words)
}

/// The first word of a line and the rest as written (for `query` and `search`, whose text is kept).
fn split_first(line: &str) -> (&str, &str) {
    let line = line.trim();
    line.split_once(char::is_whitespace)
        .map_or((line, ""), |(a, b)| (a, b.trim()))
}

/// The command a line of words stands for.
pub fn parse_words(line: &str) -> Result<Command, String> {
    match split_first(line) {
        ("query", rest) => Ok(Command::Query(rest.to_owned())),
        ("search", rest) => Ok(Command::Search(rest.to_owned())),
        _ => command_from_words(&tokenize(line)?),
    }
}

fn arg_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The command of a JSON request: `cmd` holds the first word (or the whole line), `args` the rest.
fn parse_json(v: &Value) -> Result<Command, String> {
    let cmd = v
        .get("cmd")
        .and_then(Value::as_str)
        .ok_or("the request has no \"cmd\"")?;
    let args: Vec<String> = match v.get("args") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) => a.iter().map(arg_text).collect(),
        Some(other) => vec![arg_text(other)],
    };
    let (first, rest) = split_first(cmd);
    if matches!(first, "query" | "search") {
        let text: Vec<&str> = std::iter::once(rest)
            .chain(args.iter().map(String::as_str))
            .filter(|s| !s.is_empty())
            .collect();
        return parse_words(&format!("{first} {}", text.join(" ")));
    }
    let mut words = tokenize(cmd)?;
    words.extend(args);
    command_from_words(&words)
}

/// Read one request line. The id is `null` for a line of words or JSON without one; a bad line
/// keeps the id it had, so its error can be matched to it.
pub fn parse_request(line: &str) -> Result<Request, (Value, String)> {
    let trimmed = line.trim();
    if trimmed.starts_with('{') {
        let v: Value =
            serde_json::from_str(trimmed).map_err(|e| (Value::Null, format!("bad JSON: {e}")))?;
        let id = v.get("id").cloned().unwrap_or(Value::Null);
        return parse_json(&v)
            .map(|cmd| Request {
                id: id.clone(),
                cmd,
            })
            .map_err(|e| (id, e));
    }
    parse_words(trimmed)
        .map(|cmd| Request {
            id: Value::Null,
            cmd,
        })
        .map_err(|e| (Value::Null, e))
}
