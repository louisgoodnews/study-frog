//! Saving a mock exam as a note (STORY-0186): the request, and the outcome shown to the user. Pure.

use super::{MockExamEffect, MockExamUi, map_ui, with_ui};
use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

pub(super) fn save(state: State) -> (State, Vec<Effect>) {
    let Some(ui) = state.rehearse.mock.clone() else {
        return (state, Vec::new());
    };
    let Some(g) = ui.result.as_ref() else {
        return (state, Vec::new());
    };
    let effect = Effect::MockExam(MockExamEffect::Save {
        folder: ui.folder.clone(),
        title: ui.title.clone(),
        exam: Box::new(g.exam.clone()),
        keys: g.keys.clone(),
    });
    (state, vec![effect])
}

pub(super) fn saved(state: State, result: Result<String, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(path) => {
            let ui = map_ui(&state, |ui| MockExamUi {
                saved: Some(path.clone()),
                ..ui
            });
            let state = with_ui(state, ui);
            let notice = Notice {
                kind: NoticeKind::Info,
                text: format!("The exam was saved as a note: {path}"),
            };
            (
                State {
                    notice: Some(notice),
                    ..state
                },
                Vec::new(),
            )
        }
        Err(e) => {
            let notice = Notice {
                kind: NoticeKind::Error,
                text: format!("The exam could not be saved: {e}"),
            };
            (
                State {
                    notice: Some(notice),
                    ..state
                },
                Vec::new(),
            )
        }
    }
}
