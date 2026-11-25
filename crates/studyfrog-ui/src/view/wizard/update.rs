//! The wizard's messages (STORY-0212, TASK-0299): opening it, moving between steps, the MiniLM download after its
//! click, and closing it. Pure: probes and downloads are effects, and the executor answers with a message.

use studyfrog_config::wizard::{self as flow, Step, Wizard};

use crate::agent_types::ollama::OllamaMsg;
use crate::effect::{Effect, EffectAssistant};
use crate::state::State;
use crate::view::wizard::installed::installed_of;
use crate::view::wizard::types::{Probe, WizardEffect, WizardMsg, flow_of, name_of};

/// Advance the wizard by one message.
pub fn update(state: State, msg: WizardMsg) -> (State, Vec<Effect>) {
    match msg {
        WizardMsg::Start => {
            studyfrog_log::log_info!("ui.wizard", "setup wizard reopened from Settings");
            begin(state)
        }
        WizardMsg::Next => move_by(state, flow::next, "next"),
        WizardMsg::Previous => move_by(state, flow::previous, "previous"),
        WizardMsg::Skip => skip(state),
        WizardMsg::Goto(step) => goto(state, step),
        WizardMsg::Finish => finish(state),
        WizardMsg::ConfirmMinilm => confirm_minilm(state),
        WizardMsg::Probed(probe) => probed(state, probe),
        WizardMsg::Fetched(result) => fetched(state, result),
    }
}

/// The wizard after the first vault is chosen: the vault was the only thing remembered before (STORY-0212).
pub fn first_boot(state: State) -> (State, Vec<Effect>) {
    studyfrog_log::log_info!(
        "ui.wizard",
        "first boot: the first vault is chosen, the setup wizard starts"
    );
    begin(state)
}

/// Open the wizard at its first optional step. The checks start here (the MiniLM files, the Ollama service and the
/// saved connections), so the steps show what is really set up.
pub fn begin(mut state: State) -> (State, Vec<Effect>) {
    state.wizard.open = true;
    state.wizard.flow = Some(flow::start());
    studyfrog_log::log_info!("ui.wizard", "setup wizard opened");
    studyfrog_debug::checkpoint("ui.wizard", "opened");
    let (state, entered_effects) = entered(state, Step::LocalModel);
    let mut effects = vec![
        Effect::Wizard(WizardEffect::Probe),
        Effect::Assistant(EffectAssistant::LoadAgent),
    ];
    effects.extend(entered_effects);
    (state, effects)
}

/// The flow while the wizard is open; `None` when it is closed, so a message then changes nothing.
fn open_flow(state: &State) -> Option<Wizard> {
    state.wizard.open.then(|| flow_of(&state.wizard))
}

/// What a step needs when the user comes to it: the Ollama check on the local-model step, the MiniLM probe on its own.
fn entered(state: State, step: Step) -> (State, Vec<Effect>) {
    match step {
        Step::LocalModel if !state.agent.ollama.busy => {
            crate::agent_types::ollama::update(state, OllamaMsg::Refresh)
        }
        Step::Minilm => (state, vec![Effect::Wizard(WizardEffect::Probe)]),
        _ => (state, Vec::new()),
    }
}

fn move_by(state: State, by: fn(&Wizard) -> Wizard, how: &str) -> (State, Vec<Effect>) {
    let Some(before) = open_flow(&state) else {
        return (state, Vec::new());
    };
    note_leaving(&state, &before, how);
    let next = by(&before);
    set_flow(state, next, how)
}

fn skip(state: State) -> (State, Vec<Effect>) {
    let Some(before) = open_flow(&state) else {
        return (state, Vec::new());
    };
    let step = flow::current(&before);
    if matches!(step, Step::Vault | Step::Summary) {
        return (state, Vec::new());
    }
    note_leaving(&state, &before, "skip");
    studyfrog_log::log_info!("ui.wizard", "step {} skipped", name_of(step));
    let next = flow::skip(&before);
    set_flow(state, next, "skip")
}

fn goto(state: State, step: Step) -> (State, Vec<Effect>) {
    let Some(before) = open_flow(&state) else {
        return (state, Vec::new());
    };
    note_leaving(&state, &before, "goto");
    let next = flow::goto(&before, step);
    set_flow(state, next, "goto")
}

fn finish(mut state: State) -> (State, Vec<Effect>) {
    let Some(before) = open_flow(&state) else {
        return (state, Vec::new());
    };
    note_leaving(&state, &before, "finish");
    studyfrog_log::log_info!(
        "ui.wizard",
        "setup wizard closed at step {}",
        name_of(flow::current(&before))
    );
    state.wizard.open = false;
    state.wizard.flow = None;
    (state, Vec::new())
}

/// The user leaves the MiniLM step without a download: that is a decline, and it is logged as one.
fn note_leaving(state: &State, before: &Wizard, how: &str) {
    let step = flow::current(before);
    if step == Step::Minilm && !state.wizard.minilm_installed {
        studyfrog_log::log_info!("ui.wizard", "MiniLM download declined ({})", how);
    }
    studyfrog_debug::check(
        "ui.wizard",
        !(step == Step::Minilm && state.wizard.downloading),
        "left the MiniLM step while its download runs",
    );
}

fn set_flow(mut state: State, next: Wizard, how: &str) -> (State, Vec<Effect>) {
    let step = flow::current(&next);
    studyfrog_log::log_info!("ui.wizard", "step is now {} ({})", name_of(step), how);
    studyfrog_debug::count("ui.wizard.step_changed");
    state.wizard.flow = Some(next);
    entered(state, step)
}

fn confirm_minilm(mut state: State) -> (State, Vec<Effect>) {
    let w = &state.wizard;
    if !w.open || w.downloading || w.minilm_installed || w.plan.is_none() {
        return (state, Vec::new());
    }
    state.wizard.downloading = true;
    state.wizard.download_error = None;
    studyfrog_log::log_info!("ui.wizard", "MiniLM download confirmed by the user");
    (state, vec![Effect::Wizard(WizardEffect::FetchMinilm)])
}

fn probed(mut state: State, probe: Probe) -> (State, Vec<Effect>) {
    state.wizard.minilm_installed = probe.minilm_installed;
    state.wizard.plan = probe.plan;
    studyfrog_debug::dump("ui.wizard", "installed", &installed_of(&state));
    (state, Vec::new())
}

fn fetched(mut state: State, result: Result<(), String>) -> (State, Vec<Effect>) {
    state.wizard.downloading = false;
    match result {
        Ok(()) => {
            studyfrog_log::log_info!("ui.wizard", "MiniLM download finished and its files checked");
            state.wizard.download_error = None;
            (state, vec![Effect::Wizard(WizardEffect::Probe)])
        }
        Err(e) => {
            studyfrog_log::log_warn!("ui.wizard", "MiniLM download failed: {}", e);
            state.wizard.download_error = Some(e);
            (state, Vec::new())
        }
    }
}
