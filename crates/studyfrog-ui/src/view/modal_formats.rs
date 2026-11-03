//! "Which files can I import?": every importer with the layout it expects, an example and a button to save it.

use eframe::egui::{self, RichText};
use studyfrog_service::xfer_adapters::{ADAPTERS, OBSIDIAN_LAYOUT};

use crate::library_ui::LibraryMsg;
use crate::msg::Msg;
use crate::theme::palette_of;
use crate::transfer_ui::TransferMsg;

fn format_entry(ui: &mut egui::Ui, i: usize, out: &mut Vec<Msg>) {
    let a = &ADAPTERS[i];
    ui.strong(a.name);
    ui.label(
        RichText::new(format!("Files: {}", a.extensions.join(", ")))
            .color(palette_of(ui).text_weak),
    );
    ui.label(a.layout);
    if let Some(ex) = a.example {
        ui.add(egui::Label::new(RichText::new(ex.text.trim_end()).monospace().size(12.0)).wrap());
        if ui.button(format!("Save example ({})", ex.file)).clicked() {
            out.push(Msg::Library(LibraryMsg::Transfer(
                TransferMsg::SaveExample(i),
            )));
        }
    }
    ui.add_space(8.0);
}

/// The list of formats, folded away until asked for.
pub fn formats(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::CollapsingHeader::new("Which files can I import?")
        .id_salt("import-formats")
        .show(ui, |ui| {
            for i in 0..ADAPTERS.len() {
                format_entry(ui, i, out);
            }
            ui.strong("Obsidian vault (folder)");
            ui.label(OBSIDIAN_LAYOUT);
            ui.add_space(4.0);
            ui.weak("The save button writes into the folder under Export, else Documents. More in the help: Import formats.");
        });
}
