//! Simple conversational chat with the connected assistant model.

use eframe::egui::{self, Key, RichText, TextEdit};
use studyfrog_agent::provider::Role;

use crate::agent_types::{AgentMsg, AgentTab, selected_provider};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::{palette_of, tint};
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: AgentMsg) {
    out.push(Msg::Agent(m));
}

/// The chat interface tab.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    let pal = palette_of(ui);
    let weak = pal.text_weak;

    let provider = selected_provider(a);
    if provider.is_none() {
        kit::card(ui, |ui| {
            ui.label(RichText::new("No model is currently selected.").strong());
            ui.label(
                RichText::new(
                    "Connect a local model (Ollama, LM Studio) or remote provider (OpenAI, Claude) to chat.",
                )
                .color(weak),
            );
            ui.add_space(4.0);
            if ui.button("Go to Connections…").clicked() {
                send(out, AgentMsg::Tab(AgentTab::Connections));
            }
        });
        return;
    }

    // Header info: Active provider name
    if let Some(p) = &provider {
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Chatting with {}", p.name)).strong());
            kit::pill(ui, &p.model, weak);
            if !a.chat_messages.is_empty() && ui.small_button("Clear chat").clicked() {
                send(out, AgentMsg::ChatClear);
            }
        });
    }

    ui.add_space(6.0);

    // Messages history area
    let scroll_height = (ui.available_height() - 90.0).max(180.0);
    egui::ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if a.chat_messages.is_empty() {
                ui.add_space(20.0);
                ui.weak("Ask questions about your study material, request explanations, or brainstorm study strategies.");
            }
            for m in &a.chat_messages {
                let is_user = m.role == Role::User;
                let fill = if is_user {
                    tint(pal.accent, 20)
                } else {
                    pal.surface
                };
                egui::Frame::new()
                    .fill(fill)
                    .corner_radius(6)
                    .inner_margin(8)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let name = if is_user { "You" } else { "Assistant" };
                        let name_color = if is_user { pal.accent } else { weak };
                        ui.label(RichText::new(name).small().strong().color(name_color));
                        ui.add_space(2.0);
                        ui.add(egui::Label::new(&m.content).wrap());
                    });
                ui.add_space(6.0);
            }
            if a.chat_busy {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.weak("Thinking…");
                });
            }
        });

    ui.add_space(8.0);

    // Input bar
    let mut text = a.chat_input.clone();
    let can_send = !a.chat_busy && !text.trim().is_empty();

    ui.horizontal(|ui| {
        let input_width = (ui.available_width() - 80.0).max(120.0);
        let resp = ui.add(
            TextEdit::singleline(&mut text)
                .desired_width(input_width)
                .hint_text("Type your message…"),
        );
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, "Chat message"));
        if resp.changed() {
            send(out, AgentMsg::ChatInput(text.clone()));
        }
        let enter = ui.input(|i| i.key_pressed(Key::Enter));
        let send_clicked = ui
            .add_enabled(can_send, egui::Button::new("Send"))
            .clicked();
        if can_send && (send_clicked || (resp.lost_focus() && enter)) {
            send(out, AgentMsg::ChatSend);
        }
    });
}
