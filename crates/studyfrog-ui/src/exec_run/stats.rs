//! Reads for the views: achievements, reports, the home screen and the performance explorer.

use studyfrog_core::time::UnixMs;
use studyfrog_service::achievements::check_achievements;
use studyfrog_service::home::home_summary;
use studyfrog_service::reports::build_reports_local;
use studyfrog_service::timezone::{local_day_window, local_offset_minutes};

use crate::exec::Worker;
use crate::home_ui::HomeMsg;
use crate::msg::{Msg, MsgReports};

use super::millis;

/// Measure the achievements at the current local time.
pub fn achievements(w: &mut Worker, now_ns: i64) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = match &mut w.session {
        Some(session) => {
            check_achievements(session, now, local_offset_minutes(now)).map_err(|e| e.to_string())
        }
        None => Err("no vault is open".to_owned()),
    };
    Msg::Reports(MsgReports::AchievementsChecked(result))
}

/// Compute the reports at the current local time.
pub fn reports(w: &Worker, now_ns: i64) -> Msg {
    let _timer = studyfrog_debug::timed("reports", "build");
    let now = UnixMs(millis(now_ns));
    let result = match &w.session {
        Some(session) => build_reports_local(session, now)
            .map(Box::new)
            .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    match &result {
        Ok(_) => studyfrog_log::log_debug!("reports", "reports built"),
        Err(e) => studyfrog_log::log_warn!("reports", "reports not built: {}", e),
    }
    Msg::Reports(MsgReports::ReportsBuilt(result))
}

/// Read the home screen's numbers at the current local time.
pub fn home(w: &Worker, now_ns: i64) -> Msg {
    let now = UnixMs(millis(now_ns));
    let result = match &w.session {
        Some(session) => home_summary(
            session,
            now,
            local_offset_minutes(now),
            local_day_window(now),
        )
        .map(Box::new)
        .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Home(HomeMsg::Loaded(result))
}

/// Compute the performance explorer for a scope and period at the current local time.
pub fn explorer(
    w: &Worker,
    (scope, period): (
        studyfrog_service::explore::Scope,
        studyfrog_service::explore::Period,
    ),
    now_ns: i64,
) -> Msg {
    let now = UnixMs(now_ns / 1_000_000);
    let result = match &w.session {
        Some(session) => studyfrog_service::explore::explore(
            session,
            (scope, period),
            now,
            local_offset_minutes(now),
        )
        .map(Box::new)
        .map_err(|e| e.to_string()),
        None => Err("no vault is open".to_owned()),
    };
    Msg::Explorer(crate::explorer_ui::ExplorerMsg::Loaded(result))
}
