//! The application log: every notice as a line in `studyfrog.log` and as JSON in `console.jsonl`,
//! each file rotated by size. Logging must never get in the way, so callers may ignore its errors.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde_json::json;
use studyfrog_config::paths::{LOG_JSON_FILE, LOG_TEXT_FILE};
use studyfrog_core::time::UnixMs;
use studyfrog_md::rfc3339::format_rfc3339;

/// A log file is rotated once it reaches this size.
pub const MAX_LOG_BYTES: u64 = 5 * 1024 * 1024;
/// Files kept per log: the current one and the rotated ones.
pub const KEPT_FILES: usize = 3;

/// How serious an entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Something worked.
    Info,
    /// Something is off but went on.
    Warn,
    /// Something failed.
    Error,
}

/// One line of the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// When it happened, in Unix milliseconds.
    pub at_ms: i64,
    /// How serious it is.
    pub level: LogLevel,
    /// Which part of the application wrote it.
    pub source: String,
    /// What happened.
    pub message: String,
}

/// The lower-case name of a level, as written to the JSON log.
pub fn level_name(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    }
}

fn stamp(at_ms: i64) -> String {
    format_rfc3339(UnixMs(at_ms)).unwrap_or_else(|| at_ms.to_string())
}

/// The entry as one line of `studyfrog.log`; line breaks inside the message are written as `\n`.
pub fn text_line(e: &LogEntry) -> String {
    let message = e.message.replace('\r', "").replace('\n', "\\n");
    let level = level_name(e.level).to_uppercase();
    format!("{} {level:<5} {}: {message}", stamp(e.at_ms), e.source)
}

/// The entry as one line of `console.jsonl`.
pub fn json_line(e: &LogEntry) -> String {
    json!({
        "at": stamp(e.at_ms),
        "at_ms": e.at_ms,
        "level": level_name(e.level),
        "source": e.source,
        "message": e.message,
    })
    .to_string()
}

/// The name of the `n`th rotated copy of `path`: `studyfrog.log.1`.
pub fn rotated_name(path: &Path, n: usize) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{n}"));
    PathBuf::from(name)
}

/// The renames that make room for a fresh file, oldest first (`log.1` to `log.2`, then `log` to
/// `log.1`). With `kept` files the oldest copy is overwritten. At least two files are kept.
pub fn rotation_plan(path: &Path, kept: usize) -> Vec<(PathBuf, PathBuf)> {
    let older = (1..kept.max(2) - 1)
        .rev()
        .map(|n| (rotated_name(path, n), rotated_name(path, n + 1)));
    older
        .chain(std::iter::once((path.to_path_buf(), rotated_name(path, 1))))
        .collect()
}

fn rotate_if_full(path: &Path, max_bytes: u64) -> io::Result<()> {
    let full = fs::metadata(path).is_ok_and(|m| m.len() >= max_bytes);
    if !full {
        return Ok(());
    }
    for (from, to) in rotation_plan(path, KEPT_FILES) {
        if from.exists() {
            let _ = fs::remove_file(&to);
            fs::rename(&from, &to)?;
        }
    }
    Ok(())
}

fn append_line(path: &Path, line: &str, max_bytes: u64) -> io::Result<()> {
    rotate_if_full(path, max_bytes)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")
}

/// Write `entry` to both logs in `dir`, creating it, and rotate a log that has reached `max_bytes`.
pub fn log_append(dir: &Path, entry: &LogEntry, max_bytes: u64) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    append_line(&dir.join(LOG_TEXT_FILE), &text_line(entry), max_bytes)?;
    append_line(&dir.join(LOG_JSON_FILE), &json_line(entry), max_bytes)
}

/// The last `n` lines of the log at `path`, oldest first. A missing file has no lines.
pub fn tail_lines(path: &Path, n: usize) -> io::Result<Vec<String>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<String> = text.lines().rev().take(n).map(str::to_owned).collect();
    lines.reverse();
    Ok(lines)
}
