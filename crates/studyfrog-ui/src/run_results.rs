//! Results of work done for a run after the fact: answer comparisons and changed ratings. Pure.

use studyfrog_rehearsal::session::current_item;

use crate::effect::Effect;
use crate::state::{Notice, NoticeKind, State};

fn error(state: State, text: impl Into<String>) -> (State, Vec<Effect>) {
    let notice = Some(Notice {
        kind: NoticeKind::Error,
        text: text.into(),
    });
    (State { notice, ..state }, Vec::new())
}

fn with_run(
    state: State,
    f: impl FnOnce(crate::run_state::RunView) -> crate::run_state::RunView,
) -> (State, Vec<Effect>) {
    let mut state = state;
    state.rehearse.run = state.rehearse.run.take().map(f);
    (state, Vec::new())
}

/// Keep the score if it is for the item now on screen; a late answer for an item already rated is
/// dropped. A failure is reported but never blocks rating: the user can always rate by hand.
pub fn answer_scored(
    state: State,
    uid: studyfrog_core::uid::Uid,
    result: Result<crate::run_state::Scored, String>,
) -> (State, Vec<Effect>) {
    let mut state = state;
    let current = state
        .rehearse
        .run
        .as_ref()
        .and_then(|r| current_item(&r.session));
    if current != Some(uid) {
        return (state, Vec::new());
    }
    match result {
        Ok(scored) => {
            if let Some(run) = &mut state.rehearse.run {
                run.scores.insert(uid, scored.clone());
                run.scored = Some(scored);
            }
            (state, Vec::new())
        }
        Err(e) => error(state, format!("Could not compare your answer: {e}")),
    }
}

/// A rating was replaced (or not): remember it for the review, and check achievements again.
pub fn rerated(
    state: State,
    uid: studyfrog_core::uid::Uid,
    result: Result<(), String>,
) -> (State, Vec<Effect>) {
    match result {
        Ok(()) => {
            let (state, _) = with_run(state, |mut r| {
                r.applied.insert(uid);
                r
            });
            (state, vec![Effect::CheckAchievements])
        }
        Err(e) => error(state, format!("Could not change the rating: {e}")),
    }
}
