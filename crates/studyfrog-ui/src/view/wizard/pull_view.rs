//! The suggested Ollama model on the local-model step (STORY-0212): what is downloaded, its size and where it comes from,
//! shown before the click; then the progress, the failure and the retry. Ollama checks each file's digest itself.

use eframe::egui;
use studyfrog_service::ollama::client::{DEFAULT_ADDRESS, OllamaState};
use studyfrog_service::ollama::pull::LIBRARY_HOST;

use crate::agent_types::ollama::Snapshot;
use crate::msg::Msg;
use crate::state::State;
use crate::view::wizard::pull::pick_of;
use crate::view::wizard::steps::size_text;
use crate::view::wizard::types::WizardMsg;

/// The download of the suggested model, shown above the Ollama panel.
pub fn draw(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    let Some(snap) = state.agent.ollama.snapshot.as_deref() else {
        ui.weak("The check of this computer runs first; the suggested model is known after it.");
        return;
    };
    let Some(pick) = pick_of(snap) else {
        ui.weak(no_pick_text(snap));
        return;
    };
    ui.strong(format!("Suggested model: {}", pick.name));
    ui.weak(pick.reason.as_str());
    let size = match pick.download_bytes {
        Some(bytes) => format!("about {}", size_text(bytes)),
        None => "size not known".to_owned(),
    };
    ui.weak(format!(
        "Size: {size}. Source: the Ollama service at {DEFAULT_ADDRESS} on this computer, which pulls the model from the Ollama model library ({LIBRARY_HOST})."
    ));
    ui.weak("Ollama checks each downloaded file against its digest; the wizard does not check Ollama models itself.");
    ui.add_space(6.0);
    if state.wizard.pulling {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Downloading through Ollama. Ollama reports progress only when the download has finished, so this can take a long time.");
        });
        return;
    }
    if let Some(e) = &state.wizard.pull_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("The download failed: {e}"),
        );
    }
    let label = if state.wizard.pull_error.is_some() {
        "Try again"
    } else {
        "Download"
    };
    if ui
        .button(format!("{label} ({size}, {LIBRARY_HOST})"))
        .on_hover_text("Starts the download through the local Ollama service. Nothing is contacted before this click.")
        .clicked()
    {
        out.push(Msg::Wizard(WizardMsg::ConfirmPull));
    }
}

/// Why no model is offered: Ollama does not answer, or nothing that is not installed fits this computer.
fn no_pick_text(snap: &Snapshot) -> &'static str {
    match snap.status.state {
        OllamaState::Ready | OllamaState::NoModels => {
            "No suggested model that is not installed yet fits this computer's free memory, or that memory is not known. Nothing is downloaded."
        }
        _ => {
            "Ollama does not answer on this computer, so no model can be downloaded here. Start Ollama, then check again."
        }
    }
}
