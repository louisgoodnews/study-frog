//! The editor screen: form on the left, live preview on the right.

use eframe::egui;

use crate::editor::{Editor, is_dirty};
use crate::msg::{EditAction, Msg};
use crate::state::State;
use crate::view::preview::draw_blocks;
use crate::view::widgets::text_field;

/// The whole screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(e) = &state.editor else { return };
    toolbar(e, ui, out);
    ui.separator();
    ui.columns(2, |cols| {
        form(e, &mut cols[0], out);
        preview(e, &mut cols[1], out);
    });
}

fn toolbar(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let dirty = is_dirty(e);
    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            out.push(Msg::CloseEditor);
        }
        ui.heading(&e.path);
        if dirty {
            ui.weak("unsaved changes");
        }
        let saveable = dirty && e.saving.is_none();
        if ui
            .add_enabled(saveable, egui::Button::new("Save"))
            .clicked()
        {
            out.push(Msg::SaveClicked);
        }
        if ui.add_enabled(dirty, egui::Button::new("Revert")).clicked() {
            out.push(Msg::RevertClicked);
        }
        if ui.button("Reload from disk").clicked() {
            out.push(Msg::ReloadClicked);
        }
    });
}

fn form(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::ScrollArea::vertical().id_salt("form").show(ui, |ui| {
        if let Some(t) = text_field(ui, "Title", &e.form.title, "", 1) {
            out.push(Msg::Edit(EditAction::Title(t)));
        }
        if let Some(t) = text_field(ui, "Tags (comma separated)", &e.form.tags, "", 1) {
            out.push(Msg::Edit(EditAction::Tags(t)));
        }
        for (i, f) in e.form.fields.iter().enumerate() {
            if let Some(t) = text_field(ui, f.label, &f.text, "", f.rows) {
                out.push(Msg::Edit(EditAction::Field(i, t)));
            }
        }
    });
}

fn preview(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::ScrollArea::vertical()
        .id_salt("preview")
        .show(ui, |ui| {
            ui.heading("Preview");
            for (field, blocks) in e.form.fields.iter().zip(&e.previews) {
                ui.strong(field.label);
                draw_blocks(ui, blocks, out);
                ui.add_space(8.0);
            }
        });
}
