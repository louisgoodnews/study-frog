#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The reports command of the control protocol: the tab words, and the figures each tab answers with
//! (STORY-0124, EPIC-064).

mod common;
mod control_common;

use control_common::*;
use studyfrog_ui::control_cmd::Command;
use studyfrog_ui::control_cmd::{Class, class_of};
use studyfrog_ui::control_parse::parse_request;
use studyfrog_ui::explorer_ui::ReportTab;

const TABS: [&str; 9] = [
    "performance",
    "memory",
    "familiarity",
    "calibration",
    "cognitive",
    "runs",
    "habits",
    "leeches",
    "workload",
];

#[test]
fn reports_words_parse_and_read() {
    assert_eq!(
        parse_request("reports cognitive").unwrap().cmd,
        Command::Reports(Some(ReportTab::Cognitive))
    );
    assert_eq!(
        parse_request("reports tab leeches").unwrap().cmd,
        Command::Reports(Some(ReportTab::Leeches))
    );
    assert_eq!(
        parse_request("reports").unwrap().cmd,
        Command::Reports(None)
    );
    assert!(parse_request("reports nope").is_err());
    assert_eq!(class_of(&Command::Reports(None)), Class::Read);
}

#[test]
fn each_tab_answers_with_its_figures() {
    let (vault, logs) = (sample_vault("ctl-reports"), temp_dir("ctl-reports-logs"));
    let mut e = engine(&vault.path, &logs.path, policy_default());
    for tab in TABS {
        let r = ask(&mut e, &format!("reports {tab}"));
        assert_eq!(r["ok"], true, "{tab}: {r}");
        assert_eq!(r["result"]["tab"], tab, "{r}");
        assert!(r["result"]["reviews"].is_number(), "{tab}: {r}");
        assert!(
            r["result"]["figures"].is_object() || r["result"]["figures"].is_null(),
            "{tab}: {r}"
        );
    }
    let cognitive = ask(&mut e, "reports cognitive");
    assert!(
        cognitive["result"]["figures"]["answers"].is_number(),
        "{cognitive}"
    );
    assert!(
        cognitive["result"]["figures"]["weekly_pass"].is_array(),
        "{cognitive}"
    );
    let memory = ask(&mut e, "reports memory");
    assert!(
        memory["result"]["figures"]["retention"].is_array(),
        "{memory}"
    );
}

#[test]
fn a_bare_reports_answers_the_tab_on_screen() {
    let (vault, logs) = (
        sample_vault("ctl-reports-bare"),
        temp_dir("ctl-reports-bare-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    ask(&mut e, "reports workload");
    let r = ask(&mut e, "reports");
    assert_eq!(r["result"]["tab"], "workload", "{r}");
    assert!(r["result"]["figures"]["days"].is_array(), "{r}");
}
