//! Which jobs run now: scheduled jobs once per local minute their schedule matches, and the jobs
//! bound to a lifecycle event. Pure: the time and the UTC offset are passed in.

use studyfrog_core::civil::utc_to_local;
use studyfrog_core::time::UnixMs;

use super::config::{ScriptJob, ScriptsConfig};
use super::cron::cron_matches;

/// Minutes looked back after a gap (a suspended machine): at most one hour is caught up.
pub const MAX_CATCH_UP_MINUTES: i64 = 60;

/// The scheduler's memory: the last minute (Unix minutes) whose jobs were started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Ticker {
    /// Last minute checked; `None` before the first tick.
    pub last_minute: Option<i64>,
}

/// The minutes to check at `now_ms`: every minute after the last one checked, up to now (at most
/// [`MAX_CATCH_UP_MINUTES`]); just the current minute on the first tick; none when the clock went back.
pub fn minutes_to_check(t: &Ticker, now_ms: i64) -> std::ops::RangeInclusive<i64> {
    let now = now_ms.div_euclid(60_000);
    let first = t
        .last_minute
        .map_or(now, |last| (last + 1).max(now - MAX_CATCH_UP_MINUTES + 1));
    first..=now
}

/// The names of the enabled jobs whose schedule matches one of the minutes since the last tick, each
/// once, in file order; the ticker moves on to `now_ms`. `offset_minutes` is the local UTC offset.
pub fn tick(t: &mut Ticker, cfg: &ScriptsConfig, now_ms: i64, offset_minutes: i32) -> Vec<String> {
    let minutes = minutes_to_check(t, now_ms);
    let end = *minutes.end();
    let locals: Vec<_> = minutes
        .map(|m| utc_to_local(UnixMs(m * 60_000), offset_minutes))
        .collect();
    if t.last_minute.is_some_and(|last| last >= end) {
        return Vec::new();
    }
    t.last_minute = Some(end);
    cfg.jobs
        .iter()
        .filter(|j| j.enabled)
        .filter(|j| {
            j.schedule
                .as_ref()
                .is_some_and(|(c, _)| locals.iter().any(|l| cron_matches(c, *l)))
        })
        .map(|j| j.name.clone())
        .collect()
}

/// The enabled jobs bound to the event `kind`, in file order.
pub fn jobs_for_event<'a>(cfg: &'a ScriptsConfig, kind: &str) -> Vec<&'a ScriptJob> {
    cfg.jobs
        .iter()
        .filter(|j| j.enabled && j.on.iter().any(|e| e == kind))
        .collect()
}

/// The job with this name, or the job whose script is this path.
pub fn find_job<'a>(cfg: &'a ScriptsConfig, name_or_script: &str) -> Option<&'a ScriptJob> {
    cfg.jobs
        .iter()
        .find(|j| j.name == name_or_script)
        .or_else(|| cfg.jobs.iter().find(|j| j.script == name_or_script))
}
