#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The script runner in the real binary (STORY-0132): an event-bound Python job started by an item
//! created through standard input, talking back over its own read-only socket; and `script list`.

mod bin_common;

use bin_common::*;

const ON_CREATE: &str = r#"
import json, os, studyfrog
ev = studyfrog.trigger_event()
with studyfrog.connect(start=False) as sf:
    stacks = [s["title"] for s in sf.stacks()]
    try:
        sf.create_item("note", "from the script", stack="fr")
        wrote = "yes"
    except studyfrog.StudyFrogError as e:
        wrote = str(e)
out = {"kind": ev.kind, "title": ev.data["title"], "stacks": stacks, "write": wrote,
       "access": studyfrog.access(), "job": studyfrog.job_name()}
open(os.path.join(os.environ["STUDYFROG_HOME"], "marker.json"), "w").write(json.dumps(out))
print("handled", ev.data["title"])
"#;

const CONFIG: &str = r#"
[[job]]
name = "on-create"
script = "on_create.py"
on = ["item_created"]
timeout = 30

[[job]]
name = "nightly"
script = "nightly.sh"
schedule = "0 3 * * *"
access = "write"

[[job]]
name = "broken"
script = "../outside.py"
"#;

fn has_python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn an_item_created_starts_the_bound_script_with_read_access_only() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return;
    }
    let (home, v) = (dir("run-ev-home"), dir("run-ev-vault"));
    vault(&v);
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::write(home.join("scripts/scripts.toml"), CONFIG).unwrap();
    std::fs::write(home.join("scripts/on_create.py"), ON_CREATE).unwrap();
    let mut server = headless(&home, &v, &[]);
    let mut stdin = server.stdin.take().unwrap();
    writeln!(
        stdin,
        "item create card Hund --text dog --answer Hund --stack fr"
    )
    .unwrap();
    wait_for_secs(&home.join("marker.json"), 60);
    pause(300);
    let out = json(&std::fs::read_to_string(home.join("marker.json")).unwrap());
    assert_eq!(out["kind"], "item_created");
    assert_eq!(out["title"], "Hund");
    assert_eq!(out["stacks"], serde_json::json!(["French"]));
    assert_eq!(
        (&out["access"], &out["job"]),
        (&serde_json::json!("read"), &serde_json::json!("on-create"))
    );
    assert!(
        out["write"]
            .as_str()
            .unwrap()
            .contains("writing is switched off"),
        "{out}"
    );
    assert!(
        home.join("scripts/lib/studyfrog/client.py").is_file(),
        "the client was installed"
    );
    let mut log = String::new();
    for _ in 0..100 {
        log = std::fs::read_to_string(home.join("logs/console.jsonl")).unwrap_or_default();
        if log.contains("script on-create finished") {
            break;
        }
        pause(100);
    }
    assert!(log.contains("script on-create finished"), "{log}");
    assert!(log.contains("handled Hund"), "the output is logged: {log}");
    let leftovers: Vec<_> = std::fs::read_dir(home.join("run"))
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("script-"))
        .collect();
    assert!(leftovers.is_empty(), "the script's socket is removed");
    drop(stdin);
    assert!(server.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}

#[test]
fn a_job_allowed_destructive_commands_can_delete_in_this_instance() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return;
    }
    let (home, v) = (dir("run-del-home"), dir("run-del-vault"));
    vault(&v);
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::write(
        home.join("scripts/scripts.toml"),
        "[[job]]\nname = \"tidy\"\nscript = \"tidy.py\"\non = [\"item_saved\"]\naccess = \"write\"\nallow_destructive = true\n",
    )
    .unwrap();
    std::fs::write(
        home.join("scripts/tidy.py"),
        "import studyfrog\nev = studyfrog.trigger_event()\nwith studyfrog.connect(start=False) as sf:\n    sf.delete_item(ev.data['uid'])\n",
    )
    .unwrap();
    let mut server = headless(&home, &v, &[]);
    let mut stdin = server.stdin.take().unwrap();
    writeln!(stdin, "item edit 01J00000000000000000000010 title Gone").unwrap();
    for _ in 0..200 {
        if !v.join("fr/a.md").exists() {
            break;
        }
        pause(100);
    }
    assert!(
        !v.join("fr/a.md").exists(),
        "the script deleted the saved card"
    );
    drop(stdin);
    assert!(server.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}

#[test]
fn script_list_names_the_jobs_and_what_is_wrong() {
    let home = dir("run-list-home");
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::write(home.join("scripts/scripts.toml"), CONFIG).unwrap();
    let o = Command::new(BIN)
        .args(["script", "list"])
        .env("STUDYFROG_HOME", &home)
        .output()
        .unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert_eq!(o.status.code(), Some(1), "{text}");
    assert!(
        text.contains("on-create: on_create.py [on item_created; read; 30 s]"),
        "{text}"
    );
    assert!(
        text.contains("nightly: nightly.sh [schedule \"0 3 * * *\"; write; 60 s]"),
        "{text}"
    );
    assert!(
        text.contains("error: job 3: script = \"../outside.py\""),
        "{text}"
    );
    let o = Command::new(BIN)
        .args(["script", "install-client"])
        .env("STUDYFROG_HOME", &home)
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(home.join("scripts/lib/studyfrog/__init__.py").is_file());
    let _ = std::fs::remove_dir_all(&home);
}
