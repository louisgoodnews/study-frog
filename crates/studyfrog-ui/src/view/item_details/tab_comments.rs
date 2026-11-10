//! The Comments tab of the item details notebook: view and manage timestamped comments (STORY-0135).

use eframe::egui;
use studyfrog_core::comment::{ItemComment, new_comment};

use crate::editor::Editor;
use crate::msg::{EditAction, Msg};
use crate::state::State;
use crate::view::kit;

/// Draw the Comments tab.
pub fn draw(_state: &State, e: &Editor, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Comments");
    ui.add_space(4.0);

    if e.form.comments.is_empty() {
        ui.weak("No comments on this item yet.");
    } else {
        for c in &e.form.comments {
            comment_card(c, ui, out);
            ui.add_space(6.0);
        }
    }

    ui.add_space(10.0);
    new_comment_input(ui, out);
}

fn comment_card(c: &ItemComment, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&c.author)
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.weak(format_time(c.created));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .small_button("Delete")
                    .on_hover_text("Delete comment")
                    .clicked()
                {
                    out.push(Msg::Edit(EditAction::DeleteComment(c.id.clone())));
                }
            });
        });
        ui.add_space(4.0);
        ui.label(&c.text);
    });
}

fn format_time(ts: i64) -> String {
    let secs = (ts / 1000).max(0);
    let date = studyfrog_core::civil::date_from_days(secs / 86400);
    studyfrog_core::civil::format_date(date)
}

fn new_comment_input(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let id = ui.make_persistent_id("new_comment_draft");
    let mut draft: String = ui.data(|d| d.get_temp(id).unwrap_or_default());

    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut draft)
                .hint_text("Write a comment…")
                .desired_width(180.0),
        );
        let valid = !draft.trim().is_empty();
        if ui.add_enabled(valid, egui::Button::new("Add")).clicked() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            let cid = format!("c_{}", now);
            let comment = new_comment(cid, "user", now, draft.trim());
            out.push(Msg::Edit(EditAction::AddComment(comment)));
            draft.clear();
        }
    });

    ui.data_mut(|d| d.insert_temp(id, draft));
}
