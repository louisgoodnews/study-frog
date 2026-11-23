//! The answer to `reports`: the tab and the figures the screen shows for it. Pure.

use serde_json::{Value, json};
use studyfrog_service::reports::ReportBundle;

use crate::control_reply::report_tabs::figures;
use crate::explorer_ui::ReportTab;
use crate::state::State;

/// The name a tab has on the command line.
pub fn tab_name(tab: ReportTab) -> &'static str {
    match tab {
        ReportTab::Performance => "performance",
        ReportTab::Memory => "memory",
        ReportTab::Familiarity => "familiarity",
        ReportTab::Calibration => "calibration",
        ReportTab::Cognitive => "cognitive",
        ReportTab::Runs => "runs",
        ReportTab::Habits => "habits",
        ReportTab::Leeches => "leeches",
        ReportTab::Workload => "workload",
    }
}

/// The figures of `tab` (or the tab on screen), from the reports computed for the vault.
pub fn answer(state: &State, tab: Option<ReportTab>) -> Result<Value, String> {
    let reports: &ReportBundle = state
        .reports
        .as_ref()
        .ok_or("the reports are not computed yet (open a vault first)")?;
    let tab = tab.unwrap_or(state.explorer.tab);
    Ok(json!({
        "tab": tab_name(tab),
        "reviews": reports.reviews,
        "figures": figures(state, reports, tab),
    }))
}
