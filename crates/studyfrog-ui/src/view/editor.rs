//! The item page: the title and the text areas in the main column, the details in a side column. It
//! is the same page for a note, a card, a question and a stack.

use eframe::egui::{self, RichText, TextEdit};

use crate::editor::{Editor, is_dirty};
use crate::image_ui::ImageTarget;
use crate::msg::{EditAction, Msg};
use crate::state::State;
use crate::theme::palette_of;
use crate::view::item_details;
use crate::view::md_editor::{Mode, md_editor};

const SIDE_WIDTH: f32 = 300.0;

/// The whole screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(e) = &state.editor else { return };
    toolbar(e, ui, out);
    ui.add_space(6.0);
    let total = ui.available_width();
    let side = SIDE_WIDTH.min(total * 0.4);
    let height = ui.available_height();
    ui.horizontal_top(|ui| {
        let down = egui::Layout::top_down(egui::Align::Min);
        ui.allocate_ui_with_layout(egui::vec2(total - side - 12.0, height), down, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("main")
                .show(ui, |ui| main_column(e, ui, out));
        });
        ui.allocate_ui_with_layout(egui::vec2(side, height), down, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("side")
                .show(ui, |ui| item_details::details(state, e, ui, out));
        });
    });
}

/// What the toolbar says about saving: nothing while everything is on disk.
fn status(e: &Editor, dirty: bool, ui: &mut egui::Ui) {
    if let Some(why) = &e.fail {
        let color = ui.visuals().error_fg_color;
        ui.label(RichText::new(format!("Not saved: {why}")).color(color));
    } else if e.saving.is_some() {
        ui.weak("saving…");
    } else if dirty {
        ui.weak("unsaved changes");
    }
}

fn toolbar(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let dirty = is_dirty(e);
    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            out.push(Msg::CloseEditor);
        }
        ui.label(RichText::new(&e.path).color(palette_of(ui).text_weak));
        status(e, dirty, ui);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.menu_button("More actions", |ui| {
                if ui.button("Export to PDF…").clicked() {
                    out.push(Msg::Pdf(crate::pdf_types::PdfMsg::Export(
                        crate::pdf_export_types::PdfExportMsg::Open(
                            studyfrog_service::pdf_export::ExportTarget::SingleItem(e.uid),
                        ),
                    )));
                    ui.close();
                }
            });
            if e.conflict && ui.button("Keep mine").clicked() {
                out.push(Msg::KeepMineClicked);
            }
            if ui.add_enabled(dirty, egui::Button::new("Revert")).clicked() {
                out.push(Msg::RevertClicked);
            }
            if ui.button("Reload from disk").clicked() {
                out.push(Msg::ReloadClicked);
            }
        });
    });
}

fn main_column(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let title = ui.label(RichText::new("Title").color(palette_of(ui).text_weak));
    let mut text = e.form.title.clone();
    let edit = TextEdit::singleline(&mut text)
        .font(egui::FontId::proportional(24.0))
        .desired_width(f32::INFINITY);
    if ui.add(edit).labelled_by(title.id).changed() {
        out.push(Msg::Edit(EditAction::Title(text)));
    }
    ui.add_space(8.0);
    for (i, f) in e.form.fields.iter().enumerate() {
        if let Some(t) = md_editor(
            ui,
            f.label,
            &f.text,
            (f.rows, Mode::Split),
            Some(ImageTarget::Item(i)),
            out,
        ) {
            out.push(Msg::Edit(EditAction::Field(i, t)));
        }
        ui.add_space(10.0);
    }
}
