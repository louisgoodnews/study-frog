//! The navigation rail (EPIC-070): a row of buttons in the rail fills. The selected one carries the
//! highlight; a click reports the item it selects.
use eframe::egui::{self, RichText};

use crate::theme::palette_of;
use crate::theme::style::style_of;

/// One rail item: the value it selects, its label and its hover explanation.
pub type Item<T> = (T, &'static str, &'static str);

/// Draw the rail and return the value of the item clicked this frame, if any.
pub fn buttons<T: Copy + PartialEq>(ui: &mut egui::Ui, items: &[Item<T>], current: T) -> Option<T> {
    let p = palette_of(ui);
    let s = style_of(&p);
    let mut clicked = None;
    ui.horizontal_wrapped(|ui| {
        for &(value, label, tip) in items {
            let selected = value == current;
            let text = if selected { p.accent } else { p.text };
            let fill = if selected { s.rail_active } else { s.rail };
            let button = egui::Button::new(RichText::new(label).color(text)).fill(fill);
            if ui.add(button).on_hover_text(tip).clicked() {
                clicked = Some(value);
            }
        }
    });
    clicked
}
