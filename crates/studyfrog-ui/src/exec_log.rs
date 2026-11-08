//! Writing notices to the application log, from the worker.

use studyfrog_service::logging::{LogEntry, LogLevel, MAX_LOG_BYTES, log_append};

use crate::exec::Worker;
use crate::state::NoticeKind;

/// Add a notice to the application log. A worker without a log directory, or a log that cannot be
/// written, is ignored: logging must never get in the way of the work.
pub(crate) fn log_notice(w: &Worker, kind: NoticeKind, message: String, at_ms: i64) {
    let Some(dir) = &w.state_dir else { return };
    let level = match kind {
        NoticeKind::Info => LogLevel::Info,
        NoticeKind::Error => LogLevel::Error,
    };
    let entry = LogEntry {
        at_ms,
        level,
        source: "notice".to_owned(),
        message,
    };
    let _ = log_append(dir, &entry, MAX_LOG_BYTES);
}
