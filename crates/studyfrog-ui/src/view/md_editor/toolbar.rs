//! The toolbar of an editor: the mode switch, the formatting buttons and the picture button.

use eframe::egui::{self, Id, Role, TextEdit, WidgetInfo, text::CCursor, text::CCursorRange};

use super::Mode;
use crate::image_ui::ImageTarget;
use crate::md_edit::{Cmd, apply_cmd};
use crate::msg::Msg;
use studyfrog_md::hue::{HUES, hue_name};

const TOOLS: [(Cmd, &str, &str); 14] = [
    (Cmd::Bold, "B", "Bold"),
    (Cmd::Italic, "I", "Italic"),
    (Cmd::Underline, "U", "Underline"),
    (Cmd::Strike, "S", "Strikethrough"),
    (Cmd::Code, "<>", "Code"),
    (Cmd::Heading, "H", "Heading"),
    (Cmd::Bullet, "List", "Bullet list"),
    (Cmd::Numbered, "1.", "Numbered list"),
    (Cmd::Quote, "Quote", "Quote"),
    (Cmd::Link, "Link", "Link"),
    (Cmd::WikiLink, "[[ ]]", "Link to a note"),
    (Cmd::Table, "Table", "Table"),
    (Cmd::Rule, "---", "Rule"),
    (
        Cmd::Cloze(false),
        "[...]",
        "Cloze: hide the selection as a gap (Ctrl+Shift+C; with Alt the same number)",
    ),
];

pub(super) fn toolbar(
    ui: &mut egui::Ui,
    (name, text_id): (&str, Id),
    mode_id: Id,
    mode: Mode,
    (text, image, out): (&str, Option<ImageTarget>, &mut Vec<Msg>),
) -> Option<(String, (usize, usize))> {
    let mut edited = None;
    ui.horizontal_wrapped(|ui| {
        for (m, label) in [
            (Mode::Write, "Write"),
            (Mode::Split, "Split"),
            (Mode::Preview, "Preview"),
        ] {
            let r = ui.selectable_label(mode == m, label);
            r.widget_info(|| {
                WidgetInfo::labeled(
                    Role::Button,
                    true,
                    format!("{name}: {}", label.to_lowercase()),
                )
            });
            if r.clicked() {
                ui.data_mut(|d| d.insert_temp(mode_id, m));
            }
        }
        if mode == Mode::Preview {
            return;
        }
        ui.separator();
        for (cmd, label, tip) in TOOLS {
            let r = ui.small_button(label);
            r.widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("{tip} in {name}")));
            if r.on_hover_text(tip).clicked() {
                edited = Some(apply_cmd(text, selection(ui, text_id, text), cmd));
            }
        }
        ui.menu_button("Colour", |ui| {
            for hue in HUES {
                let label = hue_name(hue);
                let r = ui.button(label);
                r.widget_info(|| {
                    let what = format!("Text colour {label} in {name}");
                    WidgetInfo::labeled(Role::Button, true, what)
                });
                if r.clicked() {
                    let sel = selection(ui, text_id, text);
                    edited = Some(apply_cmd(text, sel, Cmd::Color(hue)));
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("Colour the selected text");
        let macros = ui.menu_button("{ }", |ui| {
            ui.weak("Type one; it turns into Markdown at the closing brace.");
            for m in crate::md_macros::MACROS {
                ui.label(format!("{}: {}", m.usage, m.help));
            }
        });
        macros
            .response
            .widget_info(|| WidgetInfo::labeled(Role::Button, true, format!("Macros for {name}")));
        macros.response.on_hover_text("Macros you can type");
        if let Some(target) = image {
            let r = ui.small_button("Picture");
            r.widget_info(|| {
                WidgetInfo::labeled(Role::Button, true, format!("Add a picture to {name}"))
            });
            if r.on_hover_text("Add a picture from a file").clicked() {
                out.push(Msg::PickImage(target));
            }
        }
    });
    edited
}

/// The selected characters of the text field (the end of the text when it has no cursor yet).
fn selection(ui: &egui::Ui, text_id: Id, text: &str) -> (usize, usize) {
    TextEdit::load_state(ui.ctx(), text_id)
        .and_then(|s| s.cursor.char_range())
        .map_or((text.chars().count(), text.chars().count()), |r| {
            (r.primary.index.into(), r.secondary.index.into())
        })
}

pub(super) fn set_selection(ui: &egui::Ui, text_id: Id, (a, b): (usize, usize)) {
    if let Some(mut state) = TextEdit::load_state(ui.ctx(), text_id) {
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
        state.store(ui.ctx(), text_id);
    }
    ui.ctx().memory_mut(|m| m.request_focus(text_id));
}
