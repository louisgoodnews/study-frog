//! The status bar.

use eframe::egui::{self, Panel};

use crate::msg::Msg;
use crate::state::{NoticeKind, State};

/// The latest notice with a dismiss button, or the size of the vault when there is none.
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
                    .map_or("No vault open".to_owned(), |v| {
                        let items: usize = v
                            .overview
                            .stacks
                            .iter()
                            .map(|s| s.notes + s.cards + s.questions)
                            .sum();
                        format!(
                            "{} stacks · {} items",
                            v.overview.stacks.len(),
                            items + v.overview.inbox_items
                        )
                    });
                ui.weak(text);
            }
        });
    });
}
