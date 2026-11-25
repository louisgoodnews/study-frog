//! The setup wizard's model download (STORY-0212): the MiniLM plan the wizard shows before its click, and the fetch
//! that click starts. The network library is linked here, not in the UI crate.

use std::path::Path;

use studyfrog_net::model_fetch::fetch_confirmed;
use studyfrog_net::model_plan::minilm_plan;
use studyfrog_ui::host::{ModelHost, PlanView};

/// What this binary offers the setup wizard.
pub fn model_host() -> ModelHost {
    ModelHost {
        minilm_plan: plan_view,
        fetch_minilm: fetch,
    }
}

/// The MiniLM plan in the form the wizard shows: title, source host and each file with its expected size.
fn plan_view() -> PlanView {
    let plan = minilm_plan();
    PlanView {
        title: plan.title,
        source_host: plan.source_host,
        files: plan
            .files
            .into_iter()
            .map(|f| (f.name, f.expected_bytes))
            .collect(),
    }
}

/// Downloads MiniLM into `dir` when `confirmed` is true (the user's click). Each file is checked against its digest.
fn fetch(dir: &Path, confirmed: bool) -> Result<(), String> {
    fetch_confirmed(&minilm_plan(), dir, |_| confirmed, |_, _| {})
}
