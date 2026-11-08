//! What the CSV export adds to the reports (STORY-0098): every stack's numbers and the totals of
//! learned and mastered items and the active streak, measured as the achievements measure them.

use std::collections::BTreeSet;

use studyfrog_achieve::stats::streak_days;
use studyfrog_core::time::UnixMs;
use studyfrog_store::achievements::{load_mastered, load_review_rows};
use studyfrog_store::report_stacks::{StackReportRow, count_learned, load_stack_report};

use crate::session::{ServiceError, VaultSession};

const MS_PER_DAY: i64 = 86_400_000;

/// The vault's totals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReportTotals {
    /// Items with at least one review.
    pub learned: u32,
    /// Items scheduled far enough ahead to count as mastered.
    pub mastered: u32,
    /// Consecutive local days with a review, up to today (a streak that ended yesterday counts).
    pub streak_days: u32,
}

/// The per-stack numbers and the totals.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReportExtras {
    /// Every stack, by folder.
    pub stacks: Vec<StackReportRow>,
    /// The vault's totals.
    pub totals: ReportTotals,
}

/// Measure them at `now`; `offset_minutes` is the local UTC offset (for the days of the streak).
pub fn report_extras(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
) -> Result<ReportExtras, ServiceError> {
    let conn = &session.conn;
    let local_day = |ms: i64| (ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY);
    let days: BTreeSet<i64> = load_review_rows(conn)
        .map_err(ServiceError::Store)?
        .iter()
        .map(|r| local_day(r.at_ms))
        .collect();
    let totals = ReportTotals {
        learned: count_learned(conn).map_err(ServiceError::Store)?,
        mastered: u32::try_from(load_mastered(conn).map_err(ServiceError::Store)?.len())
            .unwrap_or(u32::MAX),
        streak_days: streak_days(&days, local_day(now.0)),
    };
    Ok(ReportExtras {
        stacks: load_stack_report(conn, now.0).map_err(ServiceError::Store)?,
        totals,
    })
}
