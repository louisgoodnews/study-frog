//! The homework screen's transitions (STORY-0167). Pure: effects are returned, not run.

use studyfrog_calendar::homework::HomeworkItem;
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;

use crate::effect::Effect;
use crate::homework_form::{apply, form_of, new_form, to_item};
use crate::homework_msg::HomeworkMsg;
use crate::homework_msg::{HomeworkEffect, HomeworkResult, HomeworkUi};
use crate::planner::Planner;
use crate::state::State;

/// Placeholder id for a new item; the worker assigns the real one.
const NEW_ID: Uid = Uid(0);

fn with_ui(state: State, homework_ui: HomeworkUi) -> State {
    let planner = Planner {
        homework_ui,
        ..state.planner.clone()
    };
    State { planner, ..state }
}

fn with_items(state: State, items: Vec<HomeworkItem>, clock: LocalTime) -> State {
    let planner = Planner {
        homework: items,
        homework_now: Some(clock),
        ..state.planner.clone()
    };
    State { planner, ..state }
}

/// The day a new item starts from: today on the last clock read.
fn today_of(state: &State) -> Date {
    state
        .planner
        .homework_now
        .map_or(state.planner.today, |now| now.date)
}

fn effect(e: HomeworkEffect) -> Vec<Effect> {
    vec![Effect::Homework(e)]
}

/// Apply one homework screen message.
pub fn update(state: State, msg: HomeworkMsg) -> (State, Vec<Effect>) {
    let ui = state.planner.homework_ui.clone();
    match msg {
        HomeworkMsg::Tab(tab) => (with_ui(state, HomeworkUi { tab, ..ui }), Vec::new()),
        HomeworkMsg::New => {
            let form = new_form(today_of(&state));
            let ui = HomeworkUi {
                form: Some(form),
                error: None,
                ..ui
            };
            (with_ui(state, ui), Vec::new())
        }
        HomeworkMsg::EditItem(id) => {
            let form = state
                .planner
                .homework
                .iter()
                .find(|i| i.id == id)
                .map(form_of);
            match form {
                Some(form) => {
                    let ui = HomeworkUi {
                        form: Some(form),
                        error: None,
                        confirm_delete: false,
                        ..ui
                    };
                    (with_ui(state, ui), Vec::new())
                }
                None => (state, Vec::new()),
            }
        }
        HomeworkMsg::Close => (
            with_ui(
                state,
                HomeworkUi {
                    form: None,
                    error: None,
                    confirm_delete: false,
                    ..ui
                },
            ),
            Vec::new(),
        ),
        HomeworkMsg::Edit(edit) => {
            let mut ui = ui;
            if let Some(form) = ui.form.as_mut() {
                apply(form, edit);
            }
            (with_ui(state, ui), Vec::new())
        }
        HomeworkMsg::Save => save(state, ui),
        HomeworkMsg::Advance(id) => (state, effect(HomeworkEffect::Advance(id))),
        HomeworkMsg::ToggleStep(id, index) => {
            (state, effect(HomeworkEffect::ToggleStep(id, index)))
        }
        HomeworkMsg::AskDelete => (
            with_ui(
                state,
                HomeworkUi {
                    confirm_delete: true,
                    ..ui
                },
            ),
            Vec::new(),
        ),
        HomeworkMsg::KeepItem => (
            with_ui(
                state,
                HomeworkUi {
                    confirm_delete: false,
                    ..ui
                },
            ),
            Vec::new(),
        ),
        HomeworkMsg::Delete(id) => (state, effect(HomeworkEffect::Delete(id))),
        HomeworkMsg::Deleted(Ok((items, clock))) => {
            let ui = HomeworkUi {
                form: None,
                error: None,
                confirm_delete: false,
                ..ui
            };
            (with_ui(with_items(state, items, clock), ui), Vec::new())
        }
        HomeworkMsg::Deleted(Err(e)) => (
            with_ui(
                state,
                HomeworkUi {
                    error: Some(e),
                    confirm_delete: false,
                    ..ui
                },
            ),
            Vec::new(),
        ),
        HomeworkMsg::Saved(Ok((items, clock))) => {
            let ui = HomeworkUi {
                form: None,
                error: None,
                confirm_delete: false,
                ..ui
            };
            (with_ui(with_items(state, items, clock), ui), Vec::new())
        }
        HomeworkMsg::Saved(Err(e)) => (
            with_ui(
                state,
                HomeworkUi {
                    error: Some(e),
                    ..ui
                },
            ),
            Vec::new(),
        ),
        HomeworkMsg::Changed(result) => settle(state, ui, result),
    }
}

fn save(state: State, ui: HomeworkUi) -> (State, Vec<Effect>) {
    let Some(form) = ui.form.as_ref() else {
        return (state, Vec::new());
    };
    match to_item(form, NEW_ID) {
        Ok(item) => {
            let existing = form.existing;
            (
                with_ui(state, ui.clone()),
                effect(HomeworkEffect::Save { existing, item }),
            )
        }
        Err(e) => (
            with_ui(
                state,
                HomeworkUi {
                    error: Some(e),
                    ..ui
                },
            ),
            Vec::new(),
        ),
    }
}

fn settle(state: State, ui: HomeworkUi, result: HomeworkResult) -> (State, Vec<Effect>) {
    match result {
        Ok((items, clock)) => (
            with_ui(
                with_items(state, items, clock),
                HomeworkUi { error: None, ..ui },
            ),
            Vec::new(),
        ),
        Err(e) => (
            with_ui(
                state,
                HomeworkUi {
                    error: Some(e),
                    ..ui
                },
            ),
            Vec::new(),
        ),
    }
}
