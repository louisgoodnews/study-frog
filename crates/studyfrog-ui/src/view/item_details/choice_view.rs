//! The answers of a choice question in the Details column (STORY-0224): every option, the correct ones marked.
//! Read only; the editor's option list is the place to change them.

use eframe::egui::{self, RichText};

use crate::editor::Editor;
use crate::theme::palette_of;
use crate::view::choice_body::{answer_type_label, block_of, rows_of};

/// Draws the options of the prompt when it holds a choice; draws nothing otherwise.
pub fn draw(e: &Editor, ui: &mut egui::Ui) {
    let Some(first) = e.form.fields.first() else {
        return;
    };
    let Some(block) = block_of(&first.text) else {
        return;
    };
    let rows = rows_of(&block);
    studyfrog_log::log_debug!(
        "ui.choice",
        "details read a choice body with {} option(s)",
        rows.len()
    );
    studyfrog_debug::check(
        "ui.choice",
        rows.len() >= 2,
        "choice body has at least two options",
    );
    let p = palette_of(ui);
    ui.label(RichText::new(answer_type_label(e.form.answer_type)).strong());
    for row in &rows {
        ui.horizontal_wrapped(|ui| {
            let (mark, color) = if row.correct {
                ("correct", p.success)
            } else {
                ("wrong", p.text_weak)
            };
            ui.label(RichText::new(mark).color(color));
            ui.label(RichText::new(&row.text).color(if row.correct {
                p.text
            } else {
                p.text_weak
            }));
        });
    }
}
