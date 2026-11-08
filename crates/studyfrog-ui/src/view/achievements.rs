//! The achievements screen: what is unlocked, and how far the rest is, grouped by what it measures or
//! by stack, with a filter for everything, the vault's own, or one stack.

use eframe::egui::{self, ProgressBar, RichText};
use studyfrog_md::rfc3339::format_rfc3339;
use studyfrog_service::achievements::Entry;

use crate::achievements::{AchievementsMsg, Filter, groups, stacks_of, visible};
use crate::msg::Msg;
use crate::state::State;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, filter: Filter) {
    out.push(Msg::Achievements(AchievementsMsg::Filter(filter)));
}

/// The filter: all, the vault's own, and one pill per stack that has achievements.
fn filter_pills(state: &State, entries: &[Entry], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let current = &state.achievements_filter;
    ui.horizontal_wrapped(|ui| {
        if ui
            .selectable_label(*current == Filter::All, "All")
            .clicked()
        {
            send(out, Filter::All);
        }
        if ui
            .selectable_label(*current == Filter::Global, "Global")
            .clicked()
        {
            send(out, Filter::Global);
        }
        for (key, title) in stacks_of(entries) {
            let here = *current == Filter::Stack(key.to_owned());
            if ui.selectable_label(here, title).clicked() {
                send(out, Filter::Stack(key.to_owned()));
            }
        }
    });
}

fn unlocked_row(e: &Entry, ui: &mut egui::Ui) {
    let when = e
        .unlocked
        .and_then(format_rfc3339)
        .map_or_else(String::new, |t| t[..10].to_owned());
    ui.label(RichText::new(format!("{} - unlocked {when}", e.title)).strong());
    ui.weak(&e.description);
}

fn open_row(e: &Entry, ui: &mut egui::Ui) {
    ui.label(RichText::new(&e.title).strong());
    ui.weak(&e.description);
    let share = e.value.min(e.threshold) * 1000 / e.threshold.max(1);
    let fraction = f32::from(u16::try_from(share).unwrap_or(1000)) / 1000.0;
    ui.add(ProgressBar::new(fraction).text(format!("{} / {}", e.value, e.threshold)));
}

fn group_card(label: &str, list: &[&Entry], ui: &mut egui::Ui) {
    let done = list.iter().filter(|e| e.unlocked.is_some()).count();
    kit::caption(ui, &format!("{label} ({done} / {} unlocked)", list.len()));
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        for (i, e) in list.iter().enumerate() {
            if i > 0 {
                ui.add_space(8.0);
            }
            if e.unlocked.is_some() {
                unlocked_row(e, ui);
            } else {
                open_row(e, ui);
            }
        }
    });
    ui.add_space(12.0);
}

/// The screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label(
        RichText::new("Achievements")
            .size(crate::typo::size(ui, crate::typo::Tier::Heading))
            .strong(),
    );
    ui.add_space(4.0);
    let Some(report) = &state.achievements else {
        ui.label("Checking achievements…");
        return;
    };
    if let Some(e) = &report.rules_error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    filter_pills(state, &report.entries, ui, out);
    ui.add_space(8.0);
    egui::ScrollArea::vertical().show(ui, |ui| {
        kit::column(ui, kit::READING_WIDTH, |ui| {
            let shown = visible(&report.entries, &state.achievements_filter);
            if shown.is_empty() {
                ui.weak("No achievements match this filter.");
            }
            for (label, list) in groups(&shown) {
                group_card(&label, &list, ui);
            }
        });
    });
}
