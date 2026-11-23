//! The local-model panel on the assistant's Connections tab (STORY-0206): check this computer and
//! Ollama, the status, the suggestions, the guide, and the test before a model is used.

mod audit;

use eframe::egui::{self, RichText};
use studyfrog_service::ollama::client::{OllamaState, OllamaStatus};

use self::audit as parts;
use crate::agent_types::AgentMsg;
use crate::agent_types::ollama::{LOCAL_NAME, OllamaMsg, Snapshot};
use crate::msg::Msg;
use crate::state::State;
use crate::theme::palette_of;
use crate::view::kit;

fn send(out: &mut Vec<Msg>, m: OllamaMsg) {
    out.push(Msg::Agent(AgentMsg::Ollama(m)));
}

/// The whole panel.
pub fn panel(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let p = &state.agent.ollama;
    kit::group(ui, "Set up a local model", |ui| {
        ui.label(
            RichText::new("Checks this computer and the Ollama service on this computer only. Nothing is downloaded, installed or started until you run a command yourself.")
                .color(palette_of(ui).text_weak),
        );
        ui.add_space(4.0);
        let label = if p.busy {
            "Checking..."
        } else {
            "Check this computer"
        };
        let check = ui.button(label).on_hover_text(
            "Reads this computer's hardware and asks the local Ollama service what it has.",
        );
        if check.clicked() && !p.busy {
            send(out, OllamaMsg::Refresh);
        }
        test_outcome(state, ui);
        match &p.snapshot {
            None => {
                ui.label("Not checked yet.");
            }
            Some(snap) => show(state, snap, ui, out),
        }
        if let Some(e) = &p.error {
            ui.colored_label(ui.visuals().error_fg_color, e);
        }
    });
}

fn show(state: &State, snap: &Snapshot, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    ui.add_space(6.0);
    status_line(&snap.status, ui);
    if let Some(note) = &snap.status.note {
        ui.label(RichText::new(note).color(palette_of(ui).text_weak));
    }
    if let Some(e) = &snap.catalog_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("The model list could not be read: {e}"),
        );
    }
    ui.add_space(6.0);
    kit::group(ui, "Setup steps", |ui| parts::guide(&snap.guide, ui));
    ui.add_space(6.0);
    kit::group(ui, "This computer", |ui| parts::audit(&snap.audit, ui));
    ui.add_space(6.0);
    kit::group(ui, "Models for this computer", |ui| {
        ui.label(
            RichText::new(
                "Ranked by fit: the first ones run best here. Sizes come from the catalog.",
            )
            .color(palette_of(ui).text_weak),
        );
        for s in &snap.suggestions {
            parts::suggestion_line(s, ui);
            use_button(state, &snap.status, &s.name, "", ui, out);
            ui.add_space(4.0);
        }
        if !snap.status.installed.is_empty() {
            ui.label(RichText::new("Installed in Ollama").strong());
            for i in &snap.status.installed {
                ui.horizontal(|ui| {
                    ui.label(format!("{} ({})", i.name, parts::bytes_text(i.size_bytes)));
                    use_button(state, &snap.status, &i.name, " (installed)", ui, out);
                });
            }
        }
    });
}

fn status_line(status: &OllamaStatus, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    let (pill, color, text) = match status.state {
        OllamaState::Missing => (
            "missing",
            pal.danger,
            "Ollama is not installed on this computer.".to_owned(),
        ),
        OllamaState::NotRunning => (
            "not running",
            pal.danger,
            format!("Ollama is installed but not running at {}.", status.address),
        ),
        OllamaState::NoModels => (
            "no models",
            pal.text_weak,
            format!(
                "Ollama {} is running and has no models yet.",
                version_of(status)
            ),
        ),
        OllamaState::Ready => (
            "ready",
            pal.success,
            format!(
                "Ollama {} is ready with {} model(s).",
                version_of(status),
                status.installed.len()
            ),
        ),
    };
    ui.horizontal_wrapped(|ui| {
        kit::pill(ui, pill, color);
        ui.label(text);
    });
}

fn version_of(status: &OllamaStatus) -> String {
    status
        .version
        .clone()
        .unwrap_or_else(|| "(version unknown)".to_owned())
}

/// The button that tests a model and makes it the local model; off while Ollama cannot be reached.
fn use_button(
    state: &State,
    status: &OllamaStatus,
    name: &str,
    suffix: &str,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    let testing = state.agent.ollama.testing.as_deref();
    let reachable = matches!(status.state, OllamaState::NoModels | OllamaState::Ready);
    let is_local = state.agent.selected.as_deref() == Some(LOCAL_NAME)
        && state
            .agent
            .connections
            .providers
            .iter()
            .any(|p| p.name == LOCAL_NAME && p.model == name);
    let busy = testing.is_some();
    let label = if is_local {
        "In use".to_owned()
    } else if testing == Some(name) {
        "Testing...".to_owned()
    } else {
        format!("Test and use {name}{suffix}")
    };
    let response = ui
        .add_enabled(reachable && !busy && !is_local, egui::Button::new(label).small())
        .on_disabled_hover_text("Start the Ollama service first; the test needs it.")
        .on_hover_text("Sends one short prompt to this model on this computer. If it answers, it becomes the local model.");
    if response.clicked() {
        send(out, OllamaMsg::Choose(name.to_owned()));
    }
}

fn test_outcome(state: &State, ui: &mut egui::Ui) {
    let pal = palette_of(ui);
    if let Some(name) = &state.agent.ollama.testing {
        ui.label(format!("Testing {name} with one short prompt..."));
    }
    match &state.agent.ollama.test {
        Some((name, Ok(text))) => {
            ui.label(RichText::new(format!("Test passed for {name}: {text}")).color(pal.success));
        }
        Some((name, Err(e))) => {
            ui.label(RichText::new(format!("The test of {name} failed: {e}")).color(pal.danger));
        }
        None => {}
    }
}
