//! Config errors. `Display` delegates to a free function (P1).

use std::fmt;

/// A settings file that could not be used. The caller adds the file name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    /// 1-based line in the file, when known.
    pub line: Option<usize>,
    /// What is wrong.
    pub message: String,
}

/// Build an error.
pub fn config_error(line: Option<usize>, message: impl Into<String>) -> ConfigError {
    ConfigError {
        line,
        message: message.into(),
    }
}

/// Human-readable description.
pub fn describe_config_error(e: &ConfigError) -> String {
    match e.line {
        Some(line) => format!("line {line}: {}", e.message),
        None => e.message.clone(),
    }
}

/// 1-based line containing byte `offset`.
pub fn line_at(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// Convert a `toml` error, quoting the offending source line so the message names the key.
pub fn from_toml(text: &str, e: &toml::de::Error) -> ConfigError {
    let line = e.span().map(|s| line_at(text, s.start));
    let quoted = line
        .and_then(|l| text.lines().nth(l - 1))
        .map(str::trim)
        .filter(|l| !l.is_empty());
    let message = match quoted {
        Some(src) => format!("{} (`{src}`)", e.message()),
        None => e.message().to_owned(),
    };
    ConfigError { line, message }
}

/// 1-based line where `key` is assigned (first line starting with `key` followed by `=`).
pub fn line_of_key(text: &str, key: &str) -> Option<usize> {
    text.lines()
        .position(|l| {
            l.trim_start()
                .strip_prefix(key)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map(|i| i + 1)
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&describe_config_error(self))
    }
}

impl std::error::Error for ConfigError {}
