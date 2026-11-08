//! The reports screen's update logic. Pure.

use studyfrog_service::reports::ReportBundle;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

/// Keep the reports; a failure is reported and the previous reports stay.
pub fn built(state: State, result: Result<ReportBundle, String>) -> (State, Vec<Effect>) {
    match result {
        Ok(reports) => (
            State {
                reports: Some(reports),
                ..state
            },
            Vec::new(),
        ),
        Err(e) => {
            let notice = Some(Notice {
                kind: NoticeKind::Error,
                text: format!("Could not compute the reports: {e}"),
            });
            (State { notice, ..state }, Vec::new())
        }
    }
}

/// Go to the rehearsal screen and start a practice run of `items`, unless a run is on.
pub fn drill(state: State, items: Vec<studyfrog_core::uid::Uid>) -> (State, Vec<Effect>) {
    if state.rehearse.run.is_some() || state.rehearse.starting {
        let notice = Some(Notice {
            kind: NoticeKind::Error,
            text: "Finish or stop the current run first.".to_owned(),
        });
        return (State { notice, ..state }, Vec::new());
    }
    let (mut state, mut effects) = crate::nav::navigate(state, crate::state::Screen::Rehearse);
    state.rehearse.starting = true;
    effects.push(Effect::Library(crate::library_bulk::LibraryEffect::Drill(
        items,
    )));
    (state, effects)
}
