//! The homework planner: open assignments in due-date order with their urgency, completed ones set apart below.

use eframe::egui::{self, RichText};
use studyfrog_calendar::homework::{HomeworkItem, HomeworkStatus, Urgency, homework_urgency};
use studyfrog_core::civil::{LocalTime, format_date, format_minute, utc_to_local};
use studyfrog_core::time::UnixMs;
use studyfrog_service::timezone::local_offset_minutes;

use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

/// The words shown after a title for an urgency.
pub fn urgency_label(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Completed => "Done",
        Urgency::Overdue => "Overdue",
        Urgency::DueToday => "Due today",
        Urgency::DueTomorrow => "Due tomorrow",
        Urgency::Later => "Upcoming",
    }
}

/// The items split into open and completed, each in due-date order.
pub fn split_by_due(items: &[HomeworkItem]) -> (Vec<&HomeworkItem>, Vec<&HomeworkItem>) {
    let mut sorted: Vec<&HomeworkItem> = items.iter().collect();
    sorted.sort_by_key(|item| item.due);
    sorted
        .into_iter()
        .partition(|item| item.status != HomeworkStatus::Completed)
}

/// The local wall-clock time at `wall_ms` (Unix milliseconds).
pub fn local_now(wall_ms: i64) -> LocalTime {
    let at = UnixMs(wall_ms);
    utc_to_local(at, local_offset_minutes(at))
}

/// Draw the planner for the homework on `state`, judged against the current local time.
pub fn draw(state: &State, ui: &mut egui::Ui, _out: &mut Vec<Msg>) {
    show(ui, &state.homework, local_now(state.wall_ms));
}

/// Draw the planner for `items` at `now`.
pub fn show(ui: &mut egui::Ui, items: &[HomeworkItem], now: LocalTime) {
    kit::section(ui, "Homework");
    if items.is_empty() {
        ui.weak("No homework yet. Assignments you add show up here, by due date.");
        return;
    }
    let (open, done) = split_by_due(items);
    for item in open {
        row(ui, item, homework_urgency(item, now));
    }
    if !done.is_empty() {
        kit::section(ui, "Completed");
        for item in done {
            row(ui, item, Urgency::Completed);
        }
    }
}

/// One assignment: title, subject, due date and time, and the urgency. Completed rows are dimmed.
fn row(ui: &mut egui::Ui, item: &HomeworkItem, urgency: Urgency) {
    let p = palette_of(ui);
    let done = urgency == Urgency::Completed;
    let title_color = if done { p.text_weak } else { p.text };
    let urgency_color = match urgency {
        Urgency::Overdue => p.danger,
        Urgency::DueToday | Urgency::DueTomorrow => p.warning,
        Urgency::Completed | Urgency::Later => p.text_weak,
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new(&item.title).color(title_color).strong());
        if let Some(subject) = &item.subject {
            kit::pill(ui, subject, p.accent);
        }
        let due = format!(
            "{} {}",
            format_date(item.due.date),
            format_minute(item.due.minute)
        );
        ui.label(RichText::new(due).color(title_color));
        ui.label(RichText::new(urgency_label(urgency)).color(urgency_color));
    });
}
