#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The job editor's service side (STORY-0151): next runs, a checked save that round-trips, and a refused save that
//! leaves the file as it was.

use std::fs;
use std::path::PathBuf;

use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::time::UnixMs;
use studyfrog_service::scripting::config::{Access, parse_scripts_toml};
use studyfrog_service::scripting::cron::parse_cron;
use studyfrog_service::scripting::jobs::{JobRow, next_run, read_config, rows_of, save};

const EVENTS: &[&str] = &["vault_opened", "item_created"];

struct Dir(PathBuf);
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp(label: &str) -> Dir {
    let d = std::env::temp_dir().join(format!("studyfrog-jobs-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    Dir(d)
}

/// Monday 2027-03-01 at the given local hour and minute.
fn at(hour: u16, minute: u16) -> LocalTime {
    LocalTime {
        date: Date {
            year: 2027,
            month: 3,
            day: 1,
        },
        minute: hour * 60 + minute,
    }
}

fn row(name: &str, schedule: &str) -> JobRow {
    JobRow {
        name: name.into(),
        script: "health.py".into(),
        args: vec!["--quick".into()],
        schedule: schedule.into(),
        on: vec!["vault_opened".into()],
        timeout_secs: 60,
        access: Access::Read,
        enabled: true,
        next_run: None,
    }
}

#[test]
fn the_next_run_is_the_first_matching_minute_after_now() {
    let c = parse_cron("30 8 * * mon-fri").unwrap();
    assert_eq!(next_run(&c, at(8, 0)), Some(at(8, 30)));
    // After 08:30 on a Monday, the next run is the next weekday at 08:30: Tuesday.
    let next = next_run(&c, at(8, 30)).unwrap();
    assert_eq!((next.date.day, next.minute), (2, 8 * 60 + 30));
}

#[test]
fn rows_show_next_runs_and_a_disabled_job_has_none() {
    let text = "[[job]]\nname = \"a\"\nscript = \"a.py\"\nschedule = \"0 9 * * *\"\n[[job]]\nname = \"b\"\nscript = \"b.py\"\nschedule = \"0 9 * * *\"\nenabled = false\n";
    let cfg = parse_scripts_toml(text, EVENTS);
    assert!(cfg.errors.is_empty(), "{:?}", cfg.errors);
    // A UTC clock at 2027-03-01 00:00 is 00:00 in offset 0, so the next 09:00 is the same day.
    let now = UnixMs(0);
    let rows = rows_of(&cfg, now, 0);
    assert!(
        rows[0].next_run.as_deref().unwrap().ends_with("09:00"),
        "{:?}",
        rows[0].next_run
    );
    assert_eq!(rows[1].next_run, None);
}

#[test]
fn a_saved_file_reads_back_the_same_jobs() {
    let d = temp("round");
    let rows = vec![row("a", "0 9 * * mon-fri"), row("b", "")];
    save(&d.0.join("scripts"), &rows, "python3", EVENTS).unwrap();
    let back = read_config(&d.0.join("scripts"), EVENTS);
    assert!(back.errors.is_empty(), "{:?}", back.errors);
    let again = rows_of(&back, UnixMs(0), 0);
    assert_eq!(again.len(), 2);
    assert_eq!(again[0].schedule, "0 9 * * mon-fri");
    assert_eq!(again[0].args, vec!["--quick".to_owned()]);
    assert_eq!(again[0].on, vec!["vault_opened".to_owned()]);
    assert_eq!(again[1].schedule, "");
}

#[test]
fn a_destructive_job_is_written_as_write_plus_allow() {
    let d = temp("destructive");
    let mut r = row("wipe", "");
    r.access = Access::Destructive;
    save(&d.0.join("scripts"), &[r], "python3", EVENTS).unwrap();
    let back = read_config(&d.0.join("scripts"), EVENTS);
    assert_eq!(back.jobs[0].access, Access::Destructive);
}

#[test]
fn a_bad_schedule_is_refused_and_the_file_is_kept() {
    let d = temp("refused");
    let scripts = d.0.join("scripts");
    save(&scripts, &[row("a", "0 9 * * *")], "python3", EVENTS).unwrap();
    let before = fs::read_to_string(scripts.join("scripts.toml")).unwrap();
    let err = save(&scripts, &[row("a", "not a schedule")], "python3", EVENTS).unwrap_err();
    assert!(
        err.contains("schedule") || err.contains("five fields"),
        "{err}"
    );
    assert_eq!(
        fs::read_to_string(scripts.join("scripts.toml")).unwrap(),
        before
    );
}
