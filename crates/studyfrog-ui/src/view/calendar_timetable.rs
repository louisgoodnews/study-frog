//! Timetable schedule planner for lectures and lessons (STORY-0141).

use eframe::egui::{self, RichText};
use studyfrog_calendar::timetable::TimetableEntry;
use studyfrog_core::civil::format_minute;
use studyfrog_core::uid::Uid;

use crate::msg::Msg;
use crate::plan_msg::PlanMsg;
use crate::state::State;
use crate::theme::palette_of;
use crate::timetable_form::TimetableForm;
use crate::view::kit;

fn weekday_name(idx: u8) -> &'static str {
    match idx {
        0 => "Monday",
        1 => "Tuesday",
        2 => "Wednesday",
        3 => "Thursday",
        4 => "Friday",
        5 => "Saturday",
        _ => "Sunday",
    }
}

fn stack_title(state: &State, uid: Option<Uid>) -> Option<String> {
    let uid = uid?;
    let v = state.vault.as_ref()?;
    v.overview
        .stacks
        .iter()
        .find(|s| s.uid == uid)
        .map(|s| s.title.clone())
}

/// Draw the weekly academic timetable.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;

    if let Some(form) = &p.timetable_form {
        draw_form(state, form, ui, out);
        return;
    }

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Weekly Academic Timetable")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if kit::primary_button(ui, "+ Add lecture / class").clicked() {
                out.push(Msg::Plan(PlanMsg::OpenTimetableForm(None)));
            }
        });
    });

    ui.add_space(8.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for day in 0..7 {
                let day_entries: Vec<&TimetableEntry> = p
                    .timetable
                    .iter()
                    .filter(|e| e.day_of_week == day)
                    .collect();

                if day >= 5 && day_entries.is_empty() {
                    continue; // Skip empty weekends
                }

                draw_day_group(state, day, &day_entries, ui, out);
            }
        });
}

fn draw_day_group(
    state: &State,
    day: u8,
    entries: &[&TimetableEntry],
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let pal = palette_of(ui);
    kit::section(ui, weekday_name(day));
    if entries.is_empty() {
        ui.label(RichText::new("No scheduled lectures or lessons").color(pal.text_weak));
        ui.add_space(6.0);
        return;
    }
    for &entry in entries {
        draw_entry_card(state, entry, ui, out);
    }
    ui.add_space(6.0);
}

fn draw_entry_card(state: &State, entry: &TimetableEntry, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let pal = palette_of(ui);
    let end_min = entry.start_minute + entry.duration_min;
    let time_span = format!(
        "{}–{}",
        format_minute(entry.start_minute),
        format_minute(end_min % 1440)
    );
    let stack_name = stack_title(state, entry.stack_uid);

    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            kit::pill(ui, "Class", pal.accent);
            ui.label(RichText::new(&entry.title).strong());
            ui.label(RichText::new(format!("{time_span} ({} min)", entry.duration_min)).weak());
        });

        let mut meta = Vec::new();
        if let Some(loc) = &entry.location {
            meta.push(format!("Room: {loc}"));
        }
        if let Some(inst) = &entry.instructor {
            meta.push(format!("Instructor: {inst}"));
        }
        if let Some(st) = &stack_name {
            meta.push(format!("Stack: {st}"));
        }
        if !meta.is_empty() {
            ui.label(RichText::new(meta.join("  ·  ")).color(pal.text_weak));
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if let Some(stack_id) = entry.stack_uid {
                let clicked = ui.button("Rehearse stack").clicked();
                if clicked {
                    out.push(Msg::Plan(PlanMsg::Rehearse(stack_id)));
                }
            }
            if ui.small_button("Edit").clicked() {
                out.push(Msg::Plan(PlanMsg::OpenTimetableForm(Some(entry.clone()))));
            }
            if ui.small_button("Delete").clicked() {
                out.push(Msg::Plan(PlanMsg::DeleteTimetable(entry.id)));
            }
        });
    });
    ui.add_space(4.0);
}

fn draw_form(state: &State, form: &TimetableForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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
            if kit::primary_button(ui, "Save").clicked() {
                let res = crate::timetable_form::timetable_from_form(&draft);
                if let Ok(entry) = res {
                    out.push(Msg::Plan(PlanMsg::SaveTimetable(entry)));
                }
            }
            if ui.button("Cancel").clicked() {
                out.push(Msg::Plan(PlanMsg::CloseTimetableForm));
            }
        });
    });
}
