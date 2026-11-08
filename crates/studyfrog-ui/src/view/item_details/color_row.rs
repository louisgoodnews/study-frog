//! The Color row of the Properties tab (STORY-0097, STORY-0123): None, the palette swatches, a
//! colour picker and a `#RRGGBB` field for any other colour.

use eframe::egui::{self, Button, Role, Stroke, WidgetInfo, vec2};
use studyfrog_core::item_color::{PALETTE, color_hex, color_rgb, normalize_color};

use crate::editor::Editor;
use crate::item_look::swatch;
use crate::msg::{EditAction, Msg};
use crate::theme::palette_of;

fn set(out: &mut Vec<Msg>, color: Option<String>) {
    out.push(Msg::Edit(EditAction::Color(color)));
}

/// One palette colour as a round swatch, ringed when chosen, named "Color: Sage" for screen readers.
fn chip(ui: &mut egui::Ui, (word, label): (&str, &str), chosen: bool) -> bool {
    let Some(fill) = swatch(word) else {
        return false;
    };
    let ring = if chosen { palette_of(ui).text } else { fill };
    let button = Button::new("")
        .fill(fill)
        .stroke(Stroke::new(if chosen { 2.5 } else { 1.0 }, ring))
        .corner_radius(11)
        .min_size(vec2(22.0, 22.0));
    let r = ui.add(button).on_hover_text(label);
    r.widget_info(|| WidgetInfo::selected(Role::Button, true, chosen, format!("Color: {label}")));
    r.clicked()
}

/// The `#RRGGBB` field; what is typed is kept while it is not yet a colour.
fn hex_field(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let id = ui.make_persistent_id(("item-color-hex", e.uid));
    let current = e.form.color.as_deref().and_then(color_rgb).map(color_hex);
    let stored: Option<(Option<String>, String)> = ui.data(|d| d.get_temp(id));
    let mut text = match stored {
        Some((was, draft)) if was == current => draft,
        _ => current.clone().unwrap_or_default(),
    };
    let r = ui.add(
        egui::TextEdit::singleline(&mut text)
            .hint_text("#RRGGBB")
            .desired_width(80.0),
    );
    r.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, "Custom color (#RRGGBB)"));
    let valid = normalize_color(&text).filter(|v| v.starts_with('#'));
    if r.changed()
        && let Some(v) = valid.clone()
    {
        set(out, Some(v));
    }
    let shown = if r.changed() {
        valid.or(current)
    } else {
        current
    };
    ui.data_mut(|d| d.insert_temp(id, (shown, text)));
}

/// The row.
pub fn draw(e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.label("Color");
    let chosen = e.form.color.as_deref();
    ui.horizontal_wrapped(|ui| {
        let none = ui.selectable_label(chosen.is_none(), "None");
        none.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Color: none"));
        if none.clicked() {
            set(out, None);
        }
        for (word, label, _) in PALETTE {
            if chip(ui, (word, label), chosen == Some(word)) {
                set(out, Some(word.to_owned()));
            }
        }
    });
    ui.horizontal(|ui| {
        let mut rgb = chosen.and_then(color_rgb).unwrap_or([0x80, 0x80, 0x80]);
        let picker = egui::color_picker::color_edit_button_srgb(ui, &mut rgb);
        picker.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Pick a color"));
        if picker.changed() {
            set(out, Some(color_hex(rgb)));
        }
        hex_field(e, ui, out);
    });
}
