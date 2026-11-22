//! Timetable schedule planner for lectures and lessons (STORY-0141).

mod form;
use self::form::draw_form;
use eframe::egui::{self, RichText};
use studyfrog_calendar::timetable::{DAY_NAMES, TimetableEntry, lesson_label, lessons_on_weekday};
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
            if kit::primary_button(ui, "+ Add lecture / class")
                .on_hover_text("Adds a lecture or class to the timetable.")
                .clicked()
            {
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
                let clicked = ui
                    .button("Rehearse stack")
                    .on_hover_text("Starts a rehearsal of this stack.")
                    .clicked();
                if clicked {
                    out.push(Msg::Plan(PlanMsg::Rehearse(stack_id)));
                }
            }
            if ui
                .small_button("Edit")
                .on_hover_text("Opens this for editing.")
                .clicked()
            {
                out.push(Msg::Plan(PlanMsg::OpenTimetableForm(Some(entry.clone()))));
            }
            if ui
                .small_button("Delete")
                .on_hover_text("Deletes this timetable entry after you confirm.")
                .clicked()
            {
                out.push(Msg::Plan(PlanMsg::DeleteTimetable(entry.id)));
            }
        });
    });
    ui.add_space(4.0);
}

/// The weekly timetable, one heading per weekday that has lessons.
pub(super) fn timetable_panel(state: &State, ui: &mut egui::Ui) {
    ui.label(RichText::new("Timetable").strong());
    let entries = &state.planner.timetable;
    if entries.is_empty() {
        ui.label("No lectures or lessons yet. Add them in the Calendar's Timetable view.");
        return;
    }
    for (day, name) in DAY_NAMES.iter().enumerate() {
        let lessons = lessons_on_weekday(entries, day as u8);
        if lessons.is_empty() {
            continue;
        }
        ui.add_space(4.0);
        ui.label(RichText::new(*name).strong());
        for lesson in lessons {
            ui.label(lesson_label(lesson));
        }
    }
}
