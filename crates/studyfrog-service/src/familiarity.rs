//! Familiarity in scheduling: after a rating, the next due date is put where the item's familiarity says
//! (`docs/research/familiarity-and-fsrs.md`). The stored memory state is never touched.

use rusqlite::Connection;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::{Date, date_from_days, date_to_days, days_between};
use studyfrog_core::event::{Difficulty, difficulty_from_name};
use studyfrog_core::review::Rating;
use studyfrog_core::settings::VaultSettings;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_report::familiarity::{FamContext, FamEvent, fam_params_default, familiarity};
use studyfrog_srs::familiarity::{
    FactorParams, before_deadline, factor_params_default, familiarity_factor, interval_days_with,
    scaled_interval_days,
};
use studyfrog_srs::fsrs::interval_days;
use studyfrog_srs::params::Params;

use crate::rehearse::RatingTiming;
use studyfrog_store::error::{StoreError, sql_err};
use studyfrog_store::schedule::{StoredMem, StoredSm2, load_reviews_timed_in};
use studyfrog_store::sessions::load_sessions;
use studyfrog_store::settings::ItemHome;
use studyfrog_store::views::load_views;

const DAY_MS: i64 = 86_400_000;

fn due_in(now: UnixMs, days: u32) -> UnixMs {
    UnixMs(now.0 + i64::from(days) * DAY_MS)
}

/// Whether familiarity tempers this item's intervals: not switched off for its stack, and a strength above 0.
fn enabled(vault: &VaultSettings, home: Option<&ItemHome>) -> bool {
    home.and_then(|h| h.familiarity).unwrap_or(true) && vault.familiarity_strength > 0.0
}

fn difficulty_mark(conn: &Connection, item: Uid) -> Result<Option<Difficulty>, StoreError> {
    let found: Result<Option<String>, _> = conn.query_row(
        "SELECT difficulty FROM item WHERE uid = ?1",
        [format_ulid(item)],
        |r| r.get(0),
    );
    match found {
        Ok(word) => Ok(word.as_deref().and_then(difficulty_from_name)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(sql_err(e)),
    }
}

/// The item's familiarity after the rating just given, and how many reviews it now has. The answer time of each
/// rating, the reading time of each view and the exam or due date of the stack all count.
fn familiarity_after(
    conn: &Connection,
    home: Option<&ItemHome>,
    (item, rating, timing, now): (Uid, Rating, Option<RatingTiming>, UnixMs),
) -> Result<Option<(f32, u32)>, StoreError> {
    let past = |(r, t): (studyfrog_core::review::Review, Option<(u32, u32)>)| FamEvent::Rating {
        at: r.at,
        rating: r.rating,
        answer_ms: t.map(|t| t.0),
        expected_ms: t.map(|t| t.1),
    };
    let mut events: Vec<FamEvent> =
        load_reviews_timed_in(conn, (item, home.map_or(Uid(0), |h| h.stack)))?
            .into_iter()
            .map(past)
            .collect();
    events.push(FamEvent::Rating {
        at: now,
        rating,
        answer_ms: timing.map(|t| t.answer_ms),
        expected_ms: timing.map(|t| t.expected_ms),
    });
    let reviews = events.len() as u32;
    events.extend(load_views(conn, item)?.into_iter().map(|v| FamEvent::View {
        at: v.at,
        active_ms: v.active_ms,
        expected_ms: v.expected_ms,
    }));
    let exam = exam_date(conn, home, now)?;
    let ctx = FamContext {
        mark: difficulty_mark(conn, item)?,
        exam,
        ..FamContext::default()
    };
    Ok(familiarity(&events, now, &ctx, &fam_params_default()).map(|f| (f.value, reviews)))
}

/// The dates of the exams and deadlines of the item's stack and of the stacks above it.
fn deadline_dates(conn: &Connection, home: Option<&ItemHome>) -> Result<Vec<Date>, StoreError> {
    let Some(home) = home else {
        return Ok(Vec::new());
    };
    let overview = studyfrog_store::overview::load_overview(conn)?;
    let own = overview
        .stacks
        .iter()
        .find(|s| s.uid == home.stack)
        .map(|s| s.folder.clone());
    let above = |uid: Uid| {
        let folder = overview
            .stacks
            .iter()
            .find(|s| s.uid == uid)
            .map(|s| s.folder.as_str());
        match (folder, own.as_deref()) {
            (Some(f), Some(own)) => own == f || own.starts_with(&format!("{f}/")),
            _ => false,
        }
    };
    Ok(load_sessions(conn)?
        .into_iter()
        .filter(|s| matches!(s.kind, EventKind::Exam | EventKind::Deadline))
        .filter(|s| s.stack.is_some_and(above))
        .map(|s| s.start.date)
        .collect())
}

/// The exam or deadline that matters on `today` among `days`: the nearest one still ahead, else the latest one that has passed.
pub(crate) fn pick_exam(days: &[i64], today: i64) -> Option<i64> {
    let ahead = days.iter().filter(|d| **d >= today).min();
    ahead.or_else(|| days.iter().max()).copied()
}

/// The exam or deadline that matters at `now` for the item's stack.
fn exam_date(
    conn: &Connection,
    home: Option<&ItemHome>,
    now: UnixMs,
) -> Result<Option<UnixMs>, StoreError> {
    let days: Vec<i64> = deadline_dates(conn, home)?
        .into_iter()
        .map(date_to_days)
        .collect();
    Ok(pick_exam(&days, now.0.div_euclid(DAY_MS)).map(|d| UnixMs(d * DAY_MS)))
}

/// Days until the nearest exam or deadline of the item's stack or a stack above it; `None` when there is none ahead.
fn days_to_deadline(
    conn: &Connection,
    home: Option<&ItemHome>,
    now: UnixMs,
) -> Result<Option<u32>, StoreError> {
    let today = date_from_days(now.0.div_euclid(DAY_MS));
    let nearest = deadline_dates(conn, home)?
        .into_iter()
        .map(|d| days_between(today, d))
        .filter(|d| *d >= 0)
        .min();
    Ok(nearest.map(|d| d as u32))
}

fn factor_params(vault: &VaultSettings) -> FactorParams {
    FactorParams {
        strength: vault.familiarity_strength,
        ..factor_params_default()
    }
}

/// Put the FSRS item's next due date where its familiarity says. A no-op when familiarity is off for it.
pub fn retime_fsrs(
    conn: &Connection,
    mem: &mut StoredMem,
    given: (Uid, Rating, Option<RatingTiming>),
    (params, vault, home): (&Params, &VaultSettings, Option<&ItemHome>),
) -> Result<(), StoreError> {
    if !enabled(vault, home) {
        return Ok(());
    }
    let now = mem.last_review;
    let Some((fam, reviews)) = familiarity_after(conn, home, (given.0, given.1, given.2, now))?
    else {
        return Ok(());
    };
    let factor = familiarity_factor(fam, mem.state.stability, reviews, &factor_params(vault));
    let base = interval_days(&mem.state, params);
    let days = interval_days_with(&mem.state, factor, params);
    mem.due = due_in(
        now,
        before_deadline(days, base, days_to_deadline(conn, home, now)?),
    );
    Ok(())
}

/// The same for an SM-2 item: the stored interval stays, only the due date moves.
pub fn retime_sm2(
    conn: &Connection,
    stored: &mut StoredSm2,
    given: (Uid, Rating, Option<RatingTiming>),
    (vault, home): (&VaultSettings, Option<&ItemHome>),
) -> Result<(), StoreError> {
    if !enabled(vault, home) {
        return Ok(());
    }
    let now = stored.last_review;
    let Some((fam, reviews)) = familiarity_after(conn, home, (given.0, given.1, given.2, now))?
    else {
        return Ok(());
    };
    let base = stored.state.interval_days;
    let factor = familiarity_factor(fam, base as f32, reviews, &factor_params(vault));
    let days = scaled_interval_days(base, factor, vault.maximum_interval_days);
    stored.due = due_in(
        now,
        before_deadline(days, base, days_to_deadline(conn, home, now)?),
    );
    Ok(())
}
