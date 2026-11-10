//! In-rehearsal edit dialog: quick edits for prompt, answer, title, and tags.

use eframe::egui::{self, Align, Layout, RichText};
use studyfrog_service::create_more::NewKind;

use crate::msg::Msg;
use crate::rehearsal_edit::{EditCardModal, RehearsalEditMsg};
use crate::state::State;
use crate::view::icons::{Glyph, icon_button};
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: RehearsalEditMsg) {
    out.push(Msg::RehearsalEdit(m));
}

pub fn draw(_state: &State, f: &EditCardModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let save_pressed = ui.input(|i| {
        (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::Enter)
    });
    if save_pressed && !f.busy && !f.title.trim().is_empty() {
        send(out, RehearsalEditMsg::Save);
    }

    header(f, ui, out);
    ui.add_space(8.0);

    if let Some(err) = &f.error {
        ui.colored_label(crate::theme::palette_of(ui).danger, err);
        ui.add_space(4.0);
    }

    if let Some(t) = text_field(ui, "Title", &f.title, "give it a title", 1) {
        send(out, RehearsalEditMsg::EditTitle(t));
    }
    ui.add_space(6.0);

    let prompt_label = match f.kind {
        NewKind::Card => "Front",
        NewKind::Question => "Prompt",
        _ => "Body",
    };
    if let Some(t) = text_field(ui, prompt_label, &f.text, "prompt text", 6) {
        send(out, RehearsalEditMsg::EditText(t));
    }
    ui.add_space(6.0);

    if f.kind == NewKind::Card || f.kind == NewKind::Question {
        let answer_label = if f.kind == NewKind::Card {
            "Back"
        } else {
            "Reference answer"
        };
        if let Some(a) = text_field(ui, answer_label, &f.answer, "answer text", 6) {
            send(out, RehearsalEditMsg::EditAnswer(a));
        }
        ui.add_space(6.0);
    }

    if let Some(t) = text_field(ui, "Tags", &f.tags, "comma, separated, tags", 1) {
        send(out, RehearsalEditMsg::EditTags(t));
    }
    ui.add_space(12.0);

    footer(f, ui, out);
}

fn header(f: &EditCardModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let heading = match f.kind {
        NewKind::Card => "Edit Card",
        NewKind::Question => "Edit Question",
        NewKind::Note => "Edit Note",
        NewKind::Stack => "Edit Stack",
    };
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(heading)
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong(),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if icon_button(ui, Glyph::Close, "Close", false).clicked() {
                send(out, RehearsalEditMsg::Close);
            }
            if ui.small_button("Open in full editor ↗").clicked() {
                let uid = f.uid;
                send(out, RehearsalEditMsg::Close);
                out.push(Msg::OpenItem(uid));
            }
        });
    });
}

fn footer(f: &EditCardModal, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal(|ui| {
        ui.weak("Ctrl+Enter to save · Esc to cancel");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let can_save = !f.busy && !f.title.trim().is_empty();
            let label = if f.busy { "Saving…" } else { "Save changes" };
            if ui
                .add_enabled(can_save, |ui: &mut egui::Ui| kit::primary_button(ui, label))
                .clicked()
            {
                send(out, RehearsalEditMsg::Save);
            }
            if ui.button("Cancel").clicked() {
                send(out, RehearsalEditMsg::Close);
            }
        });
    });
}
