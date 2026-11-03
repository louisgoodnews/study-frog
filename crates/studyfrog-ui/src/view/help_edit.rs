//! Writing a user help page: its name and its Markdown, in the same editor as notes.

use eframe::egui;

use crate::help_types::{HelpMsg, PageEdit};
use crate::msg::Msg;
use crate::view::md_editor::{Mode, md_editor};
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: HelpMsg) {
    out.push(Msg::Help(m));
}

/// The form.
pub fn editor(edit: &PageEdit, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let heading = if edit.original.is_some() {
        "Edit your page"
    } else {
        "New page"
    };
    ui.heading(heading);
    ui.weak("Kept as a Markdown file in the wiki folder of your vault. Link to a help page with [[id]], for example [[library]].");
    if let Some(name) = text_field(ui, "Page name", &edit.name, "File name, without .md", 1) {
        send(out, HelpMsg::EditName(name));
    }
    if let Some(text) = md_editor(ui, "Page text", &edit.text, (18, Mode::Split), None, out) {
        send(out, HelpMsg::EditText(text));
    }
    if let Some(why) = &edit.problem {
        ui.colored_label(ui.visuals().error_fg_color, why);
    }
    ui.horizontal(|ui| {
        let label = if edit.saving {
            "Saving…"
        } else {
            "Save page"
        };
        if ui
            .add_enabled(!edit.saving, egui::Button::new(label))
            .clicked()
        {
            send(out, HelpMsg::Save);
        }
        if ui.button("Cancel").clicked() {
            send(out, HelpMsg::Cancel);
        }
    });
}
