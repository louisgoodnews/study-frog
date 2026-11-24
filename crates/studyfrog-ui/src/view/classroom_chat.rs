//! The lecture chat (TASK-0293): role-badged turns from the teacher, simulated students and the user,
//! the lecture controls, and the composer for questions and comments.

use eframe::egui::{self, Button, Color32, Key, RichText, TextEdit};
use studyfrog_core::classroom::{ChatTurn, ClassroomSession, Participant};
use studyfrog_core::classroom_ops::participant_name;

use crate::classroom_types::ClassroomMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{Palette, palette_of, tint};

fn send(out: &mut Vec<Msg>, m: ClassroomMsg) {
    out.push(Msg::Classroom(m));
}

/// The chat of the session on screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(session) = state.classroom.session.as_ref() else {
        return;
    };
    let busy = state.classroom.busy;
    ui.horizontal(|ui| {
        ui.label(RichText::new(&session.title).strong());
        let leave = ui
            .add_enabled(!busy, Button::new("Back to setup").small())
            .on_hover_text("Leaves this session. It stays saved in the vault.");
        if leave.clicked() {
            send(out, ClassroomMsg::Leave);
        }
    });
    if let Some(error) = &state.classroom.error {
        ui.label(RichText::new(error).color(palette_of(ui).warning));
    }
    transcript(state, session, ui);
    ui.add_space(6.0);
    controls(session, busy, ui, out);
    ui.add_space(6.0);
    composer(&state.classroom.input, busy, ui, out);
}

fn transcript(state: &State, session: &ClassroomSession, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    let height = (ui.available_height() - 170.0).max(160.0);
    egui::ScrollArea::vertical()
        .id_salt("classroom-transcript")
        .max_height(height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if session.turns.is_empty() {
                ui.weak("The lecture has not started. Continue the lecture to hear the teacher.");
            }
            for turn in &session.turns {
                bubble(ui, &pal, session, turn);
            }
            if state.classroom.busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.weak("Waiting for the model…");
                });
            }
        });
}

/// One turn: a badge for the role, the speaker's name and the text.
fn bubble(ui: &mut egui::Ui, pal: &Palette, session: &ClassroomSession, turn: &ChatTurn) {
    let who = session.participants.get(turn.speaker);
    let role = role_style(who, pal);
    let name = who.map_or("Unknown", participant_name);
    egui::Frame::new()
        .fill(role.fill)
        .corner_radius(6)
        .inner_margin(8)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                badge(ui, &role);
                ui.label(RichText::new(name).small().strong().color(role.color));
            });
            ui.add_space(2.0);
            ui.add(egui::Label::new(&turn.text).wrap());
        });
    ui.add_space(6.0);
}

/// How one kind of speaker looks: the badge text and its accessible name, the colour, the bubble fill.
struct RoleStyle {
    text: &'static str,
    access: &'static str,
    color: Color32,
    fill: Color32,
}

fn role_style(who: Option<&Participant>, pal: &Palette) -> RoleStyle {
    let (text, access, color, fill) = match who {
        Some(Participant::Teacher { .. }) => ("Teacher", "Teacher role", pal.accent, pal.surface),
        Some(Participant::SimulatedStudent { .. }) => {
            ("Student", "Student role", pal.alt, pal.surface)
        }
        Some(Participant::User { .. }) => ("User", "User role", pal.success, tint(pal.success, 18)),
        None => ("Unknown", "Unknown role", pal.text_weak, pal.surface),
    };
    RoleStyle {
        text,
        access,
        color,
        fill,
    }
}

/// The role badge: a tinted tag whose text is also its accessible name.
fn badge(ui: &mut egui::Ui, role: &RoleStyle) {
    let tag = egui::Frame::new()
        .fill(tint(role.color, 40))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(role.text).small().strong().color(role.color))
        })
        .inner;
    tag.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, role.access));
}

/// The lecture controls: continue, ask a simulated student, set a task.
fn controls(session: &ClassroomSession, busy: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(!busy, Button::new("Continue lecture"))
            .on_hover_text("The teacher explains the next part of the material.")
            .clicked()
        {
            send(out, ClassroomMsg::Advance);
        }
        for (index, p) in session.participants.iter().enumerate() {
            if let Participant::SimulatedStudent { name } = p {
                let label = format!("Ask {name}");
                if ui
                    .add_enabled(!busy, Button::new(label))
                    .on_hover_text("This student asks the teacher a question.")
                    .clicked()
                {
                    send(out, ClassroomMsg::AskStudent(index));
                }
            }
        }
        if ui
            .add_enabled(!busy, Button::new("Assign task"))
            .on_hover_text("The teacher sets one task for you.")
            .clicked()
        {
            send(out, ClassroomMsg::AssignTask);
        }
    });
    for a in &session.assignments {
        ui.weak(format!("Task set: {}", a.title));
    }
}

/// The composer: a question or comment for the teacher, sent as the user.
fn composer(input: &str, busy: bool, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let mut text = input.to_owned();
    ui.horizontal(|ui| {
        let width = (ui.available_width() - 80.0).max(120.0);
        let resp = ui.add(
            TextEdit::singleline(&mut text)
                .desired_width(width)
                .hint_text("Ask the teacher or comment…"),
        );
        resp.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Message to the teacher")
        });
        if resp.changed() {
            send(out, ClassroomMsg::Input(text.clone()));
        }
        let can_send = !busy && !text.trim().is_empty();
        let enter = resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
        let clicked = ui
            .add_enabled(can_send, Button::new("Send"))
            .on_hover_text("Sends your message to the teacher.")
            .clicked();
        if can_send && (clicked || enter) {
            send(out, ClassroomMsg::Say);
        }
    });
}
