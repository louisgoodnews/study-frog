#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! BUG-0042: a script run by hand is reported under the name typed, a job's own run under the job's name.

use studyfrog_service::scripting::config::{Access, parse_scripts_toml};
use studyfrog_ui::script_runner::resolve_run;

const EVENTS: &[&str] = &["item_created"];

fn config() -> studyfrog_service::scripting::config::ScriptsConfig {
    parse_scripts_toml(
        "[[job]]\nname = \"daily-hello\"\nscript = \"hello.py\"\naccess = \"read\"\ntimeout = 7\nargs = [\"--job\"]\n",
        EVENTS,
    )
}

fn typed(words: &[&str]) -> Vec<String> {
    words.iter().map(|w| (*w).to_owned()).collect()
}

#[test]
fn a_file_run_by_hand_is_reported_under_its_name_with_the_jobs_policy() {
    let cfg = config();
    let l = resolve_run(&cfg, "hello.py", &typed(&["xray", "yankee"]), true).unwrap();
    assert_eq!(l.job, "hello.py", "reported under the file the user chose");
    assert_eq!(l.script, "hello.py");
    assert_eq!(l.args, typed(&["xray", "yankee"]));
    assert_eq!(
        (l.access, l.timeout_secs),
        (Access::Read, 7),
        "no access is granted by the by-hand run"
    );
}

#[test]
fn a_job_run_by_its_own_name_keeps_the_job_name_and_arguments() {
    let cfg = config();
    let l = resolve_run(&cfg, "daily-hello", &[], true).unwrap();
    assert_eq!(l.job, "daily-hello");
    assert_eq!(l.args, typed(&["--job"]));
}

#[test]
fn a_script_of_its_own_runs_with_write_access_under_its_name() {
    let l = resolve_run(&config(), "other.py", &[], true).unwrap();
    assert_eq!((l.job.as_str(), l.access), ("other.py", Access::Write));
}

#[test]
fn an_unknown_name_is_refused() {
    assert!(resolve_run(&config(), "nope.py", &[], false).is_none());
}
