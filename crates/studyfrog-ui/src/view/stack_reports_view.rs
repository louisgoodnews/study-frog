//! The Reports tab of a stack: its familiarity and the items it knows least, from the same report as the global screen.

use crate::msg::Msg;
use crate::state::{Screen, State};
use eframe::egui;

/// The tab.
pub fn draw(state: &State, folder: &str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(b) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    let Some(g) = b.familiarity.stacks.iter().find(|g| g.folder == folder) else {
        ui.weak("No report for this stack yet.");
        return;
    };
    ui.heading("Familiarity");
    let value = g
        .percent
        .map_or("no history yet".to_owned(), |p| format!("{p} %"));
    ui.label(format!("{}: {value} ({} item(s))", g.name, g.items));
    if g.least.is_empty() {
        ui.weak("No item has a history yet.");
    }
    ui.label("The items this stack knows least:");
    for row in &g.least {
        ui.horizontal_wrapped(|ui| {
            if ui.link(&row.title).clicked() {
                out.push(Msg::OpenItem(row.uid));
            }
            ui.weak(format!("{} % - {}", row.percent, row.drivers));
        });
    }
    ui.add_space(8.0);
    if ui.button("Open all reports").clicked() {
        out.push(Msg::Navigate(Screen::Reports));
    }
}
