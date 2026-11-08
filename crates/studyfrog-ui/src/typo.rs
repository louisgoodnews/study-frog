//! Typography (STORY-0114, `docs/specs/ui-colour-and-type.md`): a four-tier modular type scale, the line height of
//! prose, a reading measure of about 70 average characters per line, and numbers that keep their width while they change.
//! Every size follows the user's text size, because it is taken from the installed `Body` style.

use eframe::egui::{self, FontId, RichText, TextStyle};

/// The four sizes text comes in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Page titles, section headings, the prompt of a run, big numbers.
    Heading,
    /// Prose, notes, answers, ordinary labels.
    Body,
    /// Buttons, navigation, table cells, captions above groups.
    Label,
    /// Badges, keyboard hints, timestamps, small counts.
    Caption,
}

/// Every tier, largest first.
pub const TIERS: [Tier; 4] = [Tier::Heading, Tier::Body, Tier::Label, Tier::Caption];

/// The body size at 100 % text size, in points.
pub const BASE: f32 = 15.0;

/// The size of a tier as a share of the body size.
pub fn ratio(t: Tier) -> f32 {
    match t {
        Tier::Heading => 1.40,
        Tier::Body => 1.00,
        Tier::Label => 0.92,
        Tier::Caption => 0.80,
    }
}

/// Line height of prose as a share of its size.
pub const LINE_HEIGHT: f32 = 1.5;

/// Characters per line the reading measure aims at (the 60 to 75 that reading studies recommend).
pub const MEASURE_CHARS: f32 = 70.0;

/// A tier's size for a body size (points).
pub fn size_for(body: f32, t: Tier) -> f32 {
    body * ratio(t)
}

/// The body size installed in this `ui` (the user's text size is in it).
pub fn body_size(ui: &egui::Ui) -> f32 {
    ui.style()
        .text_styles
        .get(&TextStyle::Body)
        .map_or(BASE, |f| f.size)
}

/// A tier's size in this `ui`.
pub fn size(ui: &egui::Ui, t: Tier) -> f32 {
    size_for(body_size(ui), t)
}

/// A proportional font of a tier.
pub fn font(ui: &egui::Ui, t: Tier) -> FontId {
    FontId::proportional(size(ui, t))
}

/// A proportional font of a tier for drawing with a painter (the body size of the current theme's style).
pub fn painter_font(painter: &egui::Painter, t: Tier) -> FontId {
    let body = painter
        .ctx()
        .global_style()
        .text_styles
        .get(&TextStyle::Body)
        .map_or(BASE, |f| f.size);
    FontId::proportional(size_for(body, t))
}

/// Text in a tier.
pub fn text(ui: &egui::Ui, s: impl Into<String>, t: Tier) -> RichText {
    RichText::new(s).size(size(ui, t))
}

/// The line height of prose at `size` points.
pub fn line_height(size: f32) -> f32 {
    (size * LINE_HEIGHT).round()
}

/// Prose at `size` points with the prose line height.
pub fn prose(_ui: &egui::Ui, s: &str, size: f32) -> RichText {
    RichText::new(s)
        .size(size)
        .line_height(Some(line_height(size)))
}

/// A sample of ordinary English prose, for measuring the width of an average character of a font.
pub const SAMPLE: &str = "The cell membrane is a thin, flexible barrier that separates the interior of every \
cell from the world outside. Because small molecules slip through by diffusion, it is selectively permeable.";

/// The width of an average character of prose in `font`.
pub fn char_width(ctx: &egui::Context, font: &FontId) -> f32 {
    let w = ctx.fonts_mut(|f| {
        f.layout_no_wrap(SAMPLE.to_owned(), font.clone(), egui::Color32::PLACEHOLDER)
            .size()
            .x
    });
    w / SAMPLE.chars().count() as f32
}

/// The widest a column of prose should be in this `ui`: [`MEASURE_CHARS`] average characters of body text.
pub fn measure(ui: &egui::Ui) -> f32 {
    let font = FontId::proportional(body_size(ui));
    (char_width(ui.ctx(), &font) * MEASURE_CHARS).round()
}

/// Whether every digit of `font` has the same advance width (tabular figures), so a changing number keeps its
/// width as long as it keeps its number of characters.
pub fn digits_are_tabular(ctx: &egui::Context, font: &FontId) -> bool {
    let widths: Vec<f32> = ('0'..='9')
        .map(|d| ctx.fonts_mut(|f| f.glyph_width(font, d)))
        .collect();
    widths.iter().all(|w| (w - widths[0]).abs() < 0.01)
}

/// `text` with every digit replaced by a zero: the widest a changing number of the same length can be, since the
/// digits are tabular.
pub fn figure_template(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_ascii_digit() { '0' } else { c })
        .collect()
}

/// The width of `text` in `font`.
pub fn text_width(ctx: &egui::Context, text: &str, font: &FontId) -> f32 {
    ctx.fonts_mut(|f| {
        f.layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::PLACEHOLDER)
            .size()
            .x
    })
}
