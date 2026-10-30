//! Effects of a rehearsal run: create it, load item content, save steps.

use studyfrog_core::plan::RunPlan;
use studyfrog_core::run::RunMode;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::input::StepOutput;
use studyfrog_service::content::load_item_content;
use studyfrog_service::create::fresh_id_source;
use studyfrog_service::rehearse::{StepContext, record_step, start_run};
use studyfrog_service::session::current_settings;
use studyfrog_service::timezone::local_day_window;

use crate::exec::Worker;
use crate::msg::Msg;

fn millis(now_ns: i64) -> i64 {
    now_ns / 1_000_000
}

/// Create and store a run.
pub fn start(w: &Worker, plan: RunPlan, mode: RunMode, now_ns: i64) -> Msg {
    let Some(session) = &w.session else {
        return Msg::RunReady(Err("no vault is open".to_owned()));
    };
    let now_ms = millis(now_ns);
    let result = fresh_id_source(u64::try_from(now_ms).unwrap_or(0)).and_then(|ids| {
        start_run(
            session,
            plan,
            mode,
            UnixMs(now_ms),
            local_day_window(UnixMs(now_ms)),
            ids,
        )
    });
    Msg::RunReady(result.map_err(|e| e.to_string()))
}

/// Read an item's content.
pub fn load_item(w: &Worker, uid: Uid) -> Msg {
    match &w.session {
        Some(session) => Msg::ItemLoaded(
            uid,
            load_item_content(session, uid).map_err(|e| e.to_string()),
        ),
        None => Msg::ItemLoaded(uid, Err("no vault is open".to_owned())),
    }
}

/// Save one engine step.
pub fn record(w: &mut Worker, ctx: &StepContext, out: &StepOutput, now_ns: i64) -> Msg {
    let Some(session) = &mut w.session else {
        return Msg::StepRecorded(Err("no vault is open".to_owned()));
    };
    let vault = current_settings(session);
    let result = record_step(&mut session.conn, ctx, out, UnixMs(millis(now_ns)), &vault);
    Msg::StepRecorded(result.map_err(|e| e.to_string()))
}
