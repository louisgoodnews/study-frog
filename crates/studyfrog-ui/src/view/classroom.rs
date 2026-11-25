//! The Classroom screen (TASK-0293, TASK-0294): the session form until a session is started, then the
//! lecture chat beside the assignments panel. Below the wide breakpoint the panel stacks under the chat.

use eframe::egui::{self, RichText};

use crate::msg::Msg;
use crate::state::State;
use crate::view::gradient_panel;

/// The width from which the chat and the assignments panel sit side by side.
const WIDE: f32 = 720.0;
const HEADLINE_HEIGHT: f32 = 64.0;

/// Draw the form or the chat, whichever the state calls for, under the gradient headline.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let _timer = studyfrog_debug::timed("ui", "classroom draw");
    studyfrog_log::log_debug!("ui", "classroom shown");
    headline(ui);
    ui.add_space(8.0);
    match state.classroom.session {
        None => setup_and_saved(state, ui, out),
        Some(_) => workspace(state, ui, out),
    }
}

/// The screen title on the gradient panel (EPIC-070).
fn headline(ui: &mut egui::Ui) {
    gradient_panel::panel(ui, HEADLINE_HEIGHT, |ui, s| {
        ui.label(
            RichText::new("Classroom")
                .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                .strong()
                .color(s.gradient_text),
        );
    });
}

/// The session form, then the saved sessions below it, in one scrolling column.
fn setup_and_saved(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    egui::ScrollArea::vertical()
        .id_salt("classroom-setup")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            super::classroom_setup::draw(state, ui, out);
            ui.add_space(12.0);
            super::classroom_saved::draw(state, ui, out);
        });
}

/// The chat and the panel side by side when there is room, else one scrolling column.
fn workspace(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    if ui.available_width() >= WIDE {
        ui.columns(2, |cols| {
            super::classroom_chat::draw(state, &mut cols[0], out);
            egui::ScrollArea::vertical()
                .id_salt("classroom-tasks")
                .auto_shrink([false, false])
                .show(&mut cols[1], |ui| {
                    super::classroom_tasks::draw(state, ui, out);
                });
        });
    } else {
        egui::ScrollArea::vertical()
            .id_salt("classroom-narrow")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                super::classroom_chat::draw(state, ui, out);
                ui.add_space(12.0);
                super::classroom_tasks::draw(state, ui, out);
            });
    }
}
