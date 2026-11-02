//! The familiarity report: the spread of values, and per stack and per subject the roll-up with its least familiar items.

use eframe::egui::{self, ProgressBar};
use studyfrog_service::fam_report::{FamGroup, FamReport};

use crate::msg::Msg;
use crate::state::State;

fn spread(r: &FamReport, ui: &mut egui::Ui) {
    let top = r.spread.iter().copied().max().unwrap_or(0).max(1) as f32;
    ui.heading("How well you know your items");
    for (i, n) in r.spread.iter().enumerate() {
        let label = format!("{}-{} %", i * 10, if i == 9 { 100 } else { i * 10 + 9 });
        ui.horizontal(|ui| {
            ui.label(&label);
            let bar = ProgressBar::new(*n as f32 / top)
                .desired_width(240.0)
                .text(format!("{n} item(s)"));
            ui.add(bar);
        });
    }
    if r.unscored > 0 {
        ui.weak(format!("{} item(s) have no history yet.", r.unscored));
    }
}

fn groups(title: &str, list: &[FamGroup], ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(8.0);
    ui.heading(title);
    for g in list.iter().filter(|g| g.items > 0) {
        let value = g
            .percent
            .map_or("no history".to_owned(), |p| format!("{p} %"));
        ui.label(
            egui::RichText::new(format!("{}: {value} ({} item(s))", g.name, g.items)).strong(),
        );
        for row in &g.least {
            ui.horizontal_wrapped(|ui| {
                if ui.link(&row.title).clicked() {
                    out.push(Msg::OpenItem(row.uid));
                }
                ui.weak(format!("{} % - {}", row.percent, row.drivers));
            });
        }
    }
}

/// The tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(b) = &state.reports else {
        ui.label("Computing the reports…");
        return;
    };
    egui::ScrollArea::vertical().show(ui, |ui| {
        spread(&b.familiarity, ui);
        groups("By stack", &b.familiarity.stacks, ui, out);
        groups("By subject", &b.familiarity.subjects, ui, out);
    });
}
