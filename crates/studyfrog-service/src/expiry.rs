//! Stacks whose due date has passed. They are left out of the suggested queue and of the home page's
//! counts; choosing such a stack by hand in the run setup still works.

use std::collections::HashSet;

use rusqlite::Connection;
use studyfrog_calendar::session::{EventKind, Repeat};
use studyfrog_core::civil::{Date, date_from_days};
use studyfrog_core::uid::Uid;
use studyfrog_store::error::StoreError;
use studyfrog_store::overview::{IndexedItem, load_overview};
use studyfrog_store::sessions::load_sessions;

use crate::day::DayWindow;

const HALF_DAY_MS: i64 = 43_200_000;
const DAY_MS: i64 = 86_400_000;

/// The local date of a day window: the date at its middle, which is right for every offset within
/// twelve hours of UTC.
pub fn today_of(day: DayWindow) -> Date {
    date_from_days((day.start.0 + HALF_DAY_MS).div_euclid(DAY_MS))
}

/// The stacks with a deadline before `today`, and every stack inside them.
pub fn expired_stacks(conn: &Connection, today: Date) -> Result<HashSet<Uid>, StoreError> {
    let past: HashSet<Uid> = load_sessions(conn)?
        .into_iter()
        .filter(|s| {
            s.kind == EventKind::Deadline && s.repeat == Repeat::Never && s.start.date < today
        })
        .filter_map(|s| s.stack)
        .collect();
    if past.is_empty() {
        return Ok(HashSet::new());
    }
    let stacks = load_overview(conn)?.stacks;
    let folders: Vec<&str> = stacks
        .iter()
        .filter(|s| past.contains(&s.uid))
        .map(|s| s.folder.as_str())
        .collect();
    let inside = |folder: &str| {
        folders
            .iter()
            .any(|f| folder == *f || folder.starts_with(&format!("{f}/")))
    };
    Ok(stacks
        .iter()
        .filter(|s| inside(&s.folder))
        .map(|s| s.uid)
        .collect())
}

/// The items that are not in an expired stack.
pub fn without_expired(items: Vec<IndexedItem>, expired: &HashSet<Uid>) -> Vec<IndexedItem> {
    items
        .into_iter()
        .filter(|i| !expired.contains(&i.stack))
        .collect()
}
