//! Finishing a mock exam (STORY-0187): a selected local model grades the answers in the worker; without one,
//! the keyword check grades them at once. A remote model is not sent the student's answers. Pure.

use studyfrog_agent::provider::is_local;

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
    match selected_provider(&state.agent).filter(is_local) {
        Some(provider) => {
            let effect = Effect::MockExam(MockExamEffect::Grade {
                exam: Box::new(g.exam.clone()),
                keys: g.keys.clone(),
                answers: ui.answers.clone(),
                provider,
            });
            let ui = MockExamUi {
                started: false,
                judging: true,
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
