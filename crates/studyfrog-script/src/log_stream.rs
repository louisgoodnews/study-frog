//! Dedicated internal stdout and stderr log capturing streams for user scripts.

use std::fmt;

/// Distinction between standard output and diagnostic error output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    /// Standard application and print stream (`stdout`).
    Stdout,
    /// Diagnostic error stream (`stderr`).
    Stderr,
}

/// Severity classification of a log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Informational message (`print`, `log_info`).
    Info,
    /// Warning diagnostic (`log_warn`).
    Warn,
    /// Error trace or alert (`log_error`).
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A recorded log message with stream, level, and timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Destination stream (`stdout` or `stderr`).
    pub stream: StreamKind,
    /// Log severity level.
    pub level: LogLevel,
    /// Text message content.
    pub message: String,
    /// Epoch millisecond timestamp when emitted.
    pub timestamp_ms: i64,
}

/// In-memory ring/append buffer capturing stdout and stderr for a script execution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogBuffer {
    /// Sequence of captured log entries.
    pub entries: Vec<LogEntry>,
}

/// Append an informational message to stdout.
pub fn log_info(buf: &mut LogBuffer, msg: &str, now_ms: i64) {
    buf.entries.push(LogEntry {
        stream: StreamKind::Stdout,
        level: LogLevel::Info,
        message: msg.to_string(),
        timestamp_ms: now_ms,
    });
}

/// Append a warning diagnostic to stdout.
pub fn log_warn(buf: &mut LogBuffer, msg: &str, now_ms: i64) {
    buf.entries.push(LogEntry {
        stream: StreamKind::Stdout,
        level: LogLevel::Warn,
        message: msg.to_string(),
        timestamp_ms: now_ms,
    });
}

/// Append an error alert to stderr.
pub fn log_error(buf: &mut LogBuffer, msg: &str, now_ms: i64) {
    buf.entries.push(LogEntry {
        stream: StreamKind::Stderr,
        level: LogLevel::Error,
        message: msg.to_string(),
        timestamp_ms: now_ms,
    });
}

/// Retrieve all stdout lines.
pub fn get_stdout_messages(buf: &LogBuffer) -> Vec<&str> {
    buf.entries
        .iter()
        .filter(|e| e.stream == StreamKind::Stdout)
        .map(|e| e.message.as_str())
        .collect()
}

/// Retrieve all stderr lines.
pub fn get_stderr_messages(buf: &LogBuffer) -> Vec<&str> {
    buf.entries
        .iter()
        .filter(|e| e.stream == StreamKind::Stderr)
        .map(|e| e.message.as_str())
        .collect()
}
