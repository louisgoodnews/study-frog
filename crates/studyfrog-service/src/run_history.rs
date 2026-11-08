//! The history of runs for the reports: each with its mode, its pacing, the time it used and how many answers a minute it made.

use studyfrog_core::pacing::Pacing;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::pacing::run_active_ms;
use studyfrog_store::error::StoreError;
use studyfrog_store::exam_results::load_exam_result;
use studyfrog_store::home::recent_runs;
use studyfrog_store::run_load::{load_events, load_run};

/// Most runs the history lists.
pub const HISTORY: usize = 100;

/// One run of the history.
#[derive(Debug, Clone, PartialEq)]
pub struct RunSummary {
    /// The run.
    pub uid: Uid,
    /// Flip cards or typed answers.
    pub mode: RunMode,
    /// The pacing it had.
    pub pacing: Pacing,
    /// How it ended: `finished`, `aborted`, ...
    pub state: String,
    /// When it started, in Unix milliseconds.
    pub started_ms: i64,
    /// Items rated.
    pub rated: u32,
    /// Items skipped.
    pub skipped: u32,
    /// The active time it used, in milliseconds.
    pub used_ms: u32,
    /// For a graded exam: its result.
    pub exam: Option<studyfrog_core::exam::ExamResult>,
}

/// Answers rated per minute of active time; `None` when no time was used.
pub fn answers_per_minute(rated: u32, used_ms: u32) -> Option<f32> {
    (used_ms > 0).then(|| rated as f32 * 60_000.0 / used_ms as f32)
}

/// The latest runs, newest first.
pub fn run_history(conn: &rusqlite::Connection) -> Result<Vec<RunSummary>, StoreError> {
    let mut out = Vec::new();
    for r in recent_runs(conn, HISTORY)? {
        let Some(run) = load_run(conn, r.uid)? else {
            continue;
        };
        out.push(RunSummary {
            uid: r.uid,
            mode: run.mode,
            pacing: run.plan.pacing,
            state: r.state,
            started_ms: r.started_ms,
            rated: r.rated,
            skipped: r.skipped,
            used_ms: run_active_ms(&load_events(conn, r.uid)?),
            exam: match run.mode {
                RunMode::Exam => load_exam_result(conn, r.uid)?,
                _ => None,
            },
        });
    }
    Ok(out)
}
