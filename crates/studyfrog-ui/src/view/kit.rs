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
    ui.add_space(6.0);
    ui.label(RichText::new(title).size(18.0).strong());
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
