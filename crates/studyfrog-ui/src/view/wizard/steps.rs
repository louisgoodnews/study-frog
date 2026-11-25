//! The body of each wizard step (STORY-0212, TASK-0299). The local-model step is the Ollama panel of STORY-0206.
//! The remote-provider step is opt-in and says that scored text leaves the machine.

use eframe::egui::{self, Color32, RichText};
use studyfrog_config::wizard::{self as flow, Installed, Status, Step, Wizard};

use crate::msg::{Msg, MsgVault};
use crate::state::{Screen, State};
use crate::theme::palette_of;
use crate::view::kit;
use crate::view::wizard::types::{WizardMsg, title_of};

/// The body of the step the wizard is on.
pub fn body(
    state: &State,
    current: &Wizard,
    installed: &Installed,
    ui: &mut egui::Ui,
    out: &mut Vec<Msg>,
) {
    match flow::current(current) {
        Step::LocalModel => local(state, ui, out),
        Step::RemoteProvider => remote(ui, out),
        Step::Minilm => minilm(state, ui, out),
        Step::Vault | Step::Summary => summary(current, installed, ui),
    }
}

fn local(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Local model");
    ui.label(
        RichText::new("Ollama runs a language model on this computer. Nothing is downloaded, installed or started until you press one of its buttons, and the models run only here.")
            .color(palette_of(ui).text_weak),
    );
    kit::section(ui, "Suggested model (optional)");
    crate::view::wizard::pull_view::draw(state, ui, out);
    ui.add_space(8.0);
    crate::view::agent::ollama::panel(state, ui, out);
}

fn remote(ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "Remote provider (optional)");
    ui.label(
        RichText::new("Scored text leaves this computer when a remote provider is set up: the answers and questions you send to it go to that provider's servers.")
            .color(palette_of(ui).warning),
    );
    ui.weak("Off by default. A provider is added on the Assistant screen, and nothing is sent before you add one and ask it something.");
    if ui
        .button("Open the Assistant's connections")
        .on_hover_text("Closes the setup and opens the Assistant screen, where a provider is added. Settings runs the setup again.")
        .clicked()
    {
        out.push(Msg::Wizard(WizardMsg::Finish));
        out.push(Msg::Vault(MsgVault::Navigate(Screen::Agent)));
    }
}

fn minilm(state: &State, ui: &mut egui::Ui, out: &mut Vec<Msg>) {
    kit::section(ui, "MiniLM embedding model");
    let w = &state.wizard;
    if w.minilm_installed {
        ui.label("Installed in the models folder. Answers are compared by meaning; there is nothing to download.");
        return;
    }
    ui.weak("Compares typed answers by meaning. Without it, answers are compared by their words.");
    let Some(plan) = &w.plan else {
        ui.weak("The download details are not available yet.");
        return;
    };
    ui.label(plan.title.as_str());
    let total: u64 = plan.files.iter().map(|(_, bytes)| *bytes).sum();
    ui.weak(format!(
        "From {}. {} in {} files. Each file is checked against its SHA-256 digest before it is used.",
        plan.source_host,
        size_text(total),
        plan.files.len()
    ));
    for (name, bytes) in &plan.files {
        ui.weak(format!("{name}: {}", size_text(*bytes)));
    }
    ui.add_space(6.0);
    if w.downloading {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Downloading. This can take a few minutes.");
        });
        return;
    }
    if let Some(e) = &w.download_error {
        ui.colored_label(
            ui.visuals().error_fg_color,
            format!("The download failed: {e}"),
        );
    }
    let label = if w.download_error.is_some() {
        "Try again"
    } else {
        "Download"
    };
    if ui
        .button(format!(
            "{label} ({}, {})",
            size_text(total),
            plan.source_host
        ))
        .on_hover_text("Starts the download. Nothing is contacted before this click.")
        .clicked()
    {
        out.push(Msg::Wizard(WizardMsg::ConfirmMinilm));
    }
}

/// What is set up and what was skipped, one row per step.
fn summary(current: &Wizard, installed: &Installed, ui: &mut egui::Ui) {
    kit::section(ui, "Summary");
    ui.weak("What is set up on this computer and what you skipped. Settings runs the setup again.");
    for step in [
        Step::Vault,
        Step::LocalModel,
        Step::RemoteProvider,
        Step::Minilm,
    ] {
        let status = flow::status(step, installed, current);
        ui.horizontal(|ui| {
            ui.label(title_of(step));
            let color = status_color(ui, status);
            kit::pill(ui, status_word(status), color);
        });
    }
}

fn status_word(status: Status) -> &'static str {
    match status {
        Status::Current => "Open now",
        Status::Done => "Set up",
        Status::Skipped => "Skipped",
        Status::Pending => "Not set up",
    }
}

fn status_color(ui: &egui::Ui, status: Status) -> Color32 {
    let pal = palette_of(ui);
    match status {
        Status::Done => pal.success,
        Status::Current => pal.accent,
        Status::Skipped | Status::Pending => pal.text_weak,
    }
}

/// A size as the models are listed, in megabytes with one decimal.
pub fn size_text(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}
