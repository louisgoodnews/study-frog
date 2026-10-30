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
