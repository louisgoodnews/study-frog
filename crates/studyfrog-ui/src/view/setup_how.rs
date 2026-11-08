//! Step 2 of the run setup: preset, mode, order, size and what happens to the schedule. The rarer
//! filters sit under "More options".

use eframe::egui;
use studyfrog_core::plan::{Order, ScheduleEffect, StateFilter};
use studyfrog_core::run::RunMode;

use crate::msg::Msg;
use crate::setup::{Preset, SetupEdit, SetupForm};
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
    (Order::HardestFirst, "Hardest first"),
    (Order::EasiestFirst, "Easiest first"),
    (Order::StalestFirst, "Not seen for longest"),
    (Order::LowestRetrievability, "Most likely forgotten"),
    (Order::LeastFamiliar, "Least familiar first"),
];

const STATES: [(StateFilter, &str); 5] = [
    (StateFilter::Any, "Any"),
    (StateFilter::New, "New only"),
    (StateFilter::Learned, "Already learned"),
    (StateFilter::Due, "Due now"),
    (StateFilter::NotDue, "Not due yet"),
];

/// What an exam is, as the setup states it.
pub const EXAM_RULES: &str = "Exam: every card and question once, answered in writing. You cannot skip, go \
     back or see an answer; a submitted answer is locked. Your answers are compared with the references and \
     graded at the end (60 % to pass, A+ / 1+ from 95 %). Notes are left out.";

fn edit(out: &mut Vec<Msg>, e: SetupEdit) {
    out.push(Msg::Setup(e));
}

/// The step.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let form = &state.rehearse.form;
    ui.horizontal_wrapped(|ui| {
        for (preset, label) in PRESETS {
            if ui.button(label).clicked() {
                edit(out, SetupEdit::Preset(preset));
            }
        }
    });
    ui.add_space(6.0);
    crate::view::setup_pacing::draw(form, ui, out);
    ui.add_space(6.0);
    ui.label("How");
    ui.horizontal(|ui| {
        for (mode, label) in [
            (RunMode::Flip, "Flip cards"),
            (RunMode::Answer, "Type answers"),
            (RunMode::Exam, "Exam"),
        ] {
            if ui.selectable_label(form.mode == mode, label).clicked() {
                edit(out, SetupEdit::Mode(mode));
            }
        }
    });
    if form.mode == RunMode::Exam {
        ui.weak(EXAM_RULES);
    }
    ui.label("Order");
    ui.horizontal_wrapped(|ui| {
        for (order, label) in ORDERS {
            if ui.selectable_label(form.order == order, label).clicked() {
                edit(out, SetupEdit::Order(order));
            }
        }
    });
    let mut shuffle = form.shuffle;
    if ui
        .checkbox(&mut shuffle, "Shuffle the order of the chosen items")
        .changed()
    {
        edit(out, SetupEdit::Shuffle(shuffle));
    }
    if let Some(text) = text_field(ui, "Maximum number of items", &form.limit, "no limit", 1) {
        edit(out, SetupEdit::Limit(text));
    }
    schedule(form, ui, out);
    ui.collapsing("More options", |ui| more(form, ui, out));
}

fn schedule(form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if form.mode == RunMode::Exam {
        ui.weak("An exam never changes your review schedule.");
        return;
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

fn more(form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
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
    let mut fam = form.fam_on;
    if ui
        .checkbox(&mut fam, "Only items I am less familiar with than a level")
        .changed()
    {
        edit(out, SetupEdit::FamOn(fam));
    }
    if form.fam_on
        && let Some(text) = text_field(ui, "Less familiar than (percent)", &form.fam_below, "", 1)
    {
        edit(out, SetupEdit::FamBelow(text));
    }
}
