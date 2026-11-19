//! Deterministic rendering of `.studyfrog/homework.toml`.

use std::fmt::Write as _;

use crate::homework::{
    HomeworkFileDto, HomeworkItemDto, canonicalize_homework_dto, sort_homework_dtos,
};
use crate::values::toml_string;

fn push_opt(out: &mut String, key: &str, val: Option<&str>) {
    if let Some(v) = val {
        let _ = writeln!(out, "{key} = {}", toml_string(v));
    }
}

fn push_item(out: &mut String, item: &HomeworkItemDto) {
    let _ = writeln!(out, "\n[[homework]]");
    let _ = writeln!(out, "id = {}", toml_string(&item.id));
    let _ = writeln!(out, "title = {}", toml_string(&item.title));
    push_opt(out, "subject", item.subject.as_deref());
    push_opt(out, "stack_uid", item.stack_uid.as_deref());
    push_opt(out, "timetable_id", item.timetable_id.as_deref());
    let _ = writeln!(out, "due_date = {}", toml_string(&item.due_date));
    let _ = writeln!(out, "due_time = {}", toml_string(&item.due_time));
    push_opt(out, "start_date", item.start_date.as_deref());
    let _ = writeln!(out, "estimated_minutes = {}", item.estimated_minutes);
    let _ = writeln!(out, "priority = {}", toml_string(&item.priority));
    let _ = writeln!(out, "status = {}", toml_string(&item.status));
    if !item.notes.is_empty() {
        let _ = writeln!(out, "notes = {}", toml_string(&item.notes));
    }
    if !item.reminders.is_empty() {
        let list: Vec<String> = item.reminders.iter().map(ToString::to_string).collect();
        let _ = writeln!(out, "reminders = [{}]", list.join(", "));
    }
    for entry in &item.checklist {
        let _ = writeln!(
            out,
            "\n[[homework.checklist]]\ntext = {}\ndone = {}",
            toml_string(&entry.text),
            entry.done
        );
    }
}

/// Render homework items deterministically to TOML text.
pub fn render_homework_toml(file: &HomeworkFileDto) -> String {
    let mut items: Vec<HomeworkItemDto> = file
        .homework
        .iter()
        .map(canonicalize_homework_dto)
        .collect();
    sort_homework_dtos(&mut items);
    let mut out = String::from("# Homework assignments of this vault (edited in StudyFrog).\n");
    for item in &items {
        push_item(&mut out, item);
    }
    out
}
