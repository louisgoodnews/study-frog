//! The pacing choice of the run setup: no clock, a countdown with presets and a custom time, or a count up.

use eframe::egui;

use crate::msg::Msg;
use crate::setup::{SetupEdit, SetupForm};
use crate::setup_pacing::{PRESET_MINUTES, PacingEdit, PacingKind};
use crate::view::widgets::text_field;

fn edit(out: &mut Vec<Msg>, e: PacingEdit) {
    out.push(Msg::Setup(SetupEdit::Pacing(e)));
}

/// The section.
pub fn draw(form: &SetupForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Time");
    ui.horizontal_wrapped(|ui| {
        for (kind, label) in [
            (PacingKind::Free, "No clock"),
            (PacingKind::Countdown, "Countdown"),
            (PacingKind::CountUp, "Count up"),
        ] {
            if ui
                .selectable_label(form.pacing.kind == kind, label)
                .clicked()
            {
                edit(out, PacingEdit::Kind(kind));
            }
        }
    });
    match form.pacing.kind {
        PacingKind::Free => {}
        PacingKind::CountUp => {
            ui.weak(
                "The time used is shown. The run goes on until the last item or until you stop it.",
            );
        }
        PacingKind::Countdown => {
            ui.weak("The run ends by itself when the time is up; the item on screen then counts as not answered.");
            ui.horizontal_wrapped(|ui| {
                for m in PRESET_MINUTES {
                    let on = form.pacing.minutes.trim() == m.to_string();
                    if ui.selectable_label(on, format!("{m} minutes")).clicked() {
                        edit(out, PacingEdit::Minutes(m.to_string()));
                    }
                }
            });
            if let Some(text) =
                text_field(ui, "Countdown minutes (1-180)", &form.pacing.minutes, "", 1)
            {
                edit(out, PacingEdit::Minutes(text));
            }
        }
    }
}
