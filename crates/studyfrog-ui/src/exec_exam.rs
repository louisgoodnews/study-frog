//! The worker's part of exam mode: grade a finished exam from what is stored.

use studyfrog_core::time::UnixMs;
use studyfrog_service::exam::grade_exam_run;

use crate::exam_ui::{ExamEffect, ExamMsg};
use crate::exec::Worker;
use crate::msg::Msg;

/// Perform an exam effect.
pub fn execute(w: &Worker, e: ExamEffect, now_ns: i64) -> Msg {
    let ExamEffect::Grade(run) = e;
    let result = match &w.session {
        Some(session) => grade_exam_run(&session.conn, run, UnixMs(now_ns / 1_000_000))
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Exam(ExamMsg::Graded(run, result))
}
