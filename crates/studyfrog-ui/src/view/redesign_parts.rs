//! Badges and progress bars in the redesign style (EPIC-070, STORY-0229). Both take their colours from `style_of`, so a
//! badge and a bar read the same on every palette. The text is the caller's; nothing here changes what it says.

use eframe::egui::{self, Align2, CornerRadius, Frame, Margin, Rect, RichText, Sense, vec2};

use crate::theme::palette_of;
use crate::theme::style::style_of;

/// The height of a progress bar, in points.
const PROGRESS_HEIGHT: f32 = 24.0;
/// The corner radius of a badge and a progress bar, in points: fully rounded ends.
const PILL_RADIUS: u8 = 12;

/// A small rounded label on the badge fill, for a stack, a rating or a status.
pub(super) fn badge(ui: &mut egui::Ui, text: &str) {
    let s = style_of(&palette_of(ui));
    Frame::new()
        .fill(s.badge)
        .corner_radius(CornerRadius::same(PILL_RADIUS))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(s.badge_text));
        });
}

/// A progress bar across the available width: the track, the filled part up to `fraction` (0 to 1), and the text
/// centred on top.
pub(super) fn progress(ui: &mut egui::Ui, fraction: f32, text: &str) {
    let p = palette_of(ui);
    let s = style_of(&p);
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), PROGRESS_HEIGHT), Sense::hover());
    let radius = CornerRadius::same(PILL_RADIUS);
    let painter = ui.painter();
    painter.rect_filled(rect, radius, s.ring_track);
    let fill = fraction.clamp(0.0, 1.0);
    if fill > 0.0 {
        let width = (rect.width() * fill).max(rect.height());
        let done = Rect::from_min_size(rect.min, vec2(width, rect.height()));
        painter.rect_filled(done, radius, s.ring_fill);
    }
    let font = crate::typo::font(ui, crate::typo::Tier::Caption);
    ui.painter()
        .text(rect.center(), Align2::CENTER_CENTER, text, font, p.text);
}
