//! The application files section of the settings screen: where StudyFrog keeps its own files, and
//! how to read its log.

use eframe::egui::{self, RichText};
use studyfrog_config::paths::{AppPaths, LOG_JSON_FILE, LOG_TEXT_FILE};

use crate::msg::Msg;
use crate::runtime::paths;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

/// The rows of the section: what it is and where it is.
fn rows(p: &AppPaths) -> [(&'static str, String); 6] {
    let show = |path: std::path::PathBuf| path.display().to_string();
    [
        ("Application home", show(p.root.clone())),
        ("Settings and keys", show(p.config.clone())),
        ("Language models", show(p.models.clone())),
        ("Logs", show(p.logs.clone())),
        ("Text log", show(p.logs.join(LOG_TEXT_FILE))),
        ("JSON log", show(p.logs.join(LOG_JSON_FILE))),
    ]
}

/// The section.
pub fn draw(_state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let weak = palette_of(ui).text_weak;
    kit::section(ui, "Application files");
    let Some(p) = paths() else {
        ui.label(RichText::new("No home directory is known.").color(weak));
        return;
    };
    egui::Grid::new("application-files")
        .num_columns(3)
        .show(ui, |ui| {
            for (name, path) in rows(&p) {
                ui.label(name);
                // The button comes before the path, so a long path never pushes it off the screen.
                if ui
                    .small_button("Copy")
                    .on_hover_text("Copy the path")
                    .clicked()
                {
                    ui.ctx().copy_text(path.clone());
                }
                ui.label(RichText::new(&path).monospace());
                ui.end_row();
            }
        });
    let open = ui
        .button("Open logs folder")
        .on_hover_text("Show the logs folder in the file manager");
    if open.clicked() {
        out.push(Msg::Logs(crate::logs_ui::LogsMsg::Open));
    }
    ui.label(
        RichText::new(
            "Read the log in a terminal with `studyfrog logs --tail 50 [--follow] [--json]`.",
        )
        .color(weak),
    );
}
