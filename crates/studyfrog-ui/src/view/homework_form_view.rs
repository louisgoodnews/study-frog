//! The create and edit dialog of the homework screen (STORY-0167).

use eframe::egui::{self, RichText};
use studyfrog_calendar::homework::HomeworkPriority;

use crate::homework_form::{HomeworkEdit, HomeworkForm};
use crate::homework_msg::HomeworkMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit;

fn edit(out: &mut Vec<Msg>, e: HomeworkEdit) {
    out.push(Msg::Homework(HomeworkMsg::Edit(e)));
}

fn priority_name(p: HomeworkPriority) -> &'static str {
    match p {
        HomeworkPriority::Low => "Low",
        HomeworkPriority::Medium => "Medium",
        HomeworkPriority::High => "High",
    }
}

/// Draw the dialog for `form`.
pub fn draw(state: &State, form: &HomeworkForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        let heading = if form.existing.is_some() {
            "Edit homework"
        } else {
            "New homework"
        };
        ui.label(RichText::new(heading).strong());
        ui.horizontal(|ui| {
            ui.label("Title");
            let mut title = form.title.clone();
            if ui
                .add(egui::TextEdit::singleline(&mut title).desired_width(320.0))
                .changed()
            {
                edit(out, HomeworkEdit::Title(title));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Course");
            let current = state
                .vault
                .as_ref()
                .and_then(|v| {
                    v.overview
                        .stacks
                        .iter()
                        .find(|s| Some(s.uid) == form.course)
                })
                .map_or("No course".to_owned(), |s| s.title.clone());
            egui::ComboBox::from_id_salt("homework-course")
                .selected_text(current)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_label(form.course.is_none(), "No course")
                        .clicked()
                    {
                        edit(out, HomeworkEdit::Course(None));
                    }
                    let stacks = state.vault.as_ref().map(|v| v.overview.stacks.as_slice());
                    for stack in stacks.unwrap_or_default() {
                        let picked = form.course == Some(stack.uid);
                        if ui.selectable_label(picked, &stack.title).clicked() {
                            edit(out, HomeworkEdit::Course(Some(stack.uid)));
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            ui.label("Due");
            let mut date = form.due_date.clone();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut date)
                        .hint_text("YYYY-MM-DD")
                        .desired_width(110.0),
                )
                .changed()
            {
                edit(out, HomeworkEdit::DueDate(date));
            }
            let mut time = form.due_time.clone();
            if ui
                .add(
                    egui::TextEdit::singleline(&mut time)
                        .hint_text("HH:MM")
                        .desired_width(64.0),
                )
                .changed()
            {
                edit(out, HomeworkEdit::DueTime(time));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Estimate");
            let mut minutes = form.estimate_min;
            let slider = egui::Slider::new(&mut minutes, 5..=600).suffix(" min");
            if ui.add(slider).changed() {
                edit(out, HomeworkEdit::Estimate(minutes));
            }
        });
        ui.horizontal(|ui| {
            ui.label("Priority");
            for p in [
                HomeworkPriority::Low,
                HomeworkPriority::Medium,
                HomeworkPriority::High,
            ] {
                if ui
                    .selectable_label(form.priority == p, priority_name(p))
                    .clicked()
                {
                    edit(out, HomeworkEdit::Priority(p));
                }
            }
        });
        ui.label("Checklist (one step per line)");
        let mut steps = form.checklist.clone();
        if ui
            .add(
                egui::TextEdit::multiline(&mut steps)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            edit(out, HomeworkEdit::Checklist(steps));
        }
        ui.label("Notes");
        let mut notes = form.notes.clone();
        if ui
            .add(
                egui::TextEdit::multiline(&mut notes)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            )
            .changed()
        {
            edit(out, HomeworkEdit::Notes(notes));
        }
        if let Some(e) = &state.planner.homework_ui.error {
            ui.label(RichText::new(e).italics());
        }
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Save").clicked() {
                out.push(Msg::Homework(HomeworkMsg::Save));
            }
            if kit::quiet_button(ui, "Cancel").clicked() {
                out.push(Msg::Homework(HomeworkMsg::Close));
            }
        });
    });
}
