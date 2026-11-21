//! Urgency and ordering for the homework planner (STORY-0167). Pure.

use std::cmp::Ordering;

use studyfrog_core::civil::{Date, LocalTime, add_days, date_to_days};
use studyfrog_core::uid::Uid;

use crate::homework::{HomeworkItem, HomeworkPriority, HomeworkStatus};

/// How pressing an item is at a given moment. Used for filters and badges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Urgency {
    /// Open, and the due time has passed.
    Overdue,
    /// Open, due later today.
    DueToday,
    /// Open, due within the next seven days (after today).
    Upcoming,
    /// Open, due later than the next seven days.
    Later,
    /// Completed.
    Completed,
}

/// Classify an item relative to `now`.
pub fn urgency(item: &HomeworkItem, now: LocalTime) -> Urgency {
    if item.status == HomeworkStatus::Completed {
        Urgency::Completed
    } else if item.due < now {
        Urgency::Overdue
    } else if item.due.date == now.date {
        Urgency::DueToday
    } else if item.due.date <= add_days(now.date, 7) {
        Urgency::Upcoming
    } else {
        Urgency::Later
    }
}

/// The items due on `day`, earliest due time first.
pub fn due_on(items: &[HomeworkItem], day: Date) -> Vec<&HomeworkItem> {
    let mut out: Vec<&HomeworkItem> = items.iter().filter(|i| i.due.date == day).collect();
    out.sort_by_key(|i| i.due);
    out
}

/// The most pressing urgency among the items, or `None` when there are none.
pub fn most_urgent(items: &[&HomeworkItem], now: LocalTime) -> Option<Urgency> {
    items
        .iter()
        .map(|i| urgency(i, now))
        .min_by_key(|u| rank(*u))
}

/// The quick-filter tabs of the homework planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HomeworkTab {
    /// Everything, completed items included.
    #[default]
    All,
    /// Everything not yet completed.
    Active,
    /// Open items due today.
    DueToday,
    /// Open items past their due time.
    Overdue,
    /// Completed items.
    Completed,
}

/// Every tab, in display order.
pub const HOMEWORK_TABS: [HomeworkTab; 5] = [
    HomeworkTab::All,
    HomeworkTab::Active,
    HomeworkTab::DueToday,
    HomeworkTab::Overdue,
    HomeworkTab::Completed,
];

/// The filter a tab shows, optionally narrowed to one course.
pub fn tab_filter(tab: HomeworkTab, course: Option<Uid>) -> HomeworkFilter {
    let (show_completed, urgency) = match tab {
        HomeworkTab::All => (true, None),
        HomeworkTab::Active => (false, None),
        HomeworkTab::DueToday => (false, Some(Urgency::DueToday)),
        HomeworkTab::Overdue => (false, Some(Urgency::Overdue)),
        HomeworkTab::Completed => (true, Some(Urgency::Completed)),
    };
    HomeworkFilter {
        course,
        urgency,
        show_completed,
    }
}

/// How far away the due time is, for a label: "due in 3 h", "overdue by 2 d", or "done".
pub fn countdown(item: &HomeworkItem, now: LocalTime) -> String {
    if item.status == HomeworkStatus::Completed {
        return "done".to_owned();
    }
    let minutes = (date_to_days(item.due.date) - date_to_days(now.date)) * 1440
        + i64::from(item.due.minute)
        - i64::from(now.minute);
    if minutes < 0 {
        format!("overdue by {}", span(-minutes))
    } else {
        format!("due in {}", span(minutes))
    }
}

fn span(minutes: i64) -> String {
    if minutes < 60 {
        format!("{minutes} min")
    } else if minutes < 1440 {
        format!("{} h", minutes / 60)
    } else {
        format!("{} d", minutes / 1440)
    }
}

/// Which items the planner shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HomeworkFilter {
    /// Only this course, if set.
    pub course: Option<Uid>,
    /// Only this urgency, if set.
    pub urgency: Option<Urgency>,
    /// Whether completed items are shown.
    pub show_completed: bool,
}

/// Whether an item passes the filter at `now`.
pub fn matches_filter(item: &HomeworkItem, filter: &HomeworkFilter, now: LocalTime) -> bool {
    let u = urgency(item, now);
    if !filter.show_completed && u == Urgency::Completed {
        return false;
    }
    filter.course.is_none_or(|c| item.course == Some(c))
        && filter.urgency.is_none_or(|want| want == u)
}

/// The filtered items in planner order: overdue first, then by due time, higher priority first on
/// ties.
pub fn planner_order(
    items: &[HomeworkItem],
    filter: &HomeworkFilter,
    now: LocalTime,
) -> Vec<HomeworkItem> {
    let mut out: Vec<HomeworkItem> = items
        .iter()
        .filter(|i| matches_filter(i, filter, now))
        .cloned()
        .collect();
    out.sort_by(|a, b| compare(a, b, now));
    out
}

fn rank(u: Urgency) -> u8 {
    match u {
        Urgency::Overdue => 0,
        Urgency::DueToday => 1,
        Urgency::Upcoming => 2,
        Urgency::Later => 3,
        Urgency::Completed => 4,
    }
}

fn compare(a: &HomeworkItem, b: &HomeworkItem, now: LocalTime) -> Ordering {
    rank(urgency(a, now))
        .cmp(&rank(urgency(b, now)))
        .then(a.due.cmp(&b.due))
        .then(priority_rank(b.priority).cmp(&priority_rank(a.priority)))
        .then(a.id.0.cmp(&b.id.0))
}

fn priority_rank(p: HomeworkPriority) -> u8 {
    match p {
        HomeworkPriority::Low => 0,
        HomeworkPriority::Medium => 1,
        HomeworkPriority::High => 2,
    }
}
