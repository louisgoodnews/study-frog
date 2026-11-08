//! Grading an exam run from what is stored: its queue, its answers and their latest scores
//! (`docs/specs/exam-mode.md`, sections 4 to 6).

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use studyfrog_core::event::EventKind;
use studyfrog_core::exam::{ExamResult, grade_exam};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::answer_scores::load_answer_scores;
use studyfrog_store::exam_results::save_exam_result;
use studyfrog_store::run_load::{load_events, load_run};

use crate::session::ServiceError;

/// Every item of a queue once, in queue order.
pub fn exam_items(queue: &[Uid]) -> Vec<Uid> {
    let mut out: Vec<Uid> = Vec::with_capacity(queue.len());
    for uid in queue {
        if !out.contains(uid) {
            out.push(*uid);
        }
    }
    out
}

/// Compute the result of a finished exam run without storing it. The latest score of an item counts;
/// unanswered and unscored items count 0.
pub fn exam_result_of(
    conn: &Connection,
    run: Uid,
    now: UnixMs,
) -> Result<ExamResult, ServiceError> {
    let stored = load_run(conn, run)
        .map_err(ServiceError::Store)?
        .ok_or_else(|| ServiceError::Invalid("no such run".to_owned()))?;
    if stored.mode != RunMode::Exam {
        return Err(ServiceError::Invalid("this run is not an exam".to_owned()));
    }
    if stored.state != RunState::Finished {
        return Err(ServiceError::Invalid(
            "only a finished exam is graded (a stopped exam is not)".to_owned(),
        ));
    }
    let rows = load_answer_scores(conn, run).map_err(ServiceError::Store)?;
    let engine = rows.last().map(|r| r.engine.clone()).unwrap_or_default();
    let latest: HashMap<Uid, f32> = rows.into_iter().map(|r| (r.item, r.value)).collect();
    let items = exam_items(&stored.queue);
    let scores: Vec<Option<f32>> = items.iter().map(|i| latest.get(i).copied()).collect();
    let answered = load_events(conn, run)
        .map_err(ServiceError::Store)?
        .iter()
        .filter(|e| matches!(e.kind, EventKind::AnswerSubmitted(_)))
        .filter_map(|e| e.item)
        .filter(|i| items.contains(i))
        .collect::<HashSet<Uid>>()
        .len();
    Ok(ExamResult {
        engine,
        ..grade_exam(run, &scores, answered as u32, now.0)
    })
}

/// Grade a finished exam run and store its result.
pub fn grade_exam_run(
    conn: &Connection,
    run: Uid,
    now: UnixMs,
) -> Result<ExamResult, ServiceError> {
    let result = exam_result_of(conn, run, now)?;
    save_exam_result(conn, &result).map_err(ServiceError::Store)?;
    Ok(result)
}
