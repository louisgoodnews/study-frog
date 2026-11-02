//! The tabs of a stack page and the one remembered for each stack. Pure.

use studyfrog_core::settings::{AppSettings, MAX_STACK_TABS};

/// A tab of the stack page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackTab {
    /// Counts, charts, activity and sub-stacks.
    Summary,
    /// The items of the stack as a table, with its due date and PDFs.
    List,
    /// What is coming up and what is not planned yet.
    Backlog,
    /// The stacks inside this one as a tree.
    Substacks,
    /// The items and stacks on a time axis.
    Roadmap,
    /// The items in a column for each status.
    Board,
    /// The stack's sessions and planned reviews.
    Calendar,
    /// The stack's familiarity report.
    Reports,
    /// What the user wants to reach with the stack.
    Goals,
}

/// The tabs in the order shown, with their stored key and name.
pub const TABS: [(StackTab, &str, &str); 9] = [
    (StackTab::Summary, "summary", "Summary"),
    (StackTab::List, "list", "List"),
    (StackTab::Backlog, "backlog", "Backlog"),
    (StackTab::Roadmap, "roadmap", "Roadmap"),
    (StackTab::Board, "board", "Board"),
    (StackTab::Calendar, "calendar", "Calendar"),
    (StackTab::Reports, "reports", "Reports"),
    (StackTab::Substacks, "substacks", "Sub-stacks"),
    (StackTab::Goals, "goals", "Goals"),
];

/// The name of a tab.
pub fn tab_label(t: StackTab) -> &'static str {
    TABS.iter()
        .find(|(x, _, _)| *x == t)
        .map_or("", |(_, _, l)| l)
}

/// The tab remembered for the stack at `folder`, else Summary.
pub fn tab_of(app: &AppSettings, folder: &str) -> StackTab {
    let key = app
        .stack_tabs
        .iter()
        .filter_map(|e| e.rsplit_once(':'))
        .find(|(f, _)| *f == folder)
        .map(|(_, k)| k);
    TABS.iter()
        .find(|(_, k, _)| Some(*k) == key)
        .map_or(StackTab::Summary, |(t, _, _)| *t)
}

pub(crate) fn remembered(app: &AppSettings, folder: &str, tab: StackTab) -> AppSettings {
    let key = TABS
        .iter()
        .find(|(t, _, _)| *t == tab)
        .map_or("summary", |(_, k, _)| *k);
    let mut tabs: Vec<String> = app
        .stack_tabs
        .iter()
        .filter(|e| e.rsplit_once(':').is_none_or(|(f, _)| f != folder))
        .cloned()
        .collect();
    tabs.push(format!("{folder}:{key}"));
    let skip = tabs.len().saturating_sub(MAX_STACK_TABS);
    AppSettings {
        stack_tabs: tabs.split_off(skip),
        ..app.clone()
    }
}
