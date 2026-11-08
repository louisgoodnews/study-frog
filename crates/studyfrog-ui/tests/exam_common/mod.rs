#![allow(dead_code, unused_imports, clippy::unwrap_used, clippy::expect_used)] // shared test helpers
//! An exam on screen (Paris / 7x8) and the messages that play it.

use studyfrog_core::exam::{ExamGrade, ExamResult};
use studyfrog_core::review::Rating;
use studyfrog_core::run::RunMode;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::Action;
use studyfrog_service::compare::Scored;
use studyfrog_slm::score::{Score, Verdict};
use studyfrog_ui::effect::Effect;
use studyfrog_ui::exam_ui::{ExamEffect, ExamMsg};
use studyfrog_ui::model::{Model, apply};
use studyfrog_ui::msg::Msg;

use crate::common::views::running_model;

pub fn exam_model() -> Model {
    running_model(RunMode::Exam)
}

pub fn submit(m: &mut Model, text: &str) -> Vec<Effect> {
    apply(m, Msg::AnswerEdited(text.into()));
    apply(m, Msg::RunAction(Action::SubmitAnswer(text.into())))
}

pub fn scored(value: f32) -> Scored {
    Scored {
        score: Score {
            value,
            verdict: if value >= 0.8 {
                Verdict::Correct
            } else {
                Verdict::Incorrect
            },
            rubric_hits: Vec::new(),
        },
        rating: Rating::Good,
        engine: "words".into(),
    }
}

pub fn grade_effects(effects: &[Effect]) -> usize {
    effects
        .iter()
        .filter(|e| matches!(e, Effect::Exam(ExamEffect::Grade(Uid(100)))))
        .count()
}

pub fn result(percent: f32, grade: ExamGrade) -> ExamResult {
    ExamResult {
        run: Uid(100),
        items: 2,
        answered: 2,
        percent,
        grade,
        passed: grade != ExamGrade::F,
        engine: "words".into(),
        graded_at_ms: 1,
    }
}

/// Both items answered and scored, graded at 82.5 %.
pub fn graded_model() -> Model {
    let mut m = exam_model();
    submit(&mut m, "Paris");
    apply(&mut m, Msg::AnswerScored(Uid(1), Ok(scored(1.0))));
    submit(&mut m, "54");
    apply(&mut m, Msg::AnswerScored(Uid(2), Ok(scored(0.65))));
    let graded = Ok(result(82.5, ExamGrade::BPlus));
    apply(&mut m, Msg::Exam(ExamMsg::Graded(Uid(100), graded)));
    m
}
