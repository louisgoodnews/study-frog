//! Severity levels, their names and parsing.

/// How severe a record is. `Error` is the most severe, so a record is enabled when its level is
/// less than or equal to the minimum level: `record_level <= minimum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Level {
    /// A failure that the caller or the user should see.
    Error = 0,
    /// Something unexpected that the program recovered from.
    Warn = 1,
    /// A normal event worth keeping. This is the default minimum.
    Info = 2,
    /// Details for a developer.
    Debug = 3,
    /// Fine-grained steps, such as checkpoints.
    Trace = 4,
}

/// The upper-case name of a level, as written to the log.
pub fn name(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => "WARN",
        Level::Info => "INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    }
}

/// Parses a level name, ignoring case and surrounding spaces. `warning` is accepted as well as `warn`.
pub fn parse_level(text: &str) -> Option<Level> {
    match text.trim().to_ascii_lowercase().as_str() {
        "error" => Some(Level::Error),
        "warn" | "warning" => Some(Level::Warn),
        "info" => Some(Level::Info),
        "debug" => Some(Level::Debug),
        "trace" => Some(Level::Trace),
        _ => None,
    }
}

/// Converts a stored discriminant back to a level. Values above the range read as `Trace`.
pub(crate) fn from_u8(value: u8) -> Level {
    match value {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        _ => Level::Trace,
    }
}
