//! The Roadmap tab of a stack: a row for each stack inside it and each item, with a bar from its first date to its next one.
//! The layout is pure and cheap; only the rows in view are drawn. Zoom sets how many pixels a day takes.

use studyfrog_core::status::Status;
use studyfrog_core::uid::Uid;
use studyfrog_store::overview::StackRow;

use crate::library_ui::LibraryState;

/// How much of the time axis is shown at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Zoom {
    /// A few weeks, one tick a day.
    Today,
    /// A few months, one tick a week.
    #[default]
    Weeks,
    /// About a year, one tick a month.
    Months,
    /// Several years, one tick a quarter.
    Quarters,
}

/// The zooms in the order of the buttons, with their names.
pub const ZOOMS: [(Zoom, &str); 4] = [
    (Zoom::Today, "Today"),
    (Zoom::Weeks, "Weeks"),
    (Zoom::Months, "Months"),
    (Zoom::Quarters, "Quarters"),
];

/// Pixels per day.
pub fn px_per_day(z: Zoom) -> f32 {
    match z {
        Zoom::Today => 36.0,
        Zoom::Weeks => 10.0,
        Zoom::Months => 3.0,
        Zoom::Quarters => 1.0,
    }
}

/// Days between two ticks of the axis.
pub fn tick_days(z: Zoom) -> i64 {
    match z {
        Zoom::Today => 1,
        Zoom::Weeks => 7,
        Zoom::Months => 30,
        Zoom::Quarters => 91,
    }
}

/// A row of the roadmap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoadRow {
    /// Its name.
    pub title: String,
    /// 0 for an item, 1 and more for a stack by depth.
    pub depth: usize,
    /// The item, or `None` for a stack.
    pub uid: Option<Uid>,
    /// The stack's folder, or `None` for an item.
    pub folder: Option<String>,
    /// Its status.
    pub status: Status,
    /// First day of its bar (counted from 1970-01-01).
    pub start: i64,
    /// Last day of its bar, never before `start`.
    pub end: i64,
}

fn span(dates: impl Iterator<Item = (i64, i64)>, today: i64) -> (i64, i64) {
    let (mut lo, mut hi) = (i64::MAX, i64::MIN);
    for (a, b) in dates {
        lo = lo.min(a);
        hi = hi.max(b);
    }
    if lo > hi { (today, today) } else { (lo, hi) }
}

fn item_span(row: &studyfrog_core::query_eval::Row, today: i64) -> (i64, i64) {
    let start = row.created_day.unwrap_or(today);
    (start, row.due_day.unwrap_or(start).max(start))
}

/// The rows for the stack at `folder`: its stacks inside (tree order), then the items directly in it by first date.
pub fn rows(lib: &LibraryState, stacks: &[StackRow], folder: &str, today: i64) -> Vec<RoadRow> {
    let prefix = format!("{folder}/");
    let mut out: Vec<RoadRow> = stacks
        .iter()
        .filter(|s| s.folder.starts_with(&prefix))
        .map(|s| {
            let inside = lib
                .items
                .iter()
                .filter(|i| i.row.stacks.iter().step_by(2).any(|c| *c == s.folder));
            let (start, end) = span(inside.map(|i| item_span(&i.row, today)), today);
            RoadRow {
                title: s.title.clone(),
                depth: s.folder[prefix.len()..].matches('/').count() + 1,
                uid: None,
                folder: Some(s.folder.clone()),
                status: s.status.unwrap_or(Status::New),
                start,
                end,
            }
        })
        .collect();
    let mut items: Vec<RoadRow> = lib
        .items
        .iter()
        .filter(|i| {
            (i.row.stacks.len() >= 2 && i.row.stacks[i.row.stacks.len() - 2] == folder)
                || i.references
                    .iter()
                    .any(|r| r.folder.as_deref() == Some(folder))
        })
        .map(|i| {
            let (start, end) = item_span(&i.row, today);
            let title = if !i.row.stacks.iter().step_by(2).any(|s| s == folder) {
                let from = i
                    .references
                    .iter()
                    .find(|r| r.primary)
                    .map_or("Inbox", |r| r.title.as_str());
                format!("{} (from {from})", i.row.title)
            } else {
                i.row.title.clone()
            };
            RoadRow {
                title,
                depth: 0,
                uid: Some(i.uid),
                folder: None,
                status: i.row.status,
                start,
                end,
            }
        })
        .collect();
    items.sort_by_key(|r| (r.start, r.uid));
    out.extend(items);
    out
}

/// The first and last day of the axis: the rows and today, with a margin.
pub fn axis(rows: &[RoadRow], today: i64) -> (i64, i64) {
    let (lo, hi) = span(
        rows.iter()
            .map(|r| (r.start, r.end))
            .chain(std::iter::once((today, today))),
        today,
    );
    (lo - 7, hi + 14)
}

/// Left edge and width in pixels of a bar on an axis that starts at `from`; a bar is at least a day wide.
pub fn bar(row: &RoadRow, from: i64, ppd: f32) -> (f32, f32) {
    (
        (row.start - from) as f32 * ppd,
        ((row.end - row.start + 1) as f32 * ppd).max(ppd.max(4.0)),
    )
}

/// The indices of the rows in view for a scroll offset and viewport height, with one row of slack.
pub fn visible(count: usize, row_height: f32, top: f32, height: f32) -> std::ops::Range<usize> {
    let first = (top / row_height).floor().max(0.0) as usize;
    let last = ((top + height) / row_height).ceil() as usize + 1;
    first.min(count)..last.min(count)
}
