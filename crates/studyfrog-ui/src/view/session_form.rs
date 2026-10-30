//! The session form of the calendar.

use eframe::egui::{self, ComboBox};
use studyfrog_calendar::session::{EVENT_KINDS, event_kind_name};
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::session_form::{EndsKind, FormEdit, REMINDER_CHOICES, RepeatKind, SessionForm};
use crate::state::State;
use crate::view::widgets::text_field;

const DAY_NAMES: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

fn edit(out: &mut Vec<Msg>, e: FormEdit) {
    out.push(Msg::Plan(PlanMsg::Form(e)));
}

fn field(
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
    label: &str,
    value: &str,
    make: fn(String) -> FormEdit,
) {
    if let Some(text) = text_field(ui, label, value, "", 1) {
        edit(out, make(text));
    }
}

/// Draw the form for a new or existing session.
pub fn draw(state: &State, f: &SessionForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.heading(if f.editing.is_some() {
        "Edit entry"
    } else {
        "New entry"
    });
    kind_row(f, ui, out);
    field(ui, out, "Title", &f.title, FormEdit::Title);
    field(ui, out, "Date (YYYY-MM-DD)", &f.date, FormEdit::Date);
    field(ui, out, "Time (HH:MM)", &f.time, FormEdit::Time);
    field(
        ui,
        out,
        "Duration (minutes)",
        &f.duration,
        FormEdit::Duration,
    );
    stack_picker(state, f.stack, ui, out);
    repeat_rows(f, ui, out);
    reminder_row(f, ui, out);
    field(ui, out, "Notes", &f.notes, FormEdit::Notes);
    if let Some(e) = &state.planner.form_error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    ui.horizontal(|ui| {
        if ui.button("Save entry").clicked() {
            out.push(Msg::Plan(PlanMsg::Save));
        }
        if ui.button("Cancel").clicked() {
            out.push(Msg::Plan(PlanMsg::CloseForm));
        }
    });
}

fn kind_row(f: &SessionForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        for kind in EVENT_KINDS {
            let name = event_kind_name(kind);
            if ui.selectable_label(f.kind == kind, name).clicked() {
                edit(out, FormEdit::Kind(kind));
            }
        }
    });
}

fn reminder_row(f: &SessionForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Remind me");
    ui.horizontal_wrapped(|ui| {
        for (minutes, label) in REMINDER_CHOICES {
            if ui
                .selectable_label(f.reminders.contains(&minutes), label)
                .clicked()
            {
                edit(out, FormEdit::Reminder(minutes));
            }
        }
    });
}

fn stack_picker(state: &State, current: Option<Uid>, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let stacks = state
        .vault
        .as_ref()
        .map(|v| v.overview.stacks.as_slice())
        .unwrap_or_default();
    let name = current
        .and_then(|u| stacks.iter().find(|s| s.uid == u))
        .map_or("No stack", |s| s.title.as_str());
    ComboBox::from_label("Stack")
        .selected_text(name)
        .show_ui(ui, |ui| {
            if ui.selectable_label(current.is_none(), "No stack").clicked() {
                edit(out, FormEdit::Stack(None));
            }
            for s in stacks {
                if ui
                    .selectable_label(current == Some(s.uid), &s.title)
                    .clicked()
                {
                    edit(out, FormEdit::Stack(Some(s.uid)));
                }
            }
        });
}

fn repeat_rows(f: &SessionForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        for (kind, label) in [
            (RepeatKind::Never, "Does not repeat"),
            (RepeatKind::Daily, "Daily"),
            (RepeatKind::Weekly, "Weekly"),
            (RepeatKind::Monthly, "Monthly"),
        ] {
            if ui.selectable_label(f.repeat == kind, label).clicked() {
                edit(out, FormEdit::Repeat(kind));
            }
        }
    });
    if f.repeat == RepeatKind::Never {
        return;
    }
    field(
        ui,
        out,
        "Repeat every (interval)",
        &f.every,
        FormEdit::Every,
    );
    if f.repeat == RepeatKind::Weekly {
        ui.horizontal_wrapped(|ui| {
            for (i, name) in DAY_NAMES.iter().enumerate() {
                let mut on = f.days & (1 << i) != 0;
                if ui.checkbox(&mut on, *name).changed() {
                    edit(out, FormEdit::Day(i as u8, on));
                }
            }
        });
    }
    ui.horizontal_wrapped(|ui| {
        for (kind, label) in [
            (EndsKind::Forever, "Never ends"),
            (EndsKind::On, "Ends on a date"),
            (EndsKind::After, "Ends after a count"),
        ] {
            if ui.selectable_label(f.ends == kind, label).clicked() {
                edit(out, FormEdit::Ends(kind));
            }
        }
    });
    if f.ends != EndsKind::Forever {
        field(
            ui,
            out,
            "End date or count",
            &f.ends_value,
            FormEdit::EndsValue,
        );
    }
}
