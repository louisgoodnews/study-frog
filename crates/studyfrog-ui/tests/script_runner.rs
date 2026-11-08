#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The runner's steps with an injected clock (STORY-0132): a scheduled job starts in its minute and
//! only once, an event starts its jobs but not the job whose end it reports, a running job is not
//! started twice, and the command line, environment and policy of a run.

mod common;
mod control_common;

use control_common::*;
use studyfrog_config::paths::app_paths;
use studyfrog_core::civil::{LocalTime, date_new, local_to_utc};
use studyfrog_service::scripting::config::Access;
use studyfrog_service::scripting::due::Ticker;
use studyfrog_ui::control_events::bus_new;
use studyfrog_ui::script_launch::{Launch, access_policy, command_for};
use studyfrog_ui::script_runner::{Runner, child_pause, event_step, scheduler_step, start};

fn runner(label: &str, config: &str) -> (Runner, TempDir) {
    let home = temp_dir(label);
    let paths = app_paths(&home.path);
    std::fs::create_dir_all(&paths.scripts).unwrap();
    std::fs::write(paths.scripts.join("scripts.toml"), config).unwrap();
    let script = paths.scripts.join("touch.sh");
    std::fs::write(
        &script,
        "#!/bin/sh\necho \"$STUDYFROG_JOB\" >> \"$STUDYFROG_HOME/ran.txt\"\nsleep \"${1:-0}\"\n",
    )
    .unwrap();
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755)).unwrap();
    let (inbox, _jobs) = channel();
    let mut ctx = conn_ctx(inbox, &paths.logs, || {});
    ctx.events = Some(bus_new());
    let r = Runner {
        paths,
        ctx,
        bus: bus_new(),
        running: Arc::default(),
        pause: child_pause,
    };
    (r, home)
}

fn ran(home: &TempDir, lines: usize) -> Vec<String> {
    let path = home.path.join("ran.txt");
    for _ in 0..300 {
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        if text.lines().count() >= lines {
            return text.lines().map(str::to_owned).collect();
        }
        child_pause();
    }
    std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect()
}

fn ms(h: u16, m: u16, offset: i32) -> i64 {
    let t = LocalTime {
        date: date_new(2026, 11, 9).unwrap(),
        minute: h * 60 + m,
    };
    local_to_utc(t, offset).0
}

#[test]
fn a_scheduled_job_starts_in_its_minute_once() {
    let (r, home) = runner(
        "run-sched",
        "[[job]]\nname = \"morning\"\nscript = \"touch.sh\"\nschedule = \"30 7 * * *\"\n",
    );
    let mut t = Ticker::default();
    assert!(scheduler_step(&r, &mut t, ms(7, 29, 120), 120).is_empty());
    assert_eq!(
        scheduler_step(&r, &mut t, ms(7, 30, 120) + 1_000, 120),
        ["morning"]
    );
    assert!(scheduler_step(&r, &mut t, ms(7, 30, 120) + 30_000, 120).is_empty());
    assert_eq!(ran(&home, 1), ["morning"]);
    let log = ran(&home, 1);
    assert_eq!(log.len(), 1, "started once");
}

#[test]
fn events_start_bound_jobs_but_not_the_job_whose_end_they_report() {
    let cfg = "[[job]]\nname = \"after\"\nscript = \"touch.sh\"\non = [\"script_finished\", \"item_created\"]\n";
    let (r, home) = runner("run-event", cfg);
    let own = json!({"kind": "script_finished", "data": {"job": "after"}});
    assert!(event_step(&r, &own).is_empty(), "no loop on its own end");
    assert!(event_step(&r, &json!({"kind": "item_saved", "data": {}})).is_empty());
    assert_eq!(
        event_step(&r, &json!({"kind": "item_created", "data": {}})),
        ["after"]
    );
    assert_eq!(ran(&home, 1), ["after"]);
}

#[test]
fn a_job_still_running_is_not_started_again() {
    let (r, home) = runner("run-busy", "");
    let slow = Launch {
        job: "slow".into(),
        script: "touch.sh".into(),
        args: vec!["1".into()],
        access: Access::Read,
        timeout_secs: 10,
        event: None,
    };
    assert!(start(&r, "python3", slow.clone()));
    assert!(
        !start(&r, "python3", slow.clone()),
        "the first run is still going"
    );
    assert_eq!(ran(&home, 1), ["slow"]);
}

#[test]
fn a_run_gets_its_socket_its_job_and_the_event_in_its_environment() {
    let paths = app_paths(std::path::Path::new("/h"));
    let l = Launch {
        job: "j".into(),
        script: "sub/x.py".into(),
        args: vec!["a".into()],
        access: Access::Write,
        timeout_secs: 7,
        event: Some(json!({"kind": "item_saved"})),
    };
    let run = command_for(
        &paths,
        "python3.12",
        &l,
        std::path::Path::new("/h/run/s.sock"),
    );
    assert_eq!(run.program, "python3.12");
    assert_eq!(run.args, ["/h/scripts/sub/x.py", "a"]);
    assert_eq!(run.cwd, std::path::Path::new("/h/scripts"));
    let env = |k: &str| run.env.iter().find(|(n, _)| n == k).map(|(_, v)| v.clone());
    assert_eq!(env("STUDYFROG_SOCKET").as_deref(), Some("/h/run/s.sock"));
    assert_eq!(env("STUDYFROG_ACCESS").as_deref(), Some("write"));
    assert_eq!(
        env("STUDYFROG_EVENT").as_deref(),
        Some("{\"kind\":\"item_saved\"}")
    );
    assert!(env("PYTHONPATH").unwrap().starts_with("/h/scripts/lib"));
    let sh = command_for(
        &paths,
        "python3",
        &Launch {
            script: "t.sh".into(),
            ..l
        },
        std::path::Path::new("/s"),
    );
    assert_eq!((sh.program.as_str(), sh.args.len()), ("/h/scripts/t.sh", 1));
    let p = |a| access_policy(a);
    assert_eq!(
        (
            p(Access::Read).allow_write,
            p(Access::Read).allow_destructive
        ),
        (false, false)
    );
    assert_eq!(
        (
            p(Access::Write).allow_write,
            p(Access::Write).allow_destructive
        ),
        (true, false)
    );
    assert!(p(Access::Destructive).allow_destructive);
}
