//! The achievements screen: what is unlocked, and how far the rest is.

use eframe::egui::{self, ProgressBar};
use studyfrog_md::rfc3339::format_rfc3339;
use studyfrog_service::achievements::Entry;

use crate::state::State;

fn name(e: &Entry) -> String {
    match &e.stack {
        Some(stack) => format!("{} ({stack})", e.title),
        None => e.title.clone(),
    }
}

/// Unlocked achievements first, then the ones in progress.
pub fn draw(state: &State, ui: &mut egui::Ui) {
    ui.heading("Achievements");
    let Some(report) = &state.achievements else {
        ui.label("Checking achievements…");
        return;
    };
    if let Some(e) = &report.rules_error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    egui::ScrollArea::vertical().show(ui, |ui| {
        let (done, open): (Vec<&Entry>, Vec<&Entry>) =
            report.entries.iter().partition(|e| e.unlocked.is_some());
        ui.strong(format!("Unlocked ({})", done.len()));
        if done.is_empty() {
            ui.label("Nothing unlocked yet.");
        }
        for e in done {
            let when = e
                .unlocked
                .and_then(format_rfc3339)
                .map_or_else(String::new, |t| t[..10].to_owned());
            ui.label(format!("{} - unlocked {when}", name(e)));
            ui.weak(&e.description);
        }
        ui.add_space(8.0);
        ui.strong(format!("In progress ({})", open.len()));
        for e in open {
            ui.label(name(e));
            ui.weak(&e.description);
            let fraction = f32::from(
                u16::try_from(e.value.min(e.threshold) * 1000 / e.threshold.max(1)).unwrap_or(1000),
            ) / 1000.0;
            ui.add(ProgressBar::new(fraction).text(format!("{} / {}", e.value, e.threshold)));
        }
    });
}
