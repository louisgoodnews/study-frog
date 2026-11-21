//! The timetable entry form.
use super::*;

pub(super) fn draw_form(
    state: &State,
    form: &TimetableForm,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    let mut draft = form.clone();

    ui.heading(if form.id.is_some() {
        "Edit Timetable Entry"
    } else {
        "New Lecture / Class"
    });
    ui.add_space(8.0);

    if let Some(err) = &state.planner.timetable_error {
        ui.label(RichText::new(err).color(pal.danger));
        ui.add_space(4.0);
    }

    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Course Title:");
            ui.text_edit_singleline(&mut draft.title);
        });

        ui.horizontal(|ui| {
            ui.label("Day:");
            egui::ComboBox::from_id_salt("tt-day")
                .selected_text(weekday_name(draft.day_of_week))
                .show_ui(ui, |ui| {
                    for d in 0..7 {
                        ui.selectable_value(&mut draft.day_of_week, d, weekday_name(d));
                    }
                });
        });

        ui.horizontal(|ui| {
            ui.label("Start time:");
            ui.add(egui::DragValue::new(&mut draft.start_hour).range(0..=23));
            ui.label(":");
            ui.add(egui::DragValue::new(&mut draft.start_minute).range(0..=59));
            ui.label("Duration (min):");
            ui.add(egui::DragValue::new(&mut draft.duration_min).range(15..=480));
        });

        ui.horizontal(|ui| {
            ui.label("Room / Location:");
            ui.text_edit_singleline(&mut draft.location);
        });

        ui.horizontal(|ui| {
            ui.label("Instructor:");
            ui.text_edit_singleline(&mut draft.instructor);
        });

        ui.horizontal(|ui| {
            ui.label("Linked Stack:");
            let cur_title =
                stack_title(state, draft.stack_uid).unwrap_or_else(|| "None".to_owned());
            egui::ComboBox::from_id_salt("tt-stack")
                .selected_text(cur_title)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut draft.stack_uid, None, "None");
                    if let Some(v) = &state.vault {
                        for s in &v.overview.stacks {
                            ui.selectable_value(&mut draft.stack_uid, Some(s.uid), &s.title);
                        }
                    }
                });
        });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Save")
                .on_hover_text("Saves this timetable entry.")
                .clicked()
            {
                let res = crate::timetable_form::timetable_from_form(&draft);
                if let Ok(entry) = res {
                    out.push(Msg::Plan(PlanMsg::SaveTimetable(entry)));
                }
            }
            if ui
                .button("Cancel")
                .on_hover_text("Closes this without making the change.")
                .clicked()
            {
                out.push(Msg::Plan(PlanMsg::CloseTimetableForm));
            }
        });
    });
}
