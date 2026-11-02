//! The home screen: what is ready today, how the streak stands, what is planned, and what happened
//! recently. Read from the index and the stored schedule only.

use std::collections::{BTreeSet, HashMap};

use rusqlite::Connection;
use studyfrog_achieve::stats::streak_days;
use studyfrog_calendar::recurrence::occurrences_all;
use studyfrog_calendar::session::{EventKind, Session};
use studyfrog_core::civil::{Date, add_days, utc_to_local};
use studyfrog_core::plan::plan_spaced;
use studyfrog_core::time::UnixMs;
use studyfrog_core::uid::Uid;
use studyfrog_rehearsal::queue::build::build_queue;
use studyfrog_rehearsal::queue::candidate::{Budgets, budget_unlimited};
use studyfrog_store::achievements::load_review_rows;
use studyfrog_store::home::{RunRow, count_reviews_between, recent_runs};
use studyfrog_store::overview::{
    IndexedItem, ItemRow, due_items, load_indexed_items, load_overview, recent_items,
};
use studyfrog_store::schedule::load_schedules;
use studyfrog_store::settings::item_homes;

use crate::day::DayWindow;
use crate::expiry::{expired_stacks, today_of, without_expired};
use crate::planning::list_sessions;
use crate::queue::{candidates, stack_budgets};
use crate::session::{ServiceError, VaultSession, current_settings};
use crate::settings::{params_for, scheduler_for};

const MS_PER_DAY: i64 = 86_400_000;

/// How much of one stack is waiting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackDue {
    /// The stack.
    pub uid: Uid,
    /// Its folder.
    pub folder: String,
    /// Its title.
    pub title: String,
    /// Items scheduled and due now.
    pub due: usize,
    /// Items never reviewed.
    pub new: usize,
}

/// A calendar entry in the coming days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Upcoming {
    /// Title.
    pub title: String,
    /// What kind of entry.
    pub kind: EventKind,
    /// Local date.
    pub date: Date,
    /// Local start, minutes since midnight.
    pub start_minute: u16,
}

/// How many days ahead the home page looks.
pub const UPCOMING_DAYS: i64 = 7;

/// A study session happening today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodaySession {
    /// Its title.
    pub title: String,
    /// Local start, minutes since midnight.
    pub start_minute: u16,
    /// Length in minutes.
    pub duration_min: u16,
    /// The title of its stack, if it has one.
    pub stack: Option<String>,
}

/// Everything the home screen shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HomeSummary {
    /// The moment it was computed, in Unix milliseconds (for "2 days ago").
    pub now_ms: i64,
    /// The local date.
    pub today: Date,
    /// Local hour of the day, 0 to 23 (for the greeting).
    pub hour: u8,
    /// Items in today's spaced run: what is due plus the new ones the daily limits allow.
    pub ready_today: usize,
    /// Items scheduled and due now.
    pub due_now: usize,
    /// Items never reviewed.
    pub new_items: usize,
    /// Reviews recorded today.
    pub reviews_today: u32,
    /// Consecutive days with a review.
    pub streak_days: u32,
    /// Per stack, in stack order.
    pub stacks: Vec<StackDue>,
    /// The items edited most recently.
    pub recent_items: Vec<ItemRow>,
    /// The runs started most recently.
    pub recent_runs: Vec<RunRow>,
    /// Sessions planned for today, in time order.
    pub sessions_today: Vec<TodaySession>,
    /// The items that are due now, the most overdue first.
    pub due_items: Vec<ItemRow>,
    /// Calendar entries of the next days (not today).
    pub upcoming: Vec<Upcoming>,
}

/// The most overdue items, leaving out those of expired stacks.
fn kept_due_items(
    conn: &Connection,
    now: UnixMs,
    kept: &[IndexedItem],
) -> Result<Vec<ItemRow>, ServiceError> {
    let keep: BTreeSet<Uid> = kept.iter().map(|i| i.uid).collect();
    let mut rows = due_items(conn, now.0, 64).map_err(ServiceError::Store)?;
    rows.retain(|r| keep.contains(&r.uid));
    rows.truncate(8);
    Ok(rows)
}

fn upcoming(sessions: &[Session], today: Date) -> Vec<Upcoming> {
    let (from, to) = (add_days(today, 1), add_days(today, UPCOMING_DAYS));
    let mut list: Vec<Upcoming> = occurrences_all(sessions, from, to)
        .into_iter()
        .filter_map(|o| {
            let s = sessions.iter().find(|s| s.id == o.session)?;
            Some(Upcoming {
                title: s.title.clone(),
                kind: s.kind,
                date: o.date,
                start_minute: o.start_minute,
            })
        })
        .collect();
    list.sort_by_key(|u| (u.date, u.start_minute));
    list
}

/// Compute the summary at `now`; `offset_minutes` is the local UTC offset and `day` the local day.
pub fn home_summary(
    session: &VaultSession,
    now: UnixMs,
    offset_minutes: i32,
    day: DayWindow,
) -> Result<HomeSummary, ServiceError> {
    let store = ServiceError::Store;
    let conn = &session.conn;
    let vault = current_settings(session);
    let homes = item_homes(conn).map_err(store)?;
    let sched =
        load_schedules(conn, |uid| scheduler_for(&vault, homes.get(&uid))).map_err(store)?;
    let expired = expired_stacks(conn, today_of(day)).map_err(store)?;
    let items = without_expired(load_indexed_items(conn).map_err(store)?, &expired);
    let cands = candidates(&items, &sched, &HashMap::new());
    let overview = load_overview(conn).map_err(ServiceError::Store)?;

    let mut per: HashMap<Uid, (usize, usize)> = HashMap::new();
    for c in &cands {
        let e = per.entry(c.stack).or_default();
        match c.due {
            Some(due) if due <= now => e.0 += 1,
            None => e.1 += 1,
            Some(_) => {}
        }
    }
    let stacks: Vec<StackDue> = overview
        .stacks
        .iter()
        .map(|s| {
            let (due, new) = per.get(&s.uid).copied().unwrap_or_default();
            StackDue {
                uid: s.uid,
                folder: s.folder.clone(),
                title: s.title.clone(),
                due,
                new,
            }
        })
        .collect();
    let budgets = Budgets {
        global: budget_unlimited(),
        per_stack: stack_budgets(conn, day).map_err(ServiceError::Store)?,
    };
    let ready = build_queue(
        &cands,
        &plan_spaced(),
        now,
        &budgets,
        &params_for(&vault, None),
    )
    .len();

    let local_now = utc_to_local(now, offset_minutes);
    let days: BTreeSet<i64> = load_review_rows(conn)
        .map_err(ServiceError::Store)?
        .iter()
        .map(|r| (r.at_ms + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY))
        .collect();
    let today_index = (now.0 + i64::from(offset_minutes) * 60_000).div_euclid(MS_PER_DAY);
    let sessions = list_sessions(session)?;
    let title_of = |uid: Uid| {
        overview
            .stacks
            .iter()
            .find(|s| s.uid == uid)
            .map(|s| s.title.clone())
    };
    let sessions_today = occurrences_all(&sessions, local_now.date, local_now.date)
        .into_iter()
        .filter_map(|o| {
            let s = sessions.iter().find(|s| s.id == o.session)?;
            Some(TodaySession {
                title: s.title.clone(),
                start_minute: o.start_minute,
                duration_min: o.duration_min,
                stack: s.stack.and_then(title_of),
            })
        })
        .collect();
    Ok(HomeSummary {
        now_ms: now.0,
        today: local_now.date,
        hour: (local_now.minute / 60) as u8,
        ready_today: ready,
        due_now: stacks.iter().map(|s| s.due).sum(),
        new_items: stacks.iter().map(|s| s.new).sum(),
        reviews_today: count_reviews_between(conn, day.start.0, day.end.0)
            .map_err(ServiceError::Store)?,
        streak_days: streak_days(&days, today_index),
        stacks,
        recent_items: recent_items(conn, 6).map_err(ServiceError::Store)?,
        recent_runs: recent_runs(conn, 5).map_err(ServiceError::Store)?,
        sessions_today,
        due_items: kept_due_items(conn, now, &items)?,
        upcoming: upcoming(&sessions, local_now.date),
    })
}
