//! Exam runs over the control protocol (`docs/specs/exam-mode.md`, section 8): what an exam refuses, answered
//! with an error instead of being ignored, and the exam part of the `state` snapshot. Pure.

use serde_json::{Value, json};
use studyfrog_core::exam::{grade_german, grade_letter};
use studyfrog_core::run::RunState;

use crate::control_cmd::RehearseCmd;
use crate::control_state::ulid;
use crate::exam_ui::{answers, is_exam};
use crate::run_state::RunView;
use crate::run_update::is_over;
use crate::state::State;

/// Why `cmd` is refused in the exam on screen, if it is.
pub fn check(state: &State, cmd: &RehearseCmd) -> Result<(), String> {
    let Some(run) = state.rehearse.run.as_ref().filter(|r| is_exam(r)) else {
        return match cmd {
            RehearseCmd::Finish => Err("there is no exam to finish".to_owned()),
            _ => Ok(()),
        };
    };
    let blank = |t: &str| t.trim().is_empty();
    match cmd {
        RehearseCmd::Flip | RehearseCmd::Skip | RehearseCmd::Rate(_) | RehearseCmd::Mark(_) => {
            Err("an exam takes only written answers: no flip, skip, rating or mark".to_owned())
        }
        RehearseCmd::Answer(t) if blank(t) => {
            Err("write an answer before submitting it".to_owned())
        }
        RehearseCmd::Submit if blank(&run.answer) => {
            Err("the answer is empty: rehearse type <text> first".to_owned())
        }
        RehearseCmd::Finish if !is_over(run) => Err(
            "the exam is not over: answer every item (or rehearse stop, which is not graded)"
                .to_owned(),
        ),
        _ => Ok(()),
    }
}

/// The exam part of a run in the snapshot; `null` for other runs.
pub fn exam_json(r: &RunView) -> Value {
    if !is_exam(r) {
        return Value::Null;
    }
    let locked: Vec<Value> = answers(r)
        .into_iter()
        .map(|(item, text)| {
            let score = r.scores.get(&item).map(|s| s.score.value);
            json!({"item": ulid(item), "answer": text, "score": score})
        })
        .collect();
    let result = r.exam.result.as_ref().map(|x| {
        json!({"percent": (f64::from(x.percent) * 10.0).round() / 10.0, "grade": grade_letter(x.grade), "german": grade_german(x.grade),
               "passed": x.passed, "items": x.items, "answered": x.answered, "engine": x.engine})
    });
    json!({
        "items": r.session.queue.len(), "answered": locked.len(), "locked": locked,
        "graded": result.is_some(), "grading": r.exam.grading, "result": result,
        "error": r.exam.error, "stopped": r.session.run.state == RunState::Aborted,
    })
}
