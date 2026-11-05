//! The parent-stack picker and the due-date fields of the Create dialog.

use eframe::egui::{self, ComboBox, Role, WidgetInfo};

use studyfrog_service::create_more::NewKind;

use crate::modal::{CreateForm, Field, ModalMsg, stack_labels};
use crate::msg::Msg;
use crate::session_form::REMINDER_CHOICES;
use crate::state::State;
use crate::view::date_picker::{date_field, today_of};

fn send(out: &mut Vec<Msg>, m: ModalMsg) {
    out.push(Msg::Modal(m));
}

/// A stack's optional due date and the reminders before it.
pub(super) fn due_fields(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if let Some(t) = date_field(ui, "Due date (optional)", &f.due, today_of(state)) {
        send(out, ModalMsg::Edit(Field::Due, t));
    }
    if f.due.trim().is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Remind me:");
        for (minutes, label) in REMINDER_CHOICES {
            if ui
                .selectable_label(f.reminders.contains(&minutes), label)
                .clicked()
            {
                send(out, ModalMsg::ToggleReminder(minutes));
            }
        }
    });
}

pub(super) fn parent_picker(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks = state
        .vault
        .as_ref()
        .map(|v| stack_labels(&v.overview))
        .unwrap_or_default();
    let is_stack = f.kind == NewKind::Stack;
    if stacks.is_empty() && !is_stack {
        ui.colored_label(crate::theme::palette_of(ui).warning, "There is no stack yet. Every card, note and question lives in a stack: create a stack first.");
        return;
    }
    let none = "None (top level)";
    let current = f
        .parent
        .as_ref()
        .and_then(|p| stacks.iter().find(|(folder, _)| folder == p))
        .map_or(none.to_owned(), |(_, label)| label.clone());
    let label = if is_stack {
        "Inside stack (optional)"
    } else {
        "Parent stack"
    };
    let picker = ComboBox::from_id_salt("parent-stack")
        .selected_text(current)
        .width(220.0)
        .show_ui(ui, |ui| {
            if is_stack && ui.selectable_label(f.parent.is_none(), none).clicked() {
                send(out, ModalMsg::SetParent(None));
            }
            for (folder, label) in &stacks {
                if ui
                    .selectable_label(f.parent.as_deref() == Some(folder), label)
                    .clicked()
                {
                    send(out, ModalMsg::SetParent(Some(folder.clone())));
                }
            }
        });
    picker
        .response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, label));
}

pub(super) fn also_in_picker(state: &State, f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if f.kind == NewKind::Stack {
        return;
    }
    let stacks = state
        .vault
        .as_ref()
        .map(|v| stack_labels(&v.overview))
        .unwrap_or_default();
    let none = "Also in: none";
    let current = f
        .also_in
        .as_ref()
        .and_then(|p| stacks.iter().find(|(folder, _)| folder == p))
        .map_or(none.to_owned(), |(_, label)| format!("Also in: {label}"));
    let picker = ComboBox::from_id_salt("also-in-stack")
        .selected_text(current)
        .width(180.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(f.also_in.is_none(), none).clicked() {
                send(out, ModalMsg::SetAlsoIn(None));
            }
            for (folder, label) in &stacks {
                if f.parent.as_deref() != Some(folder)
                    && ui
                        .selectable_label(f.also_in.as_deref() == Some(folder), label)
                        .clicked()
                {
                    send(out, ModalMsg::SetAlsoIn(Some(folder.clone())));
                }
            }
        });
    picker
        .response
        .widget_info(|| WidgetInfo::labeled(Role::ComboBox, true, "Also in (optional)"));
}

const DIFFICULTIES: [studyfrog_core::event::Difficulty; 3] = [
    studyfrog_core::event::Difficulty::Easy,
    studyfrog_core::event::Difficulty::Medium,
    studyfrog_core::event::Difficulty::Difficult,
];

fn choice<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    (current, options): (T, &[(T, &str)]),
) -> Option<T> {
    let shown = options
        .iter()
        .find(|(v, _)| *v == current)
        .map_or("", |(_, n)| *n);
    let mut picked = None;
    ComboBox::from_label(label)
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for (v, name) in options {
                if ui.selectable_label(current == *v, *name).clicked() {
                    picked = Some(*v);
                }
            }
        });
    picked
}

pub(super) fn priority_choice(f: &CreateForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    use studyfrog_core::priority::{PRIORITIES, Priority, priority_name};
    let priorities: Vec<(Priority, &str)> =
        PRIORITIES.iter().map(|p| (*p, priority_name(*p))).collect();
    if let Some(p) = choice(ui, "Priority", (f.priority, &priorities)) {
        send(out, ModalMsg::SetPriority(p));
    }
}

pub(super) fn metadata_fields(
    state: &State,
    f: &CreateForm,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    use studyfrog_core::event::difficulty_name;
    ui.horizontal_wrapped(|ui| {
        priority_choice(f, ui, out);
        let levels: Vec<(_, &str)> = DIFFICULTIES
            .iter()
            .map(|d| (*d, difficulty_name(*d)))
            .collect();
        if let Some(d) = choice(ui, "Difficulty", (f.difficulty, &levels)) {
            send(out, ModalMsg::SetDifficulty(d));
        }
    });
    if let Some(t) = crate::view::widgets::text_field(ui, "Tags", &f.tags, "comma separated", 1) {
        send(out, ModalMsg::Edit(Field::Tags, t));
    }
    due_fields(state, f, ui, out);
}
