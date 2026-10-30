//! Small popups under a top-bar button.

use eframe::egui::{self, Order, Pos2, Rect};

use crate::chrome::ChromeMsg;
use crate::msg::Msg;

/// Draw `content` in a popup whose top-left corner is `at`, wide at least `width`. Returns the
/// popup's rectangle.
pub fn popup(
    ui: &egui::Ui,
    id: &str,
    at: Pos2,
    width: f32,
    content: impl FnOnce(&mut egui::Ui),
) -> Rect {
    egui::Area::new(egui::Id::unique(id))
        .order(Order::Foreground)
        .fixed_pos(at)
        .constrain(true)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(width);
                content(ui);
            });
        })
        .response
        .rect
}

/// Whether the user pressed outside both rectangles this frame, or pressed Escape.
pub fn dismissed(ui: &egui::Ui, popup: Rect, anchor: Rect) -> bool {
    let (pressed, pos, escape) = ui.input(|i| {
        (
            i.pointer.any_pressed(),
            i.pointer.interact_pos(),
            i.key_pressed(egui::Key::Escape),
        )
    });
    escape || (pressed && pos.is_some_and(|p| !popup.contains(p) && !anchor.contains(p)))
}

/// Ask for the open menu to close.
pub fn close(out: &mut Vec<Msg>) {
    out.push(Msg::Chrome(ChromeMsg::CloseMenu));
}
