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
use crate::typo;
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

/// The widest the text may be: one reading measure, or in Split mode two measures and the gap between them, so
/// each side is one measure. Never wider than the space there is.
fn column_width(ui: &egui::Ui, mode: Mode) -> f32 {
    let one = typo::measure(ui);
    let wanted = match mode {
        Mode::Split => 2.0 * one + ui.spacing().item_spacing.x,
        Mode::Write | Mode::Preview => one,
    };
    wanted.min(ui.available_width())
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
        // The body is the body font, not the code font: the measure is set in that font's
        // characters (`typo::measure`), and a monospace line holds about 30 % fewer of them.
        // `code_editor()` also locks focus so Tab inserts a tab; keep that.
        let edit = TextEdit::multiline(current)
            .id(text_id)
            .desired_rows(rows)
            .desired_width(f32::INFINITY)
            .font(typo::font(ui, typo::Tier::Body))
            .lock_focus(true);
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
    ui.scope(|ui| {
        ui.set_max_width(column_width(ui, mode));
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
    });
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
