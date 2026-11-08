//! The proposals AI clients staged over MCP and the changes they made that can be undone
//! (STORY-0119), on the assistant's "Work the vault" tab.

use eframe::egui::{self, RichText};

use crate::agent_mcp_types::McpMsg;
use crate::agent_types::AgentMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: McpMsg) {
    out.push(Msg::Agent(AgentMsg::Mcp(m)));
}

/// Both lists; nothing when both are empty.
pub fn staged(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let m = &state.agent.mcp;
    let weak = palette_of(ui).text_weak;
    if !m.staged.is_empty() {
        kit::section(ui, "Proposals from AI tools");
        ui.label(RichText::new("AI tools connected over MCP proposed these. Nothing is written until you review and apply a proposal.").color(weak));
    }
    for s in &m.staged {
        let p = &s.proposal;
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&p.summary).strong());
                kit::pill(ui, &p.tool, weak);
                ui.label(RichText::new(format!("{} changes", p.changes.len())).color(weak));
            });
            for why in &s.rejected {
                ui.label(RichText::new(why).color(weak));
            }
            ui.horizontal(|ui| {
                let under_review = m.reviewing.as_deref() == Some(p.id.as_str());
                if ui
                    .add_enabled(
                        !under_review,
                        egui::Button::new(format!("Review \"{}\"", p.summary)),
                    )
                    .clicked()
                {
                    send(out, McpMsg::Review(p.id.clone()));
                }
                if ui.button(format!("Discard \"{}\"", p.summary)).clicked() {
                    send(out, McpMsg::Discard(p.id.clone()));
                }
            });
        });
    }
    if !m.undo.is_empty() {
        kit::section(ui, "Changes by AI tools");
    }
    for (id, label, _) in &m.undo {
        ui.horizontal(|ui| {
            ui.label(label);
            if ui.small_button(format!("Undo: {label}")).clicked() {
                send(out, McpMsg::Undo(id.clone()));
            }
        });
    }
}
