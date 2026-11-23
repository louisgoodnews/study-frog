//! Finishing a mock exam (STORY-0187, STORY-0207): objective-only exams grade immediately; open-ended
//! items use a selected local model in the worker or keyword fallback synchronously. Pure.

use studyfrog_agent::provider::is_local;
use studyfrog_rehearsal::exam_answer::is_objective_format;
use studyfrog_rehearsal::mock_exam::all_questions;

use super::grading;
use crate::agent_conn::selected_provider;
use crate::effect::Effect;
use crate::setup::mock_exam::{MockExamEffect, MockExamUi};
use crate::state::State;

/// Grade the answers written so far.
pub fn finish(state: State) -> (State, Vec<Effect>) {
    let Some(ui) = state.rehearse.mock.clone() else {
        return (state, Vec::new());
    };
    let Some(g) = ui.result.clone() else {
        return (state, Vec::new());
    };
    let needs_model = all_questions(&g.exam)
        .iter()
        .any(|q| !is_objective_format(q.format));
    match selected_provider(&state.agent)
        .filter(is_local)
        .filter(|_| needs_model)
    {
        Some(provider) => {
            let effect = Effect::MockExam(MockExamEffect::Grade {
                exam: Box::new(g.exam.clone()),
                keys: g.keys.clone(),
                answers: ui.answers.clone(),
                provider,
            });
            let ui = MockExamUi {
                judging: true,
                locked: true,
                graded: None,
                ..ui
            };
            (super::with_ui(state, Some(ui)), vec![effect])
        }
        None => {
            let graded = grading::grade(&g.exam, &g.keys, &ui.answers);
            let ui = MockExamUi {
                started: false,
                graded: Some(graded),
                ..ui
            };
            (super::with_ui(state, Some(ui)), Vec::new())
        }
    }
}
