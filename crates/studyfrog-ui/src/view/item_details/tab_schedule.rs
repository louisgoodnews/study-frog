//! The Schedule tab of the item details notebook: view, set, modify, and clear a stack's
//! due date and reminder notifications directly within the details view.

use eframe::egui;
use studyfrog_core::civil::{add_days, days_between, format_date, parse_date};

use crate::deadline_ui::deadline_of;
use crate::editor::Editor;
use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::session_form::REMINDER_CHOICES;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::date_picker::{date_field, today_of};
use crate::view::kit;

/// Draw the Schedule tab for a stack.
pub fn draw(state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    super::item_card(e, ui, |ui| {
        ui.set_width(ui.available_width());
        let dl = deadline_of(&state.planner.sessions, e.uid);
        let today = today_of(state);

        status_section(ui, dl, today);
        ui.add_space(8.0);
        edit_section(state, e, ui, dl, today, out);
        if let Some(session) = dl {
            ui.add_space(8.0);
            reminders_section(e, ui, session, out);
        }
    });
}

fn status_section(
    ui: &mut egui::Ui,
    dl: Option<&studyfrog_calendar::session::Session>,
    today: studyfrog_core::civil::Date,
) {
    match dl {
        Some(session) => {
            ui.horizontal(|ui| {
                ui.label("Current due date:");
                ui.strong(format_date(session.start.date));
            });
            let diff = days_between(today, session.start.date);
            let text = match diff {
                0 => "Due today".to_owned(),
                1 => "Due in 1 day".to_owned(),
                n if n > 1 => format!("Due in {n} days"),
                -1 => "Overdue by 1 day".to_owned(),
                n => format!("Overdue by {} days", -n),
            };
            let color = if diff < 0 {
                palette_of(ui).warning
            } else {
                palette_of(ui).accent
            };
            kit::pill(ui, &text, color);
        }
        None => {
            ui.weak("No due date set");
        }
    }
}

fn edit_section(
    state: &State,
    e: &Editor,
    ui: &mut egui::Ui,
    dl: Option<&studyfrog_calendar::session::Session>,
    today: studyfrog_core::civil::Date,
    out: &mut Vec<Msg>,
) {
    let cur_str = dl.map_or(String::new(), |s| format_date(s.start.date));
    if let Some(new_text) = date_field(ui, "Due date", &cur_str, today_of(state)) {
        let trimmed = new_text.trim();
        if trimmed.is_empty() {
            if dl.is_some() {
                out.push(Msg::Plan(PlanMsg::SetStackDueDate {
                    stack: e.uid,
                    date: None,
                    reminders: Vec::new(),
                }));
            }
        } else if let Some(date) = parse_date(trimmed) {
            let reminders = dl.map_or_else(|| vec![1440], |s| s.reminders.clone());
            out.push(Msg::Plan(PlanMsg::SetStackDueDate {
                stack: e.uid,
                date: Some(date),
                reminders,
            }));
        }
    }

    ui.horizontal(|ui| {
        if dl.is_none() && ui.button("Set due in 1 week").clicked() {
            out.push(Msg::Plan(PlanMsg::SetStackDueDate {
                stack: e.uid,
                date: Some(add_days(today, 7)),
                reminders: vec![1440],
            }));
        }
        if dl.is_some() && ui.button("Clear due date").clicked() {
            out.push(Msg::Plan(PlanMsg::SetStackDueDate {
                stack: e.uid,
                date: None,
                reminders: Vec::new(),
            }));
        }
    });
}

fn reminders_section(
    e: &Editor,
    ui: &mut egui::Ui,
    session: &studyfrog_calendar::session::Session,
    out: &mut Vec<Msg>,
) {
    ui.label("Remind me:");
    ui.horizontal_wrapped(|ui| {
        for (mins, label) in REMINDER_CHOICES {
            let active = session.reminders.contains(&mins);
            if ui.selectable_label(active, label).clicked() {
                let mut reminders = session.reminders.clone();
                if active {
                    reminders.retain(|m| *m != mins);
                } else {
                    reminders.push(mins);
                }
                out.push(Msg::Plan(PlanMsg::SetStackDueDate {
                    stack: e.uid,
                    date: Some(session.start.date),
                    reminders,
                }));
            }
        }
    });
}
