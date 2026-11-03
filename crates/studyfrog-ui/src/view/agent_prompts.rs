//! The Prompts tab: ready-made instructions for an AI, to copy or save, that make it produce files StudyFrog imports.

use eframe::egui::{self, RichText};
use studyfrog_service::prompts::{Prompt, prompts};

use crate::msg::Msg;
use crate::prompts_ui::PromptsMsg;
use crate::theme::palette_of;
use crate::view::kit;

fn entry(p: &Prompt, id: &'static str, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(&p.title).strong());
        ui.label(RichText::new(&p.summary).color(palette_of(ui).text_weak));
        ui.horizontal(|ui| {
            if ui
                .button("Copy to clipboard")
                .on_hover_text("Paste it into any AI chat and attach your file.")
                .clicked()
            {
                ui.ctx().copy_text(p.text.clone());
            }
            if ui.button("Save as .md").clicked() {
                out.push(Msg::Prompts(PromptsMsg::Save(id)));
            }
        });
        egui::CollapsingHeader::new("Show the prompt")
            .id_salt(("prompt", id))
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&p.text).monospace().size(12.0)).wrap());
            });
    });
}

/// The tab.
pub fn prompts_tab(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let weak = palette_of(ui).text_weak;
    ui.label(RichText::new("Prompts for any AI chat. Copy one, attach your file (notes, code, data or text from a PDF) and paste the answer into a file ending in .csv; then import it. No connection to an assistant is needed.").color(weak));
    if ui.button("Save all prompts to a folder…").clicked() {
        out.push(Msg::Prompts(PromptsMsg::SaveAll));
    }
    ui.add_space(6.0);
    for (p, source) in prompts().iter().zip(studyfrog_service::prompts::PROMPTS) {
        entry(p, source.id, ui, out);
    }
}
