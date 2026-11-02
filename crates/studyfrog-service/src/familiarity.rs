//! Familiarity in scheduling: after a rating, the next due date is put where the item's familiarity says
//! (`docs/research/familiarity-and-fsrs.md`). The stored memory state is never touched.

use rusqlite::Connection;
use studyfrog_calendar::session::EventKind;
use studyfrog_core::civil::{date_from_days, days_between};
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
use studyfrog_store::error::{StoreError, sql_err};
use studyfrog_store::schedule::{StoredMem, StoredSm2, load_reviews};
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

/// The item's familiarity after the rating just given, and how many reviews it now has.
fn familiarity_after(
    conn: &Connection,
    (item, rating, now): (Uid, Rating, UnixMs),
) -> Result<Option<(f32, u32)>, StoreError> {
    let mut events: Vec<FamEvent> = load_reviews(conn, item)?
        .into_iter()
        .map(|r| FamEvent::Rating {
            at: r.at,
            rating: r.rating,
            answer_ms: None,
            expected_ms: None,
        })
        .collect();
    events.push(FamEvent::Rating {
        at: now,
        rating,
        answer_ms: None,
        expected_ms: None,
    });
    let reviews = events.len() as u32;
    events.extend(load_views(conn, item)?.into_iter().map(|v| FamEvent::View {
        at: v.at,
        active_ms: v.active_ms,
        expected_ms: None,
    }));
    let ctx = FamContext {
        mark: difficulty_mark(conn, item)?,
        ..FamContext::default()
    };
    Ok(familiarity(&events, now, &ctx, &fam_params_default()).map(|f| (f.value, reviews)))
}

/// Days until the nearest exam or deadline of the item's stack or a stack above it; `None` when there is none.
fn days_to_deadline(
    conn: &Connection,
    home: Option<&ItemHome>,
    now: UnixMs,
) -> Result<Option<u32>, StoreError> {
    let Some(home) = home else { return Ok(None) };
    let today = date_from_days(now.0.div_euclid(DAY_MS));
    let ours = studyfrog_store::overview::load_overview(conn)?;
    let own_folder = ours
        .stacks
        .iter()
        .find(|s| s.uid == home.stack)
        .map(|s| s.folder.clone());
    let above = |uid: Uid| {
        let folder = ours
            .stacks
            .iter()
            .find(|s| s.uid == uid)
            .map(|s| s.folder.as_str());
        match (folder, own_folder.as_deref()) {
            (Some(f), Some(own)) => own == f || own.starts_with(&format!("{f}/")),
            _ => false,
        }
    };
    let nearest = load_sessions(conn)?
        .into_iter()
        .filter(|s| matches!(s.kind, EventKind::Exam | EventKind::Deadline))
        .filter(|s| s.stack.is_some_and(above))
        .map(|s| days_between(today, s.start.date))
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
    given: (Uid, Rating),
    (params, vault, home): (&Params, &VaultSettings, Option<&ItemHome>),
) -> Result<(), StoreError> {
    if !enabled(vault, home) {
        return Ok(());
    }
    let now = mem.last_review;
    let Some((fam, reviews)) = familiarity_after(conn, (given.0, given.1, now))? else {
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
    given: (Uid, Rating),
    (vault, home): (&VaultSettings, Option<&ItemHome>),
) -> Result<(), StoreError> {
    if !enabled(vault, home) {
        return Ok(());
    }
    let now = stored.last_review;
    let Some((fam, reviews)) = familiarity_after(conn, (given.0, given.1, now))? else {
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
