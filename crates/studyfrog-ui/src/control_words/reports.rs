//! `reports` (the tab on screen), `reports TAB` and `reports tab TAB`.

use crate::control_cmd::Command;
use crate::explorer_ui::{ReportTab, tab_named};

/// Parse the words after `reports`.
pub fn parse(rest: &[String]) -> Result<Command, String> {
    let word = match rest {
        [] => return Ok(Command::Reports(None)),
        [tab, name] if tab == "tab" => name,
        [name] => name,
        _ => return Err("reports takes one tab name".to_owned()),
    };
    let tab: ReportTab = tab_named(word).ok_or_else(|| {
        format!(
            "unknown report tab {word} (performance, memory, familiarity, calibration, cognitive, runs, habits, leeches, workload)"
        )
    })?;
    Ok(Command::Reports(Some(tab)))
}
