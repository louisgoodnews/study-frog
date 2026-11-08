#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `scripts.toml` (STORY-0132): valid jobs with defaults, every bad entry reported and left out.

use studyfrog_service::scripting::config::{
    Access, DEFAULT_TIMEOUT_SECS, parse_scripts_toml, safe_script_path,
};
use studyfrog_service::scripting::due::find_job;

const EVENTS: &[&str] = &["item_created", "vault_opened"];

#[test]
fn a_job_gets_defaults_and_its_access() {
    let cfg = parse_scripts_toml(
        "python = \"/usr/bin/python3.12\"\n\
         [[job]]\nname = \"a\"\nscript = \"a.py\"\n\
         [[job]]\nname = \"w\"\nscript = \"sub/w.py\"\naccess = \"write\"\nargs = [\"--x\"]\ntimeout = 5\non = [\"item_created\"]\n\
         [[job]]\nname = \"d\"\nscript = \"d.py\"\naccess = \"write\"\nallow_destructive = true\n",
        EVENTS,
    );
    assert!(cfg.errors.is_empty(), "{:?}", cfg.errors);
    assert_eq!(cfg.python, "/usr/bin/python3.12");
    let a = &cfg.jobs[0];
    assert_eq!(
        (a.access, a.timeout_secs, a.enabled, a.schedule.is_none()),
        (Access::Read, DEFAULT_TIMEOUT_SECS, true, true)
    );
    let w = &cfg.jobs[1];
    assert_eq!(
        (w.access, w.timeout_secs, w.args.clone(), w.on.clone()),
        (
            Access::Write,
            5,
            vec!["--x".to_owned()],
            vec!["item_created".to_owned()]
        )
    );
    assert_eq!(cfg.jobs[2].access, Access::Destructive);
    assert_eq!(find_job(&cfg, "sub/w.py").unwrap().name, "w");
    assert_eq!(find_job(&cfg, "d").unwrap().script, "d.py");
    assert!(find_job(&cfg, "nope").is_none());
}

#[test]
fn bad_entries_are_reported_with_their_reason_and_left_out() {
    let cfg = parse_scripts_toml(
        "[[job]]\nname = \"ok\"\nscript = \"ok.py\"\n\
         [[job]]\nname = \"up\"\nscript = \"../evil.py\"\n\
         [[job]]\nname = \"abs\"\nscript = \"/bin/sh\"\n\
         [[job]]\nname = \"ev\"\nscript = \"e.py\"\non = [\"moon_rose\"]\n\
         [[job]]\nname = \"cron\"\nscript = \"c.py\"\nschedule = \"61 * * * *\"\n\
         [[job]]\nname = \"slow\"\nscript = \"s.py\"\ntimeout = 0\n\
         [[job]]\nname = \"acc\"\nscript = \"x.py\"\naccess = \"root\"\n\
         [[job]]\nname = \"ok\"\nscript = \"again.py\"\n\
         [[job]]\nname = \"typo\"\nscript = \"t.py\"\nshedule = \"@daily\"\n",
        EVENTS,
    );
    let names: Vec<&str> = cfg.jobs.iter().map(|j| j.name.as_str()).collect();
    assert_eq!(names, ["ok"]);
    let all = cfg.errors.join("\n");
    for says in [
        "job 2: script = \"../evil.py\"",
        "job 3: script = \"/bin/sh\"",
        "job 4: unknown event moon_rose",
        "job 5: minute",
        "job 6: timeout = 0",
        "job 7: access = \"root\"",
        "job 8: the name ok is used twice",
        "job 9: unknown field `shedule`",
    ] {
        assert!(all.contains(says), "missing {says:?} in\n{all}");
    }
}

#[test]
fn a_broken_file_is_one_error_and_no_jobs() {
    let cfg = parse_scripts_toml("[[job]\nname=", EVENTS);
    assert!(cfg.jobs.is_empty());
    assert_eq!(cfg.errors.len(), 1);
    assert!(cfg.errors[0].starts_with("scripts.toml:"));
    assert_eq!(parse_scripts_toml("", EVENTS), {
        let mut c = parse_scripts_toml("", &[]);
        c.python = "python3".into();
        c
    });
}

#[test]
fn script_paths_stay_inside_the_folder() {
    assert!(safe_script_path("a.py") && safe_script_path("lib/b.sh"));
    for bad in ["", " ", "../a.py", "a/../../b", "/etc/passwd", "./a.py"] {
        assert!(!safe_script_path(bad), "{bad}");
    }
}
