//! Exam Scheduler form and workload preview view (EPIC-076, TASK-0309).
//!
//! Complies with Invariant P1: zero inherent impl blocks; all logic in free functions.

use eframe::egui::{self, RichText};
use studyfrog_calendar::exam_plan::ExamTarget;
use studyfrog_core::civil::format_date;

use crate::msg::Msg;
use crate::planner::PlanMsg;
use crate::planner::exam_sched::{ExamSchedForm, ExamSchedMsg};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::date_picker::{date_field, today_of};
use crate::view::widgets::text_field;
use crate::view::{home_hero, kit};

fn send(out: &mut Vec<Msg>, m: ExamSchedMsg) {
    out.push(Msg::Plan(PlanMsg::ExamSched(m)));
}

/// Draw the Exam Scheduler creation/editing panel with target picker, window mode, and live preview.
pub(super) fn draw_form(state: &State, f: &ExamSchedForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = palette_of(ui);
    home_hero::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(
            RichText::new(if f.editing.is_some() {
                "Edit exam schedule"
            } else {
                "Schedule an exam"
            })
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
        );
        ui.add_space(6.0);
        if let Some(t) = text_field(ui, "Exam title", &f.title, "e.g. Diagnostik Klausur", 1) {
            send(out, ExamSchedMsg::SetTitle(t));
        }
        if let Some(d) = date_field(ui, "Exam date (YYYY-MM-DD)", &f.exam_date, today_of(state)) {
            send(out, ExamSchedMsg::SetDate(d));
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Preparation window").strong());
        ui.horizontal_wrapped(|ui| {
            if ui
                .selectable_label(f.use_proposed, "StudyFrog proposed")
                .on_hover_text("Calculates weeks automatically from your vault stats.")
                .clicked()
            {
                send(out, ExamSchedMsg::SetUseProposed(true));
            }
            if ui
                .selectable_label(!f.use_proposed, "Custom weeks")
                .on_hover_text("Sets the preparation interval in weeks yourself.")
                .clicked()
            {
                send(out, ExamSchedMsg::SetUseProposed(false));
            }
        });
        if f.use_proposed {
            if let Some(prop) = &f.proposed {
                kit::pill(
                    ui,
                    &format!("Recommended: {} week(s)", prop.weeks),
                    p.accent,
                );
                for reason in &prop.reasons {
                    ui.label(RichText::new(format!("- {reason}")).color(p.text_weak));
                }
            } else {
                ui.weak("Attach stacks or items to calculate a recommended week window.");
            }
        } else if let Some(w) = text_field(ui, "Weeks before exam", &f.custom_weeks, "3", 1) {
            send(out, ExamSchedMsg::SetCustomWeeks(w));
        }

        ui.add_space(6.0);
        ui.label(RichText::new("Linked study material").strong());
        if let Some(vault) = &state.vault {
            ui.horizontal_wrapped(|ui| {
                for stack in &vault.overview.stacks {
                    let target = ExamTarget::Stack(stack.folder.clone());
                    let active = f.targets.contains(&target);
                    let count = stack.cards + stack.questions + stack.notes;
                    let label = format!("{} ({count})", stack.title);
                    if ui
                        .selectable_label(active, label)
                        .on_hover_text("Attaches or removes this stack from the exam schedule.")
                        .clicked()
                    {
                        send(out, ExamSchedMsg::ToggleStack(stack.folder.clone()));
                    }
                }
            });
        }

        if let Some(preview) = &f.preview {
            ui.add_space(6.0);
            ui.label(RichText::new("Schedule preview").strong());
            if preview.blocks.is_empty() {
                ui.weak("No study blocks generated yet.");
            } else {
                let total_slots: usize = preview.blocks.iter().map(|b| b.item_uids.len()).sum();
                ui.label(format!(
                    "{} study day(s), {} scheduled item slots",
                    preview.blocks.len(),
                    total_slots
                ));
                for block in preview.blocks.iter().take(5) {
                    ui.label(
                        RichText::new(format!(
                            "  {} - {} item(s) ({} new, {} review)",
                            format_date(block.date),
                            block.item_uids.len(),
                            block.new_count,
                            block.review_count
                        ))
                        .color(p.text_weak),
                    );
                }
            }
            for warn in &preview.warnings {
                ui.colored_label(p.warning, warn);
            }
        }

        if let Some(err) = &f.error {
            ui.add_space(4.0);
            ui.colored_label(p.danger, err);
        }

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if kit::primary_button(ui, "Apply schedule")
                .on_hover_text("Saves the exam and schedules its items across the window.")
                .clicked()
            {
                send(out, ExamSchedMsg::Apply);
            }
            if ui
                .button("Preview")
                .on_hover_text("Recalculates the recommended window and daily blocks.")
                .clicked()
            {
                send(out, ExamSchedMsg::RefreshPreview);
            }
            if ui
                .button("Cancel")
                .on_hover_text("Closes the exam scheduler without saving.")
                .clicked()
            {
                send(out, ExamSchedMsg::Close);
            }
        });
    });
}

/// Side-panel section in Calendar showing scheduled exam blocks for `selected` and active exam plans.
pub(super) fn side_section(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.planner;
    let pal = palette_of(ui);
    if ui
        .button("+ Schedule exam")
        .on_hover_text("Creates an exam and schedules its items across a preparation window.")
        .clicked()
    {
        send(out, ExamSchedMsg::OpenNew);
    }

    for exam in &p.exam_plans {
        let on_day = exam.blocks.iter().find(|b| b.date == p.selected);
        ui.add_space(6.0);
        home_hero::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                kit::pill(ui, "exam", pal.accent);
                ui.label(RichText::new(&exam.title).strong());
            });
            ui.label(
                RichText::new(format!(
                    "Exam: {} · {} week(s)",
                    format_date(exam.exam_date),
                    studyfrog_calendar::exam_plan::prep_window_weeks(&exam.window)
                ))
                .color(pal.text_weak),
            );
            if let Some(block) = on_day {
                ui.label(format!(
                    "Scheduled today: {} item(s) ({} new, {} review)",
                    block.item_uids.len(),
                    block.new_count,
                    block.review_count
                ));
                if kit::primary_button(ui, &format!("Rehearse {} items", exam.title))
                    .on_hover_text("Starts a rehearsal run for today's scheduled exam items.")
                    .clicked()
                {
                    send(out, ExamSchedMsg::RehearseBlock(block.item_uids.clone()));
                }
            }
            ui.horizontal_wrapped(|ui| {
                if ui
                    .small_button(format!("Edit {}", exam.title))
                    .on_hover_text("Edits this exam's targets and preparation window.")
                    .clicked()
                {
                    send(out, ExamSchedMsg::OpenEdit(exam.id));
                }
                if ui
                    .small_button(format!("Rebalance {}", exam.title))
                    .on_hover_text("Re-balances remaining exam study blocks from today.")
                    .clicked()
                {
                    send(out, ExamSchedMsg::Rebalance(exam.id));
                }
                if ui
                    .small_button(format!("Delete {}", exam.title))
                    .on_hover_text("Deletes this exam schedule.")
                    .clicked()
                {
                    send(out, ExamSchedMsg::Delete(exam.id));
                }
            });
        });
    }
}
