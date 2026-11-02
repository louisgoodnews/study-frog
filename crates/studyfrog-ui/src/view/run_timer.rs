//! The clock on the rehearsal screen: time left or time used, and the warnings at one minute and ten seconds, in text and
//! announced to screen readers. Colour is never the only cue.

use eframe::egui::{self, RichText};

use crate::run_clock::{Clock, Warning, warning_text};
use crate::theme::palette_of;
use crate::view::kit;

fn announce(ui: &mut egui::Ui, w: Warning) {
    let color = palette_of(ui).danger;
    let label = ui.label(RichText::new(warning_text(w)).strong().color(color));
    ui.ctx().accesskit_node_builder(label.id, |node| {
        node.set_live(egui::accesskit::Live::Assertive)
    });
}

/// The clock and, near the end of a countdown, its warning.
pub fn draw(clock: &Clock, ui: &mut egui::Ui) {
    let weak = palette_of(ui).text_weak;
    kit::pill(ui, &clock.text, weak);
    if let Some(w) = clock.warning {
        announce(ui, w);
    }
}
