//! The answers of `homework ...`: the items as the app last read them, and why the last change was
//! refused.

use serde_json::{Value, json};
use studyfrog_calendar::homework::{
    HomeworkItem, HomeworkPriority, HomeworkStatus, checklist_progress,
};
use studyfrog_calendar::homework_query::{
    HomeworkTab, Urgency, countdown, planner_order, tab_filter, urgency,
};
use studyfrog_core::civil::{format_date, format_minute};

use crate::control_state::ulid;
use crate::state::State;

fn name_of_status(s: HomeworkStatus) -> &'static str {
    match s {
        HomeworkStatus::NotStarted => "not_started",
        HomeworkStatus::InProgress => "in_progress",
        HomeworkStatus::Completed => "completed",
    }
}

fn name_of_priority(p: HomeworkPriority) -> &'static str {
    match p {
        HomeworkPriority::Low => "low",
        HomeworkPriority::Medium => "medium",
        HomeworkPriority::High => "high",
    }
}

fn name_of_urgency(u: Urgency) -> &'static str {
    match u {
        Urgency::Overdue => "overdue",
        Urgency::DueToday => "due_today",
        Urgency::Upcoming => "due_soon",
        Urgency::Later => "later",
        Urgency::Completed => "completed",
    }
}

fn row(item: &HomeworkItem, now: studyfrog_core::civil::LocalTime) -> Value {
    let (done, total) = checklist_progress(item);
    json!({
        "id": ulid(item.id),
        "title": item.title,
        "course": item.course.map(ulid),
        "due_date": format_date(item.due.date),
        "due_time": format_minute(item.due.minute),
        "estimate_min": item.estimate_min,
        "priority": name_of_priority(item.priority),
        "status": name_of_status(item.status),
        "urgency": name_of_urgency(urgency(item, now)),
        "countdown": countdown(item, now),
        "steps": { "done": done, "total": total },
    })
}

/// The homework list of the active tab, in planner order, as the app last read it.
pub fn list_json(state: &State) -> Value {
    let Some(now) = state.planner.homework_now else {
        return json!({ "loaded": false, "count": 0, "items": [] });
    };
    let tab: HomeworkTab = state.planner.homework_ui.tab;
    let items: Vec<Value> = planner_order(&state.planner.homework, &tab_filter(tab, None), now)
        .iter()
        .map(|i| row(i, now))
        .collect();
    json!({ "loaded": true, "count": items.len(), "items": items })
}

/// The reason the homework dialog or a change refused, if it did.
pub fn refusal(state: &State) -> Option<String> {
    state.planner.homework_ui.error.clone()
}

fn tab_name(tab: studyfrog_calendar::homework_query::HomeworkTab) -> &'static str {
    use studyfrog_calendar::homework_query::HomeworkTab;
    match tab {
        HomeworkTab::All => "all",
        HomeworkTab::Active => "active",
        HomeworkTab::DueToday => "due-today",
        HomeworkTab::Overdue => "overdue",
        HomeworkTab::Completed => "completed",
    }
}

/// The tab in front and the subject (stack folder) the list is narrowed to, if any.
pub fn tab_json(state: &State) -> Value {
    let ui = &state.planner.homework_ui;
    let subject = ui.subject.and_then(|uid| {
        state
            .vault
            .as_ref()
            .and_then(|v| v.overview.stacks.iter().find(|s| s.uid == uid))
            .map(|s| s.folder.clone())
    });
    json!({"tab": tab_name(ui.tab), "subject": subject})
}
