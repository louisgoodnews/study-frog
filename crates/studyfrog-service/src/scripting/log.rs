//! How a script run is written to the application log (source `script`): one entry for the outcome,
//! the tail of what it printed on standard error as a second one when it failed.

use std::path::Path;

use crate::logging::{LogEntry, LogLevel, MAX_LOG_BYTES, log_append};

use super::exec::{ScriptOutcome, succeeded};

/// Characters of output kept in a log entry.
pub const LOG_OUTPUT_CHARS: usize = 2000;

/// The last `n` characters of `text`, trimmed.
pub fn tail_chars(text: &str, n: usize) -> String {
    let t = text.trim();
    let skip = t.chars().count().saturating_sub(n);
    t.chars().skip(skip).collect()
}

/// One line saying how the run of `job` ended.
pub fn outcome_line(job: &str, o: &ScriptOutcome) -> String {
    if let Some(e) = &o.spawn_error {
        return format!("script {job} could not start: {e}");
    }
    if o.timed_out {
        return format!(
            "script {job} ran over its timeout and was stopped after {} ms",
            o.millis
        );
    }
    match o.code {
        Some(0) => format!("script {job} finished in {} ms", o.millis),
        Some(c) => format!(
            "script {job} failed with exit code {c} after {} ms",
            o.millis
        ),
        None => format!("script {job} was ended by a signal after {} ms", o.millis),
    }
}

/// The log entries of a run: the outcome (with what it printed), and its errors when it failed.
pub fn outcome_entries(job: &str, o: &ScriptOutcome, at_ms: i64) -> Vec<LogEntry> {
    let ok = succeeded(o);
    let out = tail_chars(&o.stdout, LOG_OUTPUT_CHARS);
    let mut message = outcome_line(job, o);
    if !out.is_empty() {
        message = format!("{message}\n{out}");
    }
    let entry = |level, message| LogEntry {
        at_ms,
        level,
        source: "script".to_owned(),
        message,
    };
    let mut entries = vec![entry(
        if ok { LogLevel::Info } else { LogLevel::Error },
        message,
    )];
    let err = tail_chars(&o.stderr, LOG_OUTPUT_CHARS);
    if !err.is_empty() {
        let level = if ok { LogLevel::Warn } else { LogLevel::Error };
        entries.push(entry(
            level,
            format!("script {job} wrote to standard error:\n{err}"),
        ));
    }
    entries
}

/// Append the run's entries to the log in `logs`.
pub fn log_outcome(logs: &Path, job: &str, o: &ScriptOutcome, at_ms: i64) {
    for e in outcome_entries(job, o, at_ms) {
        let _ = log_append(logs, &e, MAX_LOG_BYTES);
    }
}
