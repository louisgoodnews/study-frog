#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Running a script as a child process (STORY-0132): output captured and capped, exit codes, a
//! timeout that kills it (and what it started), and the log entries of each outcome.

use std::time::Duration;

use studyfrog_service::logging::LogLevel;
use studyfrog_service::scripting::exec::{MAX_OUTPUT, ScriptRun, run_script, succeeded};
use studyfrog_service::scripting::log::{outcome_entries, outcome_line};

fn pause() {
    let (_tx, rx) = std::sync::mpsc::channel::<()>();
    let _ = rx.recv_timeout(Duration::from_millis(10));
}

fn sh(script: &str, timeout_ms: u64) -> ScriptRun {
    ScriptRun {
        program: "sh".into(),
        args: vec!["-c".into(), script.into()],
        cwd: std::env::temp_dir(),
        env: vec![("SF_TEST_VALUE".into(), "frog".into())],
        timeout: Duration::from_millis(timeout_ms),
    }
}

#[test]
fn output_exit_code_and_environment_are_kept() {
    let o = run_script(&sh("echo out $SF_TEST_VALUE; echo err >&2", 10_000), &pause);
    assert!(succeeded(&o), "{o:?}");
    assert_eq!(
        (o.stdout.as_str(), o.stderr.as_str()),
        ("out frog\n", "err\n")
    );
    let failed = run_script(&sh("echo nope >&2; exit 3", 10_000), &pause);
    assert_eq!((failed.code, succeeded(&failed)), (Some(3), false));
    let entries = outcome_entries("health", &failed, 7);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].level, LogLevel::Error);
    assert!(
        entries[0]
            .message
            .starts_with("script health failed with exit code 3")
    );
    assert!(entries[1].message.contains("nope"));
    assert_eq!(entries[1].source, "script");
    let ok = outcome_entries("health", &o, 7);
    assert_eq!(ok[0].level, LogLevel::Info);
    assert!(ok[0].message.contains("finished") && ok[0].message.contains("out frog"));
}

#[test]
fn a_script_over_its_timeout_is_killed_with_what_it_started() {
    let started = std::time::Instant::now();
    let o = run_script(&sh("echo begun; sleep 30 & sleep 30", 300), &pause);
    assert!(o.timed_out, "{o:?}");
    assert_eq!(o.code, None);
    assert_eq!(o.stdout, "begun\n");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "not waited for the sleeps"
    );
    assert!(outcome_line("slow", &o).contains("ran over its timeout"));
}

#[test]
fn a_program_left_running_does_not_hold_the_run_open() {
    let started = std::time::Instant::now();
    let o = run_script(&sh("sleep 30 & echo done", 10_000), &pause);
    assert!(succeeded(&o), "{o:?}");
    assert!(o.stdout.contains("done"));
    assert!(started.elapsed() < Duration::from_secs(10));
}

#[test]
fn output_is_capped_and_a_missing_program_is_reported() {
    let o = run_script(&sh("head -c 200000 /dev/zero | tr '\\0' x", 10_000), &pause);
    assert_eq!(o.stdout.len(), MAX_OUTPUT);
    let missing = ScriptRun {
        program: "/no/such/interpreter".into(),
        ..sh("", 1000)
    };
    let m = run_script(&missing, &pause);
    assert!(
        m.spawn_error
            .as_deref()
            .unwrap()
            .contains("/no/such/interpreter")
    );
    assert!(outcome_line("x", &m).contains("could not start"));
}
