//! The setup wizard's effects (STORY-0212): the probe of the MiniLM files and plan, and the MiniLM download, which
//! is sent only after the user's click. The network library belongs to the application; the host passes the fetch in.

use studyfrog_slm::models::{MODEL_FILES, MODEL_NAME, missing_files};

use studyfrog_service::ollama::pull::{pull_outcome, pull_request};

use crate::exec::Worker;
use crate::msg::Msg;
use crate::view::wizard::types::{Probe, WizardEffect, WizardMsg};

/// Perform one wizard effect; the answer is a wizard message.
pub fn run(w: &Worker, effect: WizardEffect) -> Vec<Msg> {
    let msg = match effect {
        WizardEffect::Probe => probe(w),
        WizardEffect::FetchMinilm => fetch(w),
        WizardEffect::PullModel(name) => pull(w, &name),
    };
    vec![Msg::Wizard(msg)]
}

fn probe(w: &Worker) -> WizardMsg {
    let minilm_dir = w.models_dir.as_ref().map(|dir| dir.join(MODEL_NAME));
    let minilm_installed = minilm_dir
        .as_deref()
        .is_some_and(|dir| missing_files(dir).is_empty());
    let plan = w.model_host.map(|host| (host.minilm_plan)());
    studyfrog_debug::check(
        "ui.wizard",
        plan.as_ref()
            .is_none_or(|p| p.files.len() == MODEL_FILES.len()),
        "the MiniLM plan lists other files than the loader reads",
    );
    studyfrog_log::log_debug!("ui.wizard", "probe: MiniLM installed={}", minilm_installed);
    WizardMsg::Probed(Probe {
        minilm_installed,
        plan,
    })
}

fn fetch(w: &Worker) -> WizardMsg {
    let _timer = studyfrog_debug::timed("ui.wizard", "minilm download");
    let result = match (w.model_host, w.models_dir.as_ref()) {
        (Some(host), Some(dir)) => (host.fetch_minilm)(&dir.join(MODEL_NAME), true),
        (None, _) => Err("this build cannot download models".to_owned()),
        (Some(_), None) => Err("no folder for models is known".to_owned()),
    };
    studyfrog_log::log_info!("ui.wizard", "MiniLM download ended, ok={}", result.is_ok());
    WizardMsg::Fetched(result)
}

/// Ask the local Ollama service to pull `name` (one request; the answer comes when it has finished).
fn pull(w: &Worker, name: &str) -> WizardMsg {
    let _timer = studyfrog_debug::timed("ui.wizard", "model download");
    let result = match w.transport {
        Some(transport) => transport(&pull_request(name)).and_then(|answer| pull_outcome(&answer)),
        None => Err("This build cannot reach the network.".to_owned()),
    };
    studyfrog_log::log_info!("ui.wizard", "model download ended, ok={}", result.is_ok());
    WizardMsg::Pulled(result)
}
