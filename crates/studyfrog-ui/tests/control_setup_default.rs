#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `rehearse setup` without a preset starts from Spaced (BUG-0054), and a run from it writes reviews.

mod common;
mod control_common;

use control_common::*;

fn reviews(e: &studyfrog_ui::control_engine::Engine) -> i64 {
    let conn = &e.worker.session.as_ref().unwrap().conn;
    conn.query_row("SELECT COUNT(*) FROM review", [], |x| x.get(0))
        .unwrap()
}

/// Flip and rate Good until the run is finished (or ten items).
fn rate_until_finished(e: &mut studyfrog_ui::control_engine::Engine) {
    for _ in 0..10 {
        let s = ask(e, "state");
        if s["result"]["rehearse"]["run"]["state"] == "finished" {
            break;
        }
        ask(e, "rehearse flip");
        ask(e, "rehearse rate good");
    }
}

#[test]
fn a_setup_without_a_preset_starts_spaced_after_a_practice_preset() {
    let (vault, logs) = (
        sample_vault("setup-default"),
        temp_dir("setup-default-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    assert_eq!(
        ask(&mut e, "rehearse setup --preset all --stack french")["ok"],
        true
    );
    assert_eq!(ask(&mut e, "state")["result"]["rehearse"]["preset"], "all");
    assert_eq!(ask(&mut e, "rehearse setup --stack french")["ok"], true);
    assert_eq!(
        ask(&mut e, "state")["result"]["rehearse"]["preset"],
        "spaced"
    );
}

#[test]
fn a_run_from_the_defaulted_setup_writes_reviews_and_a_practice_preset_does_not() {
    let (vault, logs) = (
        sample_vault("setup-default-run"),
        temp_dir("setup-default-run-logs"),
    );
    let mut e = engine(&vault.path, &logs.path, policy_default());
    ask(&mut e, "rehearse setup --preset all --stack french");
    assert_eq!(ask(&mut e, "rehearse start")["ok"], true);
    rate_until_finished(&mut e);
    ask(&mut e, "rehearse leave");
    assert_eq!(reviews(&e), 0, "a practice-only run writes no reviews");

    assert_eq!(ask(&mut e, "rehearse setup --stack french")["ok"], true);
    assert_eq!(ask(&mut e, "rehearse start")["ok"], true);
    rate_until_finished(&mut e);
    ask(&mut e, "rehearse leave");
    assert!(
        reviews(&e) > 0,
        "a run from the defaulted (spaced) setup writes reviews"
    );
}
