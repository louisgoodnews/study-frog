//! The achievements screen: what is unlocked, and how far the rest is.

use eframe::egui::{self, ProgressBar, RichText};
use studyfrog_md::rfc3339::format_rfc3339;
use studyfrog_service::achievements::Entry;

use crate::state::State;
use crate::view::kit;

fn name(e: &Entry) -> String {
    match &e.stack {
        Some(stack) => format!("{} ({stack})", e.title),
        None => e.title.clone(),
    }
}

/// Unlocked achievements first, then the ones in progress.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    ui.label(RichText::new("Achievements").size(17.0).strong());
    ui.add_space(4.0);
    let Some(report) = &state.achievements else {
        ui.label("Checking achievements…");
        return;
    };
    if let Some(e) = &report.rules_error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        kit::column(ui, kit::READING_WIDTH, |ui| {
            let (done, open): (Vec<&Entry>, Vec<&Entry>) =
                report.entries.iter().partition(|e| e.unlocked.is_some());
            kit::caption(ui, &format!("Unlocked ({})", done.len()));
            kit::card(ui, |ui| {
                ui.set_width(ui.available_width());
                if done.is_empty() {
                    ui.weak("Nothing unlocked yet.");
                }
                for (i, e) in done.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(8.0);
                    }
                    let when = e
                        .unlocked
                        .and_then(format_rfc3339)
                        .map_or_else(String::new, |t| t[..10].to_owned());
                    ui.label(RichText::new(format!("{} - unlocked {when}", name(e))).strong());
                    ui.weak(&e.description);
                }
            });
            ui.add_space(16.0);
            kit::caption(ui, &format!("In progress ({})", open.len()));
            kit::card(ui, |ui| {
                ui.set_width(ui.available_width());
                if open.is_empty() {
                    ui.weak("No achievements in progress.");
                }
                for (i, e) in open.iter().enumerate() {
                    if i > 0 {
                        ui.add_space(8.0);
                    }
                    ui.label(RichText::new(name(e)).strong());
                    ui.weak(&e.description);
                    let fraction = f32::from(
                        u16::try_from(e.value.min(e.threshold) * 1000 / e.threshold.max(1))
                            .unwrap_or(1000),
                    ) / 1000.0;
                    ui.add(
                        ProgressBar::new(fraction).text(format!("{} / {}", e.value, e.threshold)),
                    );
                }
            });
        });
    });
}
