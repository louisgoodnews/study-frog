//! The few building blocks every screen is made of: cards, pills, a primary button, stat tiles,
//! section headings and a progress bar. They take their colours from the theme.

use eframe::egui::{self, Button, Color32, CornerRadius, Frame, Margin, RichText, vec2};

use crate::theme::{palette_of, tint};

pub use super::kit_card::{caption, card, card_button, card_with, group, section};

/// A small rounded label with a tinted background.
pub fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    figure_pill(ui, text, color, "");
}

/// A pill for a number that changes (a timer, a count): never narrower than `widest`, so the controls beside it
/// stay where they are while the digits tick (the fonts' digits are tabular, `tests/typography.rs`).
pub fn figure_pill(ui: &mut egui::Ui, text: &str, color: Color32, widest: &str) {
    let font = crate::typo::font(ui, crate::typo::Tier::Caption);
    let min = crate::typo::text_width(ui.ctx(), widest, &font);
    Frame::new()
        .fill(tint(color, 40))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.set_min_width(min);
            ui.label(RichText::new(text).color(color).font(font));
        });
}

/// The main action of a screen: an accent-coloured button.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette_of(ui);
    crate::theme_space::note_primary(ui.ctx());
    ui.add(
        Button::new(RichText::new(text).color(p.on_accent).strong())
            .fill(p.accent)
            .min_size(vec2(
                0.0,
                crate::theme::MIN_TARGET.max(ui.spacing().interact_size.y),
            )),
    )
}

/// A secondary action that should still be found quickly: the quiet fill of other buttons, the label in the
/// accent. It does not count as the view's one accent-filled primary action.
pub fn quiet_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette_of(ui);
    ui.add(Button::new(RichText::new(text).color(p.accent).strong()))
}

/// A big number with a caption under it.
pub fn stat(ui: &mut egui::Ui, value: &str, caption: &str) {
    ui.vertical(|ui| {
        ui.label(
            RichText::new(value)
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.label(RichText::new(caption).color(palette_of(ui).text_weak));
    });
}

/// A full-width bar with `text` centred on it; screen readers hear `name` and the text. The filled part is the
/// theme's tinted accent, so the text keeps its normal colour and stays readable at any fill.
pub fn progress_with_text(ui: &mut egui::Ui, fraction: f32, text: &str, name: &str) {
    let p = palette_of(ui);
    ui.add_space(4.0);
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 24.0), egui::Sense::hover());
    let radius = CornerRadius::same(12);
    ui.painter().rect_filled(rect, radius, p.surface_hover);
    let fill = fraction.clamp(0.0, 1.0);
    if fill > 0.0 {
        let width = (rect.width() * fill).max(rect.height());
        let done = egui::Rect::from_min_size(rect.min, vec2(width, rect.height()));
        ui.painter().rect_filled(done, radius, tint(p.accent, 110));
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        text,
        crate::typo::font(ui, crate::typo::Tier::Label),
        p.text,
    );
    let label = format!("{name}: {text}");
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, true, &label));
}

/// Widest a column of reading or form content is drawn, however wide the window is.
pub const READING_WIDTH: f32 = 780.0;

/// A column at most `max` points wide, centred in the space there is, so lines do not run across the whole window.
pub fn column<R>(ui: &mut egui::Ui, max: f32, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let width = ui.available_width().min(max);
    let side = ((ui.available_width() - width) / 2.0).max(0.0);
    ui.horizontal_top(|ui| {
        ui.add_space(side);
        ui.vertical(|ui| {
            ui.set_width(width);
            add(ui)
        })
        .inner
    })
    .inner
}

/// A centred column as wide as the reading measure (`typo::measure`, about 70 characters of body text), for pages
/// that are mostly prose.
pub fn reading<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let width = crate::typo::measure(ui);
    column(ui, width, add)
}

/// One row of [`bar_rows`].
pub struct BarRow {
    /// The name in the first column.
    pub label: String,
    /// How full the bar is, 0 to 1.
    pub fraction: f32,
    /// Text written on the bar.
    pub text: String,
    /// The colour of the bar and the name; the theme's accent when `None`.
    pub color: Option<Color32>,
}

/// Rows of a name and a bar: the names share one column and the bars start at the same place and have the same
/// full length, so the bars can be compared by eye. The numbers are in the text, never only in the colour.
pub fn bar_rows(ui: &mut egui::Ui, id: &str, rows: &[BarRow]) {
    let bar_width = ui.available_width().clamp(120.0, 360.0) - 96.0;
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([16.0, 8.0])
        .show(ui, |ui| {
            for r in rows {
                match r.color {
                    Some(c) => ui.colored_label(c, &r.label),
                    None => ui.label(&r.label),
                };
                let bar = egui::ProgressBar::new(r.fraction.clamp(0.0, 1.0))
                    .desired_width(bar_width.max(80.0))
                    .desired_height(20.0)
                    .text(RichText::new(&r.text));
                // Without a colour the theme's tinted selection fill is used: its text colour is the accent, which
                // would vanish on an accent fill.
                ui.add(match r.color {
                    Some(c) => bar.fill(c),
                    None => bar,
                });
                ui.end_row();
            }
        });
}
