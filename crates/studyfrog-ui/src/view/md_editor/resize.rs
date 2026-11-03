//! The bar under an editor that makes it taller.

use eframe::egui::{self, Id, Role, WidgetInfo};

pub(super) const LINE: f32 = 16.0;
pub(super) const MIN_HEIGHT: f32 = 48.0;
pub(super) const MAX_HEIGHT: f32 = 1200.0;

/// A bar under the editor that makes it taller when dragged, or with the arrow keys when it has focus.
pub(super) fn resize_grip(ui: &mut egui::Ui, (name, id): (&str, Id), height: f32) {
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
