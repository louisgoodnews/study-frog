//! The few building blocks every screen is made of: cards, pills, a primary button, stat tiles,
//! section headings and a progress bar. They take their colours from the theme.

use eframe::egui::{self, Button, Color32, CornerRadius, Frame, Margin, RichText, Stroke, vec2};

use crate::theme::{palette_of, tint};

/// A surface with a border and rounded corners that holds a group of content.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let p = palette_of(ui);
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), add)
                .inner
        })
        .inner
}

/// A card that can be clicked as a whole; returns whether it was.
pub fn card_button(ui: &mut egui::Ui, id: &str, add: impl FnOnce(&mut egui::Ui)) -> bool {
    let p = palette_of(ui);
    let outer = ui.make_persistent_id(id);
    let frame = Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(12))
        .inner_margin(Margin::same(16));
    let mut prepared = frame.begin(ui);
    let width = prepared.content_ui.available_width();
    prepared.content_ui.set_width(width);
    prepared
        .content_ui
        .with_layout(egui::Layout::top_down(egui::Align::Min), add);
    let response = prepared.allocate_space(ui);
    let response = ui.interact(response.rect, outer, egui::Sense::click());
    if response.hovered() {
        prepared.frame.fill = p.surface_hover;
        prepared.frame.stroke = Stroke::new(1.0, p.accent);
    }
    prepared.paint(ui);
    response.clicked()
}

/// A small rounded label with a tinted background.
pub fn pill(ui: &mut egui::Ui, text: &str, color: Color32) {
    Frame::new()
        .fill(tint(color, 40))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).color(color).size(12.5));
        });
}

/// The main action of a screen: an accent-coloured button.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let p = palette_of(ui);
    ui.add(
        Button::new(RichText::new(text).color(p.on_accent).strong())
            .fill(p.accent)
            .min_size(vec2(0.0, 34.0)),
    )
}

/// A big number with a caption under it.
pub fn stat(ui: &mut egui::Ui, value: &str, caption: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).size(28.0).strong());
        ui.label(RichText::new(caption).color(palette_of(ui).text_weak));
    });
}

/// A heading for a block of a screen.
pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(12.0);
    ui.label(RichText::new(title).size(17.0).strong());
    ui.add_space(2.0);
}

/// A thin bar showing `fraction` (0 to 1) of the way, with a name for screen readers.
pub fn progress(ui: &mut egui::Ui, fraction: f32, name: &str) {
    let p = palette_of(ui);
    let bar = egui::ProgressBar::new(fraction.clamp(0.0, 1.0))
        .desired_height(8.0)
        .fill(p.accent)
        .text(RichText::new("").size(1.0));
    ui.add(bar)
        .widget_info(|| egui::WidgetInfo::labeled(egui::Role::ProgressIndicator, true, name));
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

/// A small grey caption above a group of controls.
pub fn caption(ui: &mut egui::Ui, text: &str) {
    ui.add_space(10.0);
    ui.label(
        RichText::new(text)
            .size(13.0)
            .strong()
            .color(palette_of(ui).text_weak),
    );
    ui.add_space(2.0);
}

/// A caption and, under it, a card that holds the group.
pub fn group<R>(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    caption(ui, title);
    card(ui, |ui| {
        ui.set_width(ui.available_width());
        add(ui)
    })
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
