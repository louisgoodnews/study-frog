//! Writing notices and worker events to the application log, and showing the logs folder.

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

/// Write the log lines `worker_log` chose for one effect (source `worker`).
pub(crate) fn log_worker(
    w: &Worker,
    label: &str,
    msgs: &[crate::msg::Msg],
    took_ms: u128,
    at_ms: i64,
) {
    let Some(dir) = &w.state_dir else { return };
    for (level, message) in crate::worker_log::worker_events(label, msgs, took_ms) {
        let entry = LogEntry {
            at_ms,
            level,
            source: "worker".to_owned(),
            message,
        };
        let _ = log_append(dir, &entry, MAX_LOG_BYTES);
    }
}

/// Show the logs folder in the system's file manager through the host's opener.
pub(crate) fn open_logs(w: &Worker) -> Vec<crate::msg::Msg> {
    let result = match (&w.state_dir, w.opener) {
        (None, _) => Err("no logs folder is known".to_owned()),
        (Some(_), None) => Err("this build cannot start a file manager".to_owned()),
        (Some(dir), Some(open)) => std::fs::create_dir_all(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))
            .and_then(|()| open(dir))
            .map(|()| dir.display().to_string()),
    };
    vec![crate::msg::Msg::Logs(crate::logs_ui::LogsMsg::Opened(
        result,
    ))]
}

/// Execute an effect and log what `worker_log` picks from its outcome (headless instances; the
/// window's worker thread does the same in `runtime`).
pub fn execute_logged(
    w: &mut Worker,
    effect: crate::effect::Effect,
    now_ns: i64,
) -> Vec<crate::msg::Msg> {
    let (label, start) = (
        crate::worker_log::effect_label(&effect),
        std::time::Instant::now(),
    );
    let msgs = crate::exec::execute(w, effect, now_ns);
    log_worker(
        w,
        &label,
        &msgs,
        start.elapsed().as_millis(),
        now_ns / 1_000_000,
    );
    msgs
}
