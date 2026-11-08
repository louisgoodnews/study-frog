//! Exam mode in the window (`docs/specs/exam-mode.md`): the answers given, the scores as they arrive, grading
//! once every answer has its score, and the result. Pure: the worker grades (`exec_exam`).

use std::collections::HashSet;

use studyfrog_core::event::EventKind;
use studyfrog_core::exam::{ExamResult, grade_badge};
use studyfrog_core::run::{RunMode, RunState};
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::run_state::{RunView, Scored};
use crate::state::{Notice, NoticeKind, State};

/// What the window knows about the exam on screen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExamProgress {
    /// Answers whose comparison failed; they count 0.
    pub unscored: HashSet<Uid>,
    /// The worker is grading.
    pub grading: bool,
    /// The graded result.
    pub result: Option<ExamResult>,
    /// Why grading failed.
    pub error: Option<String>,
}

/// Exam messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ExamMsg {
    /// The worker graded the run (or could not).
    Graded(Uid, Result<ExamResult, String>),
}

/// Exam work for the worker.
#[derive(Debug, Clone, PartialEq)]
pub enum ExamEffect {
    /// Grade the finished exam run and store its result.
    Grade(Uid),
}

/// True when the run on screen is an exam.
pub fn is_exam(run: &RunView) -> bool {
    run.session.run.mode == RunMode::Exam
}

/// The locked answers, in the order they were given.
pub fn answers(run: &RunView) -> Vec<(Uid, &str)> {
    run.events
        .iter()
        .filter_map(|e| match (&e.kind, e.item) {
            (EventKind::AnswerSubmitted(text), Some(item)) => Some((item, text.as_str())),
            _ => None,
        })
        .collect()
}

/// The answer locked for `item`, if any.
pub fn answer_of(run: &RunView, item: Uid) -> Option<&str> {
    answers(run)
        .into_iter()
        .find(|(i, _)| *i == item)
        .map(|(_, a)| a)
}

/// Every answer has a score or failed to get one.
fn all_scored(run: &RunView) -> bool {
    answers(run)
        .iter()
        .all(|(i, _)| run.scores.contains_key(i) || run.exam.unscored.contains(i))
}

/// Ask for grading once a finished exam has every score; at most once.
pub fn grade_when_ready(run: &mut RunView) -> Option<Effect> {
    let ready = is_exam(run)
        && run.session.run.state == RunState::Finished
        && !run.exam.grading
        && run.exam.result.is_none()
        && all_scored(run);
    ready.then(|| {
        run.exam.grading = true;
        Effect::Exam(ExamEffect::Grade(run.session.run.uid))
    })
}

/// A comparison arrived for an exam answer (any item: the exam has moved on by then).
pub fn score_arrived(
    state: State,
    uid: Uid,
    result: Result<Scored, String>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    let mut effects = Vec::new();
    if let Err(e) = &result {
        state.notice = Some(Notice {
            kind: NoticeKind::Error,
            text: format!("Could not compare an answer; it counts 0: {e}"),
        });
    }
    if let Some(run) = &mut state.rehearse.run {
        match result {
            Ok(scored) => {
                run.scores.insert(uid, scored);
            }
            Err(_) => {
                run.exam.unscored.insert(uid);
            }
        }
        effects.extend(grade_when_ready(run));
    }
    (state, effects)
}

/// The result as one line, for the diary note.
pub fn result_line(r: &ExamResult) -> String {
    let passed = if r.passed { "passed" } else { "not passed" };
    format!(
        "Exam: {:.1} % ({}), {passed}",
        r.percent,
        grade_badge(r.grade)
    )
}

/// Handle exam messages.
pub fn update(state: State, msg: ExamMsg) -> (State, Vec<Effect>) {
    let mut state = state;
    let ExamMsg::Graded(uid, result) = msg;
    let Some(run) = state
        .rehearse
        .run
        .as_mut()
        .filter(|r| r.session.run.uid == uid)
    else {
        return (state, Vec::new());
    };
    run.exam.grading = false;
    match result {
        Ok(r) => {
            if let Some(p) = run.diary_prompt.as_mut().filter(|p| p.note.is_empty()) {
                p.note = result_line(&r);
            }
            run.exam.result = Some(r);
            (state, Vec::new())
        }
        Err(e) => {
            run.exam.error = Some(e.clone());
            state.notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not grade the exam: {e}"),
            });
            (state, Vec::new())
        }
    }
}

/// Load every exam item not loaded yet, so the review can show all references.
pub fn load_rest(run: &mut RunView) -> Vec<Effect> {
    let rest: Vec<Uid> = run.session.queue.clone();
    rest.into_iter()
        .filter(|uid| run.requested.insert(*uid))
        .map(Effect::LoadItem)
        .collect()
}
