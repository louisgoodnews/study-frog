//! Performing the homework screen's effects (STORY-0167). Thin: the rules live in the service.

use studyfrog_calendar::homework::HomeworkItem;
use studyfrog_core::civil::{LocalTime, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::homework::{
    advance_homework_status, new_homework_id, save_homework_checked, toggle_homework_step,
};
use studyfrog_service::session::{ServiceError, VaultSession};
use studyfrog_service::timezone::local_offset_minutes;

use crate::exec::Worker;
use crate::homework_msg::HomeworkMsg;
use crate::homework_msg::{HomeworkEffect, HomeworkResult};
use crate::msg::Msg;

const NO_VAULT: &str = "no vault is open";

fn clock(now_ns: i64) -> LocalTime {
    let now = UnixMs(now_ns / 1_000_000);
    utc_to_local(now, local_offset_minutes(now))
}

/// Run one change on the open vault and read the items back.
fn run(
    w: &mut Worker,
    clock: LocalTime,
    change: impl FnOnce(&mut VaultSession) -> Result<Vec<HomeworkItem>, ServiceError>,
) -> HomeworkResult {
    match &mut w.session {
        Some(s) => change(s)
            .map(|items| (items, clock))
            .map_err(|e| e.to_string()),
        None => Err(NO_VAULT.to_owned()),
    }
}

fn save(
    w: &mut Worker,
    existing: Option<studyfrog_core::uid::Uid>,
    mut item: HomeworkItem,
    now_ns: i64,
) -> HomeworkResult {
    let clock = clock(now_ns);
    let id = match existing {
        Some(id) => id,
        None => {
            let ids = fresh_id_source(u64::try_from(now_ns / 1_000_000).unwrap_or(0))
                .map_err(|e| e.to_string())?;
            new_homework_id(ids)
        }
    };
    item.id = id;
    run(w, clock, |s| save_homework_checked(s, item))
}

/// Perform one homework effect and report the result as a message.
pub fn execute(w: &mut Worker, effect: HomeworkEffect, now_ns: i64) -> Msg {
    let clock = clock(now_ns);
    match effect {
        HomeworkEffect::Save { existing, item } => {
            Msg::Homework(HomeworkMsg::Saved(save(w, existing, item, now_ns)))
        }
        HomeworkEffect::Advance(id) => Msg::Homework(HomeworkMsg::Changed(run(w, clock, |s| {
            advance_homework_status(s, id)
        }))),
        HomeworkEffect::ToggleStep(id, index) => {
            Msg::Homework(HomeworkMsg::Changed(run(w, clock, |s| {
                toggle_homework_step(s, id, index)
            })))
        }
    }
}
