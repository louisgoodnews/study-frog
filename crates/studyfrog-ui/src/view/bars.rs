//! The status bar.

use eframe::egui::{self, Panel};

use crate::msg::Msg;
use crate::state::{NoticeKind, State};

/// The latest notice with a dismiss button, or the vault path when there is none.
pub fn status(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    Panel::bottom("status").show(ui, |ui| {
        ui.horizontal(|ui| match &state.notice {
            Some(n) => {
                let color = match n.kind {
                    NoticeKind::Info => ui.visuals().text_color(),
                    NoticeKind::Error => ui.visuals().error_fg_color,
                };
                ui.colored_label(color, &n.text);
                if ui.button("Dismiss").clicked() {
                    out.push(Msg::DismissNotice);
                }
            }
            None => {
                let text = state
                    .vault
                    .as_ref()
                    .map_or("No vault open", |v| v.root.as_str());
                ui.weak(text);
            }
        });
    });
}
