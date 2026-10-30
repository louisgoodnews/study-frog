//! The run setup screen: presets, order, filters, effect on the schedule, and a live item count.

use eframe::egui;
use studyfrog_core::plan::{ItemKind, Order, ScheduleEffect, StateFilter};
use studyfrog_core::run::RunMode;

use crate::msg::Msg;
use crate::setup::{Preset, Preview, SetupEdit, SetupForm};
use crate::state::State;
use crate::view::widgets::text_field;

const PRESETS: [(Preset, &str); 5] = [
    (Preset::Spaced, "Spaced repetition"),
    (Preset::EverythingInOrder, "Everything, in order"),
    (Preset::Shuffled, "Everything, shuffled"),
    (Preset::Hardest, "Hardest first"),
    (Preset::Stalest, "Not seen for longest"),
];

const ORDERS: [(Order, &str); 7] = [
    (Order::Due, "Due first"),
    (Order::Sequential, "In order"),
    (Order::Shuffled, "Shuffled"),
    (Order::HardestFirst, "Hardest first"),
    (Order::EasiestFirst, "Easiest first"),
    (Order::StalestFirst, "Not seen for longest"),
    (Order::LowestRetrievability, "Most likely forgotten"),
];

const STATES: [(StateFilter, &str); 5] = [
    (StateFilter::Any, "Any"),
    (StateFilter::New, "New only"),
    (StateFilter::Learned, "Already learned"),
    (StateFilter::Due, "Due now"),
    (StateFilter::NotDue, "Not due yet"),
];

const KINDS: [(ItemKind, &str); 3] = [
    (ItemKind::Note, "Notes"),
    (ItemKind::Card, "Cards"),
    (ItemKind::Question, "Questions"),
];

fn edit(out: &mut Vec<Msg>, e: SetupEdit) {
    out.push(Msg::Setup(e));
}

/// The whole screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let (form, preview) = (&state.rehearse.form, &state.rehearse.preview);
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Rehearse");
        ui.label("Start from a preset, then adjust anything below.");
        ui.horizontal_wrapped(|ui| {
            for (preset, label) in PRESETS {
                if ui.button(label).clicked() {
                    edit(out, SetupEdit::Preset(preset));
                }
            }
        });
        ui.separator();
        how(form, ui, out);
        ui.separator();
        filters(state, form, ui, out);
        ui.separator();
        schedule(form, ui, out);
        ui.separator();
        preview_line(preview, ui);
    });
}

fn how(form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("How");
    ui.horizontal(|ui| {
        for (mode, label) in [
            (RunMode::Flip, "Flip cards"),
            (RunMode::Answer, "Type answers"),
        ] {
            if ui.selectable_label(form.mode == mode, label).clicked() {
                edit(out, SetupEdit::Mode(mode));
            }
        }
    });
    ui.label("Order");
    ui.horizontal_wrapped(|ui| {
        for (order, label) in ORDERS {
            if ui.selectable_label(form.order == order, label).clicked() {
                edit(out, SetupEdit::Order(order));
            }
        }
    });
    if let Some(text) = text_field(ui, "Maximum number of items", &form.limit, "no limit", 1) {
        edit(out, SetupEdit::Limit(text));
    }
}

fn filters(state: &State, form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("Which items");
    ui.horizontal(|ui| {
        for (kind, label) in KINDS {
            let mut on = form.kinds.contains(&kind);
            if ui.checkbox(&mut on, label).changed() {
                edit(out, SetupEdit::Kind(kind, on));
            }
        }
    });
    ui.weak("Nothing ticked means all kinds.");
    if let Some(vault) = &state.vault {
        ui.horizontal_wrapped(|ui| {
            for s in &vault.overview.stacks {
                let mut on = form.stacks.contains(&s.uid);
                if ui
                    .checkbox(&mut on, format!("Stack: {}", s.title))
                    .changed()
                {
                    edit(out, SetupEdit::Stack(s.uid, on));
                }
            }
        });
    }
    if let Some(text) = text_field(
        ui,
        "Tags (any of, comma separated)",
        &form.tags,
        "e.g. grammar, verbs",
        1,
    ) {
        edit(out, SetupEdit::Tags(text));
    }
    ui.horizontal_wrapped(|ui| {
        for (s, label) in STATES {
            if ui.selectable_label(form.state == s, label).clicked() {
                edit(out, SetupEdit::State(s));
            }
        }
    });
    let mut on = form.difficulty_on;
    if ui
        .checkbox(&mut on, "Only items within a difficulty range")
        .changed()
    {
        edit(out, SetupEdit::DifficultyOn(on));
    }
    if form.difficulty_on {
        if let Some(text) = text_field(ui, "Lowest difficulty (1-10)", &form.diff_min, "", 1) {
            edit(out, SetupEdit::DiffMin(text));
        }
        if let Some(text) = text_field(ui, "Highest difficulty (1-10)", &form.diff_max, "", 1) {
            edit(out, SetupEdit::DiffMax(text));
        }
    }
}

fn schedule(form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.strong("Afterwards");
    let mut requeue = form.requeue_again;
    if ui
        .checkbox(
            &mut requeue,
            "Show items I rate Again once more in this run",
        )
        .changed()
    {
        edit(out, SetupEdit::Requeue(requeue));
    }
    let mut update = form.effect == ScheduleEffect::UpdateSchedule;
    if ui
        .checkbox(&mut update, "Update my review schedule with these ratings")
        .changed()
    {
        let effect = if update {
            ScheduleEffect::UpdateSchedule
        } else {
            ScheduleEffect::PracticeOnly
        };
        edit(out, SetupEdit::Effect(effect));
    }
    if !update {
        ui.weak("Practice only: your ratings and timings are recorded, but the schedule does not change.");
    }
}

fn preview_line(preview: &Preview, ui: &mut egui::Ui) {
    match preview {
        Preview::Unknown | Preview::Counting => {
            ui.label("Counting matching items…");
        }
        Preview::Count(n) => {
            ui.strong(format!("{n} items match"));
        }
        Preview::Invalid(reason) => {
            ui.colored_label(ui.visuals().error_fg_color, reason);
        }
        Preview::Failed(reason) => {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Could not count: {reason}"),
            );
        }
    }
}
