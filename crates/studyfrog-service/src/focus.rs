//! The focus timer's record (STORY-0112, `docs/specs/focus-timer.md`): a finished work block is stored
//! with the calendar study session it overlapped most, if any.

use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{EventKind, Session};
use studyfrog_core::civil::{Date, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_store::focus::{FocusBlock, focus_minutes_between, insert_focus_block};

use crate::planning::list_sessions;
use crate::session::{ServiceError, VaultSession};

/// A work block that just ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusDone {
    /// When it started (Unix milliseconds).
    pub started_ms: i64,
    /// When it ended.
    pub ended_ms: i64,
    /// Minutes focused (pauses excluded).
    pub minutes: u32,
    /// The stack folder chosen for it, if any.
    pub stack: Option<String>,
}

/// What was recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusRecorded {
    /// Minutes of the block.
    pub minutes: u32,
    /// The study session it counts towards: id, title and local date.
    pub session: Option<(Uid, String, Date)>,
    /// Minutes of focus recorded today so far, this block included.
    pub today: u32,
}

/// The study session occurrence that overlaps the block most (local minutes of the block's end date).
pub fn active_session(
    sessions: &[Session],
    (start, end): (UnixMs, UnixMs),
    offset_minutes: i32,
) -> Option<(Uid, String, Date)> {
    let (a, b) = (
        utc_to_local(start, offset_minutes),
        utc_to_local(end, offset_minutes),
    );
    let from = if a.date == b.date {
        i32::from(a.minute)
    } else {
        0
    };
    let to = i32::from(b.minute).max(from + 1);
    let study: Vec<Session> = sessions
        .iter()
        .filter(|s| s.kind == EventKind::Study)
        .cloned()
        .collect();
    occurrences_all(&study, b.date, b.date)
        .into_iter()
        .map(|o| {
            let (s, e) = (
                i32::from(o.start_minute),
                i32::from(o.start_minute) + i32::from(o.duration_min),
            );
            (o, to.min(e) - from.max(s))
        })
        .filter(|(_, overlap)| *overlap > 0)
        .max_by_key(|(_, overlap)| *overlap)
        .and_then(|(o, _)| {
            let title = study.iter().find(|s| s.id == o.session)?.title.clone();
            Some((o.session, title, o.date))
        })
}

/// Store the block with the session it overlapped, at the UTC offset `offset_minutes`.
pub fn record_focus_at(
    session: &VaultSession,
    done: &FocusDone,
    offset_minutes: i32,
) -> Result<FocusRecorded, ServiceError> {
    let sessions = list_sessions(session)?;
    let span = (UnixMs(done.started_ms), UnixMs(done.ended_ms));
    let active = active_session(&sessions, span, offset_minutes);
    let block = FocusBlock {
        started_ms: done.started_ms,
        ended_ms: done.ended_ms.max(done.started_ms),
        minutes: done.minutes.min(1440),
        stack: done.stack.clone(),
        session: active.as_ref().map(|(u, _, d)| (*u, *d)),
    };
    insert_focus_block(&session.conn, &block).map_err(ServiceError::Store)?;
    let local_ms = i64::from(offset_minutes) * 60_000;
    let day = 86_400_000;
    let start = (done.ended_ms + local_ms).div_euclid(day) * day - local_ms;
    let today =
        focus_minutes_between(&session.conn, start, start + day).map_err(ServiceError::Store)?;
    Ok(FocusRecorded {
        minutes: block.minutes,
        session: active,
        today,
    })
}

/// [`record_focus_at`] at the local UTC offset of the block's end.
pub fn record_focus(
    session: &VaultSession,
    done: &FocusDone,
) -> Result<FocusRecorded, ServiceError> {
    let offset = crate::timezone::local_offset_minutes(UnixMs(done.ended_ms));
    record_focus_at(session, done, offset)
}
