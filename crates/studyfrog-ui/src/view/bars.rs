//! The status bar.

use eframe::egui::{self, Align, Layout, Panel};

use crate::help_types::HelpMsg;
use crate::msg::Msg;
use crate::state::{NoticeKind, State};

/// The latest notice with a dismiss button, or the size of the vault when there is none; the help
/// button for the screen at the right end.
pub fn status(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    Panel::bottom("status").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button("Help")
                    .on_hover_text("Help for this screen (F1)")
                    .clicked()
                {
                    out.push(Msg::Help(HelpMsg::ForScreen));
                }
                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    message(state, ui, out)
                });
            });
        });
    });
}

fn message(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    match &state.notice {
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
    }
}
