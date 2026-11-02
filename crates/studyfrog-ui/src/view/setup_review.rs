//! Step 3 of the run setup: the choices in words, each with a way back to change it.

use eframe::egui::{self, RichText};
use studyfrog_core::plan::{Order, ScheduleEffect, StateFilter};
use studyfrog_core::run::RunMode;

use crate::msg::Msg;
use crate::setup::{SetupForm, SetupStep};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn order_text(o: Order) -> &'static str {
    match o {
        Order::Due => "due items first",
        Order::Sequential => "in order",
        Order::HardestFirst => "hardest first",
        Order::EasiestFirst => "easiest first",
        Order::StalestFirst => "not seen for longest first",
        Order::LowestRetrievability => "most likely forgotten first",
        Order::LeastFamiliar => "least familiar first",
    }
}

fn state_text(s: StateFilter) -> &'static str {
    match s {
        StateFilter::Any => "any state",
        StateFilter::New => "new items only",
        StateFilter::Learned => "already learned items",
        StateFilter::Due => "items due now",
        StateFilter::NotDue => "items not due yet",
    }
}

fn what_line(state: &State, form: &SetupForm) -> String {
    let stacks = state.vault.as_ref().map_or(Vec::new(), |v| {
        v.overview
            .stacks
            .iter()
            .filter(|s| form.stacks.contains(&s.uid))
            .map(|s| s.title.clone())
            .collect()
    });
    let from = if stacks.is_empty() {
        "all stacks".to_owned()
    } else {
        stacks.join(", ")
    };
    let kinds = if form.kinds.is_empty() {
        "all kinds".to_owned()
    } else {
        format!("{} kind(s)", form.kinds.len())
    };
    format!("{from}; {kinds}; {}", state_text(form.state))
}

fn how_line(form: &SetupForm) -> String {
    let mode = if form.mode == RunMode::Flip {
        "flip cards"
    } else {
        "type answers"
    };
    let shuffle = if form.shuffle { ", shuffled" } else { "" };
    let limit = if form.limit.trim().is_empty() {
        "no limit".to_owned()
    } else {
        format!("at most {}", form.limit.trim())
    };
    format!("{mode}; {}{shuffle}; {limit}", order_text(form.order))
}

fn effect_line(form: &SetupForm) -> &'static str {
    match form.effect {
        ScheduleEffect::UpdateSchedule => "Your ratings update the review schedule.",
        ScheduleEffect::PracticeOnly => "Practice only: the review schedule does not change.",
    }
}

/// The summary card.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let form = &state.rehearse.form;
    let weak = palette_of(ui).text_weak;
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for (head, text, back) in [
            ("What", what_line(state, form), SetupStep::What),
            ("How", how_line(form), SetupStep::How),
            ("Schedule", effect_line(form).to_owned(), SetupStep::How),
        ] {
            ui.horizontal(|ui| {
                ui.label(RichText::new(head).strong());
                ui.label(RichText::new(text).color(weak));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(format!("Change {head}")).clicked() {
                        out.push(Msg::SetupStep(back));
                    }
                });
            });
        }
    });
}
