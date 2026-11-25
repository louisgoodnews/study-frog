//! The reports headline (EPIC-070, STORY-0218): the title and the CSV exports on the gradient panel, then the tabs.

use eframe::egui::{self, RichText};

use crate::explorer_ui::{ExplorerMsg, ReportTab};
use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::transfer_ui::TransferMsg;
use crate::view::gradient_panel;

/// The height of the headline panel, in points.
const HEADER_HEIGHT: f32 = 64.0;

/// The title and the two CSV exports, on the gradient panel.
pub fn headline(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    gradient_panel::panel(ui, HEADER_HEIGHT, |ui, s| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Reports")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong()
                    .color(s.gradient_text),
            );
            export_buttons(state, ui, out);
        });
    });
}

fn export_buttons(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let button = egui::Button::new("Export as CSV");
    let tip = "Write the reports as a CSV file into the exports folder of the vault.";
    if ui
        .add_enabled(state.vault.is_some(), button)
        .on_hover_text(tip)
        .clicked()
    {
        studyfrog_log::log_info!(
            "reports",
            "CSV export requested into the vault exports folder"
        );
        out.push(Msg::Library(LibraryMsg::Transfer(
            TransferMsg::ExportReport(false),
        )));
    }
    let pick = egui::Button::new("Save CSV to...");
    let tip = "Choose a folder and write the reports there as a CSV file.";
    if ui
        .add_enabled(state.vault.is_some(), pick)
        .on_hover_text(tip)
        .clicked()
    {
        studyfrog_log::log_info!("reports", "CSV export requested to a chosen folder");
        let to = TransferMsg::ExportReport(true);
        out.push(Msg::Library(LibraryMsg::Transfer(to)));
    }
}

/// The tab strip. A click selects the tab; the report itself is drawn by `reports::draw`.
pub fn tabs(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        for (tab, label) in [
            (ReportTab::Performance, "Performance"),
            (ReportTab::Memory, "Memory and forecast"),
            (ReportTab::Familiarity, "Familiarity"),
            (ReportTab::Calibration, "Self-assessment"),
            (ReportTab::Cognitive, "Cognitive profile"),
            (ReportTab::Runs, "Runs"),
            (ReportTab::Habits, "Habits"),
            (ReportTab::Leeches, "Leeches"),
            (ReportTab::Workload, "Workload"),
        ] {
            if ui
                .selectable_label(state.explorer.tab == tab, label)
                .clicked()
            {
                studyfrog_log::log_info!("reports", "report tab selected: {}", label);
                out.push(Msg::Explorer(ExplorerMsg::Tab(tab)));
            }
        }
    });
}

/// The name of a tab, for logs and timings.
pub fn tab_name(tab: ReportTab) -> &'static str {
    match tab {
        ReportTab::Performance => "performance",
        ReportTab::Memory => "memory",
        ReportTab::Familiarity => "familiarity",
        ReportTab::Calibration => "calibration",
        ReportTab::Cognitive => "cognitive",
        ReportTab::Runs => "runs",
        ReportTab::Habits => "habits",
        ReportTab::Leeches => "leeches",
        ReportTab::Workload => "workload",
    }
}
