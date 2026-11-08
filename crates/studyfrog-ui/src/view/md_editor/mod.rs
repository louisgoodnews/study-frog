//! A Markdown editor: a toolbar of formatting buttons, and the text shown as Write, Split (text beside
//! its rendering) or Preview. The text is stored by the caller; the mode and the parsed preview live in
//! egui's memory, so the preview is parsed only when the text changes.

mod blocks;
mod keys;
mod resize;
mod toolbar;

use eframe::egui::{self, TextEdit};

use crate::image_ui::ImageTarget;
use crate::msg::Msg;
use crate::theme::palette_of;
use crate::view::preview::draw_blocks;
use blocks::blocks_of;
use resize::{LINE, MAX_HEIGHT, MIN_HEIGHT, resize_grip};
use toolbar::{set_selection, toolbar};

/// A macro whose closing brace was just typed: the new text, the selection and whether to choose a picture.
fn macro_at_caret(
    ui: &egui::Ui,
    text_id: egui::Id,
    (before, now): (&str, &str),
) -> Option<(String, (usize, usize), bool)> {
    // Only a single typed character can finish a macro; deleting or pasting never expands one.
    if now.chars().count() != before.chars().count() + 1 {
        return None;
    }
    let range = TextEdit::load_state(ui.ctx(), text_id)?
        .cursor
        .char_range()?;
    let caret: usize = range.primary.index.into();
    let (span, e) = crate::md_macros::macro_before(now, caret)?;
    let (text, sel) = crate::md_macros::expand_in(now, span, &e);
    Some((text, sel, e.picture))
}

/// How the editor shows the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Only the text.
    Write,
    /// The text and its rendering side by side.
    Split,
    /// Only the rendering.
    Preview,
}

/// The editor for `text`, named `name` (also the accessible name of the text area). Returns the new
pub fn md_editor(
    ui: &mut egui::Ui,
    name: &str,
    text: &str,
    (rows, start): (usize, Mode),
    image: Option<ImageTarget>,
    out: &mut Vec<Msg>,
) -> Option<String> {
    let id = ui.make_persistent_id(("md-editor", name));
    let (text_id, mode_id) = (id.with("text"), id.with("mode"));
    let mode = ui.data(|d| d.get_temp::<Mode>(mode_id)).unwrap_or(start);
    let label = ui.label(egui::RichText::new(name).strong());
    let mut current = text.to_owned();
    let mut changed = false;
    if let Some((new, sel)) = toolbar(ui, (name, text_id), mode_id, mode, (text, image, out)) {
        current = new;
        changed = true;
        ui.data_mut(|d| d.insert_temp(id.with("select"), sel));
    }
    if mode != Mode::Preview
        && let Some((new, sel)) =
            keys::list_key(ui, text_id, &current).or_else(|| keys::cloze_key(ui, text_id, &current))
    {
        current = new;
        changed = true;
        ui.data_mut(|d| d.insert_temp(id.with("select"), sel));
    }
    let pal = palette_of(ui);
    let height_id = id.with("height");
    let height = ui
        .data(|d| d.get_temp::<f32>(height_id))
        .unwrap_or(rows as f32 * LINE)
        .clamp(MIN_HEIGHT, MAX_HEIGHT);
    let rows = (height / LINE).round().max(2.0) as usize;
    let body = |ui: &mut egui::Ui, current: &mut String, changed: &mut bool| {
        let edit = TextEdit::multiline(current)
            .id(text_id)
            .desired_rows(rows)
            .desired_width(f32::INFINITY)
            .code_editor();
        if ui.add(edit).labelled_by(label.id).changed() {
            *changed = true;
        }
    };
    let rendered = |ui: &mut egui::Ui, shown: &str, out: &mut Vec<Msg>| {
        let cache = blocks_of(ui, id, shown);
        egui::Frame::new()
            .fill(pal.surface)
            .inner_margin(egui::Margin::same(8))
            .corner_radius(6)
            .show(ui, |ui| {
                ui.set_min_height(rows as f32 * 16.0);
                ui.set_width(ui.available_width());
                draw_blocks(ui, &cache.1, out);
            });
    };
    match mode {
        Mode::Write => body(ui, &mut current, &mut changed),
        Mode::Preview => rendered(ui, text, out),
        Mode::Split => {
            ui.columns(2, |cols| {
                body(&mut cols[0], &mut current, &mut changed);
                rendered(&mut cols[1], &current, out);
            });
        }
    }
    if changed
        && mode != Mode::Preview
        && let Some(done) = macro_at_caret(ui, text_id, (text, &current))
    {
        current = done.0;
        ui.data_mut(|d| d.insert_temp(id.with("select"), done.1));
        if done.2
            && let Some(target) = image
        {
            out.push(Msg::PickImage(target));
        }
    }
    resize_grip(ui, (name, height_id), height);
    if let Some(sel) = ui.data_mut(|d| d.remove_temp::<(usize, usize)>(id.with("select"))) {
        set_selection(ui, text_id, sel);
    }
    changed.then_some(current)
}
