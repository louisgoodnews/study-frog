//! Performing the calendar and board effects. Thin: the rules live in the service and domain crates.

use studyfrog_board::ops::BoardOp;
use studyfrog_calendar::session::Session;
use studyfrog_core::civil::{Date, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_service::create::{IdSource, fresh_id_source};
use studyfrog_service::kanban::{apply_board_op, board_or_default};
use studyfrog_service::planning::{
    cancel_occurrence, delete_session_checked, list_sessions, new_session_id, save_session_checked,
};
use studyfrog_service::reminders::take_due_reminders;
use studyfrog_service::session::{ServiceError, VaultSession};
use studyfrog_service::timezone::local_offset_minutes;

use crate::exec::Worker;
use crate::kanban::BoardMsg;
use crate::msg::Msg;
use crate::planner::PlanMsg;

const NO_VAULT: &str = "no vault is open";

fn ids(now_ns: i64) -> Result<IdSource, ServiceError> {
    fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
}

/// Today's date on the user's clock.
/// Reminders that came due; each is marked shown, so it is returned once.
pub fn reminders(w: &Worker, now_ns: i64) -> Msg {
    let now = UnixMs(now_ns / 1_000_000);
    let result = match &w.session {
        Some(session) => {
            take_due_reminders(session, now, local_offset_minutes(now)).map_err(|e| e.to_string())
        }
        None => Ok(Vec::new()),
    };
    Msg::RemindersDue(result)
}

fn today(now_ns: i64) -> Date {
    let now = UnixMs(now_ns / 1_000_000);
    utc_to_local(now, local_offset_minutes(now)).date
}

fn run_sessions(
    w: &mut Worker,
    f: impl FnOnce(&mut VaultSession) -> Result<Vec<Session>, ServiceError>,
) -> Msg {
    let result = match &mut w.session {
        Some(s) => f(s).map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    Msg::Plan(PlanMsg::Changed(result))
}

/// Read the sessions and today's date.
pub fn load_sessions(w: &mut Worker, now_ns: i64) -> Msg {
    let result = match &w.session {
        Some(s) => list_sessions(s)
            .map(|all| (all, today(now_ns)))
            .map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    Msg::Plan(PlanMsg::Loaded(result))
}

/// Store a session, giving a new one its id first.
pub fn save(w: &mut Worker, s: Session, now_ns: i64) -> Msg {
    run_sessions(w, |session| {
        let id = if s.id == Uid(0) {
            new_session_id(ids(now_ns)?)
        } else {
            s.id
        };
        save_session_checked(session, Session { id, ..s })
    })
}

/// Delete a session.
pub fn delete(w: &mut Worker, id: Uid) -> Msg {
    run_sessions(w, |session| delete_session_checked(session, id))
}

/// Skip one occurrence.
pub fn skip(w: &mut Worker, id: Uid, date: Date) -> Msg {
    run_sessions(w, |session| cancel_occurrence(session, id, date))
}

fn board_msg(
    w: &mut Worker,
    f: impl FnOnce(&mut VaultSession) -> Result<studyfrog_board::model::Board, ServiceError>,
) -> Msg {
    let result = match &mut w.session {
        Some(s) => f(s).map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    Msg::Board(BoardMsg::Loaded(result))
}

/// Read the board, creating the default one on first use.
pub fn load_board(w: &mut Worker, now_ns: i64) -> Msg {
    board_msg(w, |s| board_or_default(s, ids(now_ns)?))
}

/// Apply a board command.
pub fn board_command(w: &mut Worker, op: &BoardOp, now_ns: i64) -> Msg {
    board_msg(w, |s| apply_board_op(s, op, ids(now_ns)?))
}

fn reviews(m: crate::planner_reviews::ReviewMsg) -> Msg {
    Msg::Plan(PlanMsg::Reviews(m))
}

/// Read the reviews the schedule planned between two local days.
pub fn review_plan(w: &Worker, days: (i64, i64)) -> Msg {
    use crate::planner_reviews::ReviewMsg;
    let result = match &w.session {
        Some(s) => studyfrog_service::review_plan::plan_reviews(s, days).map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    reviews(ReviewMsg::Loaded(result))
}

/// Make the items due on a local day.
pub fn move_reviews(
    w: &Worker,
    (stack, items): (Option<Uid>, &[Uid]),
    day: i64,
    now_ns: i64,
) -> Msg {
    use crate::planner_reviews::ReviewMsg;
    let now = UnixMs(now_ns / 1_000_000);
    let result = match &w.session {
        Some(s) => studyfrog_service::review_plan::move_reviews(s, (stack, items), day, now)
            .map(Box::new)
            .map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    reviews(ReviewMsg::Moved(result))
}

/// Put the due dates of a move back.
pub fn undo_move(w: &Worker, undo: &[studyfrog_service::review_plan::Prior]) -> Msg {
    use crate::planner_reviews::ReviewMsg;
    let result = match &w.session {
        Some(s) => studyfrog_service::review_plan::undo_move(s, undo).map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    };
    reviews(ReviewMsg::Undone(result))
}

/// Perform one effect on the planned reviews.
pub fn reviews_effect(
    w: &Worker,
    effect: crate::planner_reviews::ReviewEffect,
    now_ns: i64,
) -> Msg {
    use crate::planner_reviews::ReviewEffect;
    match effect {
        ReviewEffect::Load(from, to) => review_plan(w, (from, to)),
        ReviewEffect::Move(stack, items, day) => move_reviews(w, (stack, &items), day, now_ns),
        ReviewEffect::Undo(undo) => undo_move(w, &undo),
    }
}
