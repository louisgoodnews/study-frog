//! A Markdown editor: a toolbar of formatting buttons, and the text shown as Write, Split (text beside
//! its rendering) or Preview. The text is stored by the caller; the mode and the parsed preview live in
//! egui's memory, so the preview is parsed only when the text changes.

use std::sync::Arc;

use eframe::egui::{self, Id, Role, TextEdit, WidgetInfo, text::CCursor, text::CCursorRange};
use studyfrog_md::blocks::{Block, parse_blocks};

use crate::image_ui::ImageTarget;
use crate::md_edit::{Cmd, apply_cmd};
use crate::msg::Msg;
use crate::theme::palette_of;
use crate::view::preview::draw_blocks;

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

const TOOLS: [(Cmd, &str, &str); 10] = [
    (Cmd::Bold, "B", "Bold"),
    (Cmd::Italic, "I", "Italic"),
    (Cmd::Code, "<>", "Code"),
    (Cmd::Heading, "H", "Heading"),
    (Cmd::Bullet, "List", "Bullet list"),
    (Cmd::Numbered, "1.", "Numbered list"),
    (Cmd::Quote, "Quote", "Quote"),
    (Cmd::Link, "Link", "Link"),
    (Cmd::WikiLink, "[[ ]]", "Link to a note"),
    (Cmd::Rule, "---", "Rule"),
];

const LINE: f32 = 16.0;
const MIN_HEIGHT: f32 = 48.0;
const MAX_HEIGHT: f32 = 1200.0;

/// A bar under the editor that makes it taller when dragged, or with the arrow keys when it has focus.
fn resize_grip(ui: &mut egui::Ui, (name, id): (&str, Id), height: f32) {
    let (rect, r) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 10.0),
        egui::Sense::click_and_drag(),
    );
    let color = ui.visuals().weak_text_color();
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "....",
        egui::FontId::monospace(10.0),
        color,
    );
    let label = format!("Resize {name}");
    r.widget_info(|| WidgetInfo::labeled(Role::Button, true, label.clone()));
    let key = ui.input(|i| {
        i.key_pressed(egui::Key::ArrowDown) as i8 - i.key_pressed(egui::Key::ArrowUp) as i8
    });
    let step = if r.has_focus() {
        f32::from(key) * LINE * 2.0
    } else {
        0.0
    };
    let drag = if r.dragged() { r.drag_delta().y } else { 0.0 };
    if step != 0.0 || drag != 0.0 {
        ui.data_mut(|d| d.insert_temp(id, (height + step + drag).clamp(MIN_HEIGHT, MAX_HEIGHT)));
    }
}

type Cached = Arc<(String, Vec<Block>)>;

fn blocks_of(ui: &egui::Ui, id: Id, text: &str) -> Arc<(String, Vec<Block>)> {
    let key = id.with("blocks");
    if let Some(hit) = ui
        .data(|d| d.get_temp::<Cached>(key))
        .filter(|c| c.0 == text)
    {
        return hit;
    }
    let fresh: Cached = Arc::new((text.to_owned(), parse_blocks(text)));
    ui.data_mut(|d| d.insert_temp(key, fresh.clone()));
    fresh
}

fn toolbar(
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
                let sel = TextEdit::load_state(ui.ctx(), text_id)
                    .and_then(|s| s.cursor.char_range())
                    .map_or((text.chars().count(), text.chars().count()), |r| {
                        (r.primary.index.into(), r.secondary.index.into())
                    });
                edited = Some(apply_cmd(text, sel, cmd));
            }
        }
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

fn set_selection(ui: &egui::Ui, text_id: Id, (a, b): (usize, usize)) {
    if let Some(mut state) = TextEdit::load_state(ui.ctx(), text_id) {
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(a), CCursor::new(b))));
        state.store(ui.ctx(), text_id);
    }
    ui.ctx().memory_mut(|m| m.request_focus(text_id));
}

/// The editor for `text`, named `name` (also the accessible name of the text area). Returns the new
/// text when it changed. `rows` is the height of the text area in lines.
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
    resize_grip(ui, (name, height_id), height);
    if let Some(sel) = ui.data_mut(|d| d.remove_temp::<(usize, usize)>(id.with("select"))) {
        set_selection(ui, text_id, sel);
    }
    changed.then_some(current)
}
