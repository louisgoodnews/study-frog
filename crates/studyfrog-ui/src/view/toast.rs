//! In-app notifications that can be clicked. They sit at the top right, just under the header, so they
//! never cover the controls at the bottom of a screen (the setup's Next button, the status bar).

use eframe::egui::{self, Align2, Button, Order, vec2};

use crate::modal::ModalMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};

/// The widest a toast is drawn, in points: narrow enough to stay clear of the step bars.
const TOAST_WIDTH: f32 = 340.0;

/// The toasts, newest at the bottom of their stack.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if state.toasts.is_empty() || state.modal.is_some() {
        return;
    }
    egui::Area::new(egui::Id::unique("toasts"))
        .order(Order::Foreground)
        .anchor(Align2::RIGHT_TOP, vec2(-16.0, 56.0))
        .show(ui.ctx(), |ui| {
            ui.set_max_width(TOAST_WIDTH);
            for (i, toast) in state.toasts.iter().enumerate() {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let text = format!("{}\n{}", toast.title, toast.detail);
                        let card = Button::new(text).min_size(vec2(300.0, 54.0)).wrap();
                        if ui
                            .add(card)
                            .on_hover_text("Opens the notification it describes.")
                            .clicked()
                        {
                            out.push(Msg::Modal(ModalMsg::ToastClicked(i)));
                        }
                        if icon_button(ui, Glyph::Close, "Dismiss notification", false).clicked() {
                            out.push(Msg::Modal(ModalMsg::ToastDismissed(i)));
                        }
                    });
                });
            }
        });
}
