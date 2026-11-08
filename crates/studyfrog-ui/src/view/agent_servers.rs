//! The assistant's external tool servers on the Connections tab (STORY-0121): each registered
//! server with its switches and what it offers, and the form for a new one.

use eframe::egui::{self, RichText};
use studyfrog_agent::mcp_client::{Discovered, McpServer, McpTransport, server_is_local};
use studyfrog_agent::provider::host_of;

use crate::agent_mcp_types::McpMsg;
use crate::agent_types::AgentMsg;
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::widgets::text_field;

fn send(out: &mut Vec<Msg>, m: McpMsg) {
    out.push(Msg::Agent(AgentMsg::Mcp(m)));
}

fn where_it_is(s: &McpServer) -> String {
    match &s.transport {
        McpTransport::Stdio { command, args } => format!("runs {command} {}", args.join(" ")),
        McpTransport::Http { url } => url.clone(),
    }
}

fn offered(d: &Discovered, ui: &mut egui::Ui) {
    let weak = palette_of(ui).text_weak;
    ui.label(
        RichText::new(format!(
            "{}: {} tools, {} resources, {} prompts",
            d.info,
            d.tools.len(),
            d.resources.len(),
            d.prompts.len()
        ))
        .color(weak),
    );
    for t in &d.tools {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new(&t.name).strong());
            let badge = match (t.read_only, t.destructive) {
                (true, _) => "read-only",
                (false, true) => "may delete",
                (false, false) => "changes things",
            };
            kit::pill(ui, badge, weak);
            ui.label(RichText::new(&t.description).color(weak));
        });
    }
    for r in &d.resources {
        ui.label(RichText::new(format!("Resource: {r}")).color(weak));
    }
    for p in &d.prompts {
        ui.label(RichText::new(format!("Prompt: {p}")).color(weak));
    }
}

fn server_card(state: &State, s: &McpServer, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let weak = palette_of(ui).text_weak;
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(&s.name).strong());
            ui.label(RichText::new(where_it_is(s)).color(weak));
        });
        ui.horizontal_wrapped(|ui| {
            let mut on = s.enabled;
            if ui.checkbox(&mut on, format!("Use {}", s.name)).changed() {
                send(out, McpMsg::Enable(s.name.clone(), on));
            }
            let mut trust = s.trust_read_only;
            if ui
                .checkbox(
                    &mut trust,
                    format!("Run read-only tools of {} without asking", s.name),
                )
                .changed()
            {
                send(out, McpMsg::Trust(s.name.clone(), trust));
            }
            if let (false, McpTransport::Http { url }) = (server_is_local(s), &s.transport) {
                let mut allowed = s.allowed;
                if ui
                    .checkbox(
                        &mut allowed,
                        format!("Allow sending text to {}", host_of(url)),
                    )
                    .changed()
                {
                    send(out, McpMsg::Allow(s.name.clone(), allowed));
                }
            }
        });
        ui.horizontal(|ui| {
            let can = !state.agent.busy;
            if ui
                .add_enabled(can, egui::Button::new(format!("Discover {}", s.name)))
                .clicked()
            {
                send(out, McpMsg::Discover(s.name.clone()));
            }
            if ui.small_button(format!("Remove {}", s.name)).clicked() {
                send(out, McpMsg::RemoveServer(s.name.clone()));
            }
        });
        if let Some(d) = state.agent.mcp.found.iter().find(|d| d.server == s.name) {
            offered(d, ui);
        }
    });
}

/// The section; always shown, so a first server can be added.
pub fn servers(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let m = &state.agent.mcp;
    ui.add_space(8.0);
    kit::section(ui, "Tool servers (MCP)");
    ui.label(RichText::new("Let the assistant use tools of other MCP servers, such as a literature search or a citation manager. A tool that may change something always asks you first.").color(palette_of(ui).text_weak));
    for s in &m.servers {
        server_card(state, s, ui, out);
        ui.add_space(6.0);
    }
    kit::group(ui, "New tool server", |ui| {
        if let Some(t) = text_field(ui, "Server name", &m.form.name, "e.g. zotero", 1) {
            send(out, McpMsg::FormName(t));
        }
        let hint = "a program such as npx -y some-mcp-server, or an http:// address";
        if let Some(t) = text_field(ui, "Program or address", &m.form.line, hint, 1) {
            send(out, McpMsg::FormLine(t));
        }
        if kit::primary_button(ui, "Add tool server").clicked() {
            send(out, McpMsg::AddServer);
        }
    });
}

/// The card for a tool call that waits for the user's go-ahead; nothing when none waits.
pub fn approval(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(p) = &state.agent.mcp.approval else {
        return;
    };
    kit::card(ui, |ui| {
        ui.set_width(ui.available_width());
        let ask = format!(
            "Allow Assistant to run tool {} on server {}?",
            p.call.tool, p.call.server
        );
        ui.label(RichText::new(ask).strong());
        let args = serde_json::to_string_pretty(&p.call.arguments).unwrap_or_default();
        ui.label(RichText::new(args).monospace());
        ui.horizontal(|ui| {
            if kit::primary_button(ui, "Allow this call").clicked() {
                send(out, McpMsg::Approve);
            }
            if ui.button("Do not allow").clicked() {
                send(out, McpMsg::Deny);
            }
        });
    });
    ui.add_space(8.0);
}
