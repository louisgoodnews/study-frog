//! Small shared widgets.

use eframe::egui::{self, TextEdit};

/// A label with a text field bound to it for assistive technology. Returns the new text when the
/// user changed it. `rows` of 1 gives a single-line field.
pub fn text_field(
    ui: &mut egui::Ui,
    label: &str,
    current: &str,
    hint: &str,
    rows: usize,
) -> Option<String> {
    let name = ui.label(label);
    let mut text = current.to_owned();
    let edit = if rows <= 1 {
        TextEdit::singleline(&mut text)
            .hint_text(hint)
            .desired_width(f32::INFINITY)
    } else {
        TextEdit::multiline(&mut text)
            .hint_text(hint)
            .desired_rows(rows)
            .desired_width(f32::INFINITY)
    };
    ui.add(edit).labelled_by(name.id);
    (text != current).then_some(text)
}
