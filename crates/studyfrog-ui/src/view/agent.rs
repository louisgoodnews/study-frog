//! The assistant screen: tabs, the stacks it may read, and the connections.

use eframe::egui::{self, RichText};
use studyfrog_agent::provider::{PROTOCOLS, default_models, host_of, is_local, protocol_name};

use crate::agent_types::{AgentMsg, AgentTab, ConnForm, FormEdit};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::widgets::text_field;
use crate::view::{agent_wiki, kit};

fn send(out: &mut Vec<Msg>, m: AgentMsg) {
    out.push(Msg::Agent(m));
}

/// The whole screen.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(
                RichText::new("Assistant")
                    .size(crate::typo::size(ui, crate::typo::Tier::Heading))
                    .strong(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (AgentTab::Connections, "Connections"),
                    (AgentTab::Wiki, "Work the vault"),
                    (AgentTab::Exam, "Exam me"),
                    (AgentTab::Prompts, "Prompts"),
                ] {
                    if ui.selectable_label(a.tab == tab, label).clicked() {
                        send(out, AgentMsg::Tab(tab));
                    }
                }
            });
            if let Some(e) = &a.error {
                ui.colored_label(ui.visuals().error_fg_color, e);
            }
            ui.add_space(8.0);
            kit::column(ui, 860.0, |ui| {
                crate::view::agent_servers::approval(state, ui, out)
            });
            kit::column(ui, 860.0, |ui| match a.tab {
                AgentTab::Connections => connections(state, ui, out),
                AgentTab::Wiki => agent_wiki::wiki(state, ui, out),
                AgentTab::Exam => crate::view::agent_exam::exam(state, ui, out),
                AgentTab::Prompts => crate::view::agent_prompts::moved_link(ui, out),
            });
        });
}

/// The models that were connected, and the form for a new one.
fn connections(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let a = &state.agent;
    let weak = palette_of(ui).text_weak;
    ui.label(RichText::new("Connect a model that runs on this computer (Ollama and similar) or at a provider (OpenAI, Anthropic and similar). The assistant only proposes changes; you decide what is applied. API keys are kept in a private file in your config folder, never in the vault.").color(weak));
    if let Some(e) = &a.connections.error {
        ui.colored_label(ui.visuals().error_fg_color, e);
    }
    for p in &a.connections.providers {
        kit::card(ui, |ui| {
            ui.set_width(ui.available_width());
            let in_use = a.selected.as_deref() == Some(p.name.as_str());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&p.name).strong());
                kit::pill(ui, protocol_name(p.protocol), weak);
                ui.label(
                    RichText::new(format!("{} · {}", p.model, host_of(&p.base_url))).color(weak),
                );
            });
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(in_use, format!("Use {}", p.name))
                    .clicked()
                {
                    send(out, AgentMsg::Select(p.name.clone()));
                }
                if is_local(p) {
                    ui.label(RichText::new("runs on this computer").color(weak));
                } else {
                    let mut allowed = p.allowed;
                    let text = format!(
                        "Allow sending text from my vault to {}",
                        host_of(&p.base_url)
                    );
                    if ui.checkbox(&mut allowed, text).changed() {
                        send(out, AgentMsg::Allow(p.name.clone(), allowed));
                    }
                    let key = if a.connections.with_key.contains(&p.name) {
                        "API key saved"
                    } else {
                        "no API key"
                    };
                    ui.label(RichText::new(key).color(weak));
                }
                crate::view::agent_conn_test::button(a, p, ui, out);
                if ui.small_button(format!("Remove {}", p.name)).clicked() {
                    send(out, AgentMsg::RemoveConnection(p.name.clone()));
                }
            });
            crate::view::agent_conn_test::outcome(a, p, ui);
        });
        ui.add_space(8.0);
    }
    crate::view::agent_mcp::snippet(state, ui);
    new_connection(state, ui, out);
    crate::view::agent_servers::servers(state, ui, out);
}

/// One button per model the chosen protocol suggests; a click fills in the Model field.
fn suggested_models(f: &ConnForm, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let models = default_models(f.protocol);
    if models.is_empty() {
        return;
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Suggested models:");
        for model in models {
            if ui.selectable_label(f.model == *model, *model).clicked() {
                send(out, AgentMsg::Form(FormEdit::Model((*model).to_owned())));
            }
        }
    });
}

fn new_connection(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let f = &state.agent.form;
    ui.add_space(8.0);
    kit::group(ui, "New connection", |ui| {
        ui.horizontal(|ui| {
            for p in PROTOCOLS {
                if ui
                    .selectable_label(f.protocol == p, protocol_name(p))
                    .clicked()
                {
                    send(out, AgentMsg::Form(FormEdit::Protocol(p)));
                }
            }
        });
        for (label, value, hint, make) in [
            (
                "Connection name",
                &f.name,
                "e.g. Local Llama",
                FormEdit::Name as fn(String) -> FormEdit,
            ),
            ("Address", &f.base_url, "", FormEdit::BaseUrl),
            (
                "Model",
                &f.model,
                "e.g. llama3.2 or gpt-4o",
                FormEdit::Model,
            ),
        ] {
            if let Some(t) = text_field(ui, label, value, hint, 1) {
                send(out, AgentMsg::Form(make(t)));
            }
        }
        suggested_models(f, ui, out);
        let name = ui.label("API key (not needed for a local model)");
        let mut key = f.key.clone();
        let edit = egui::TextEdit::singleline(&mut key)
            .password(true)
            .desired_width(f32::INFINITY);
        if ui.add(edit).labelled_by(name.id).changed() {
            send(out, AgentMsg::Form(FormEdit::Key(key)));
        }
        ui.add_space(4.0);
        if kit::primary_button(ui, "Add connection").clicked() {
            send(out, AgentMsg::AddConnection);
        }
    });
}
