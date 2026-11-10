//! Comments drawer displayed during active rehearsal and exam runs (STORY-0140).

use eframe::egui::{self, Align, Layout, RichText};
use studyfrog_core::comment::ItemComment;
use studyfrog_rehearsal::session::current_item;

use crate::msg::Msg;
use crate::rehearsal_edit::RehearsalEditMsg;
use crate::run_state::RunView;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

/// Draw the expandable comments drawer below the active card.
pub fn draw(_state: &State, run: &RunView, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(uid) = current_item(&run.session) else {
        return;
    };
    let pal = palette_of(ui);
    let comments: &[ItemComment] = run
        .content
        .get(&uid)
        .map(|c| c.comments.as_slice())
        .unwrap_or(&[]);

    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("Card Comments").strong());
            kit::pill(ui, &format!("{}", comments.len()), pal.accent);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.small_button("Close (C)").clicked() {
                    out.push(Msg::RehearsalEdit(RehearsalEditMsg::ToggleComments));
                }
            });
        });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        if comments.is_empty() {
            ui.weak("No comments on this card yet.");
        } else {
            for c in comments {
                comment_row(c, ui);
                ui.add_space(4.0);
            }
        }

        ui.add_space(6.0);
        new_comment_box(run, uid, ui, out);
    });
}

fn comment_row(c: &ItemComment, ui: &mut egui::Ui) {
    kit::card(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&c.author)
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            ui.weak(format_time(c.created));
        });
        ui.add_space(2.0);
        ui.label(RichText::new(&c.text).color(ui.visuals().text_color()));
    });
}

fn format_time(ts: i64) -> String {
    let secs = (ts / 1000).max(0);
    let date = studyfrog_core::civil::date_from_days(secs / 86400);
    studyfrog_core::civil::format_date(date)
}

fn new_comment_box(
    run: &RunView,
    uid: studyfrog_core::uid::Uid,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let mut draft = run.comment_draft.clone();
    ui.horizontal(|ui| {
        let field = egui::TextEdit::singleline(&mut draft)
            .hint_text("Add reflection or note… (Enter)")
            .desired_width(ui.available_width() - 65.0);
        let resp = ui.add(field);
        if resp.changed() {
            out.push(Msg::RehearsalEdit(RehearsalEditMsg::DraftComment(
                draft.clone(),
            )));
        }
        let can_submit = !draft.trim().is_empty();
        let enter_pressed = resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let add_clicked = ui
            .add_enabled(can_submit, egui::Button::new("Add"))
            .clicked();

        if can_submit && (enter_pressed || add_clicked) {
            out.push(Msg::RehearsalEdit(RehearsalEditMsg::SubmitComment(uid)));
        }
    });
}
