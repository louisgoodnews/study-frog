//! The Classroom screen (TASK-0293, TASK-0294): the session form until a session is started, then the
//! lecture chat beside the assignments panel. Below the wide breakpoint the panel stacks under the chat.

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;

/// The width from which the chat and the assignments panel sit side by side.
const WIDE: f32 = 720.0;

/// Draw the form or the chat, whichever the state calls for.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    match state.classroom.session {
        None => super::classroom_setup::draw(state, ui, out),
        Some(_) => workspace(state, ui, out),
    }
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
