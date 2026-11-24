//! The Classroom screen (TASK-0293): the session form until a session is started, then the lecture chat.

use eframe::egui;

use crate::msg::Msg;
use crate::state::State;

/// Draw the form or the chat, whichever the state calls for.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    match state.classroom.session {
        None => super::classroom_setup::draw(state, ui, out),
        Some(_) => super::classroom_chat::draw(state, ui, out),
    }
}
