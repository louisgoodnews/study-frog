#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! `studyfrog script run` (STORY-0132): with no instance it starts a headless one whose policy is the
//! script's access; with one running it goes through a proxy that refuses what the access does not
//! allow; a script over its timeout is stopped.

mod bin_common;

use bin_common::*;

const EXPORT: &str = r#"
import sys, studyfrog
vault, out = sys.argv[1], sys.argv[2]
with studyfrog.connect(start=False) as sf:
    sf.open_vault(vault)
    sf.export("json", out)
    print("exported", len(sf.query("type = card")), "cards")
    try:
        sf.delete_item("01J00000000000000000000010")
    except studyfrog.StudyFrogError as e:
        print("refused:", e)
"#;

fn script(home: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let o = Command::new(BIN)
        .arg("script")
        .args(args)
        .env("STUDYFROG_HOME", home)
        .output()
        .unwrap();
    let text = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (o.status.code(), text(o.stdout), text(o.stderr))
}

fn has_python() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn without_an_instance_a_headless_one_serves_the_run() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return;
    }
    let (home, v, work) = (dir("srun-home"), dir("srun-vault"), dir("srun-work"));
    vault(&v);
    let file = work.join("export.py");
    std::fs::write(&file, EXPORT).unwrap();
    let out = work.join("out");
    std::fs::create_dir_all(&out).unwrap();
    let (code, stdout, stderr) = script(
        &home,
        &[
            "run",
            file.to_str().unwrap(),
            "--",
            v.to_str().unwrap(),
            out.to_str().unwrap(),
        ],
    );
    assert_eq!(code, Some(0), "{stdout}\n{stderr}");
    assert!(stdout.contains("exported 1 cards"), "{stdout}");
    assert!(
        stdout.contains("refused:") && stdout.contains("destructive"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_dir(&out).unwrap().count(),
        1,
        "the export was written"
    );
    assert!(
        !home.join("run/control.sock").exists(),
        "the headless instance stopped"
    );
    let log = std::fs::read_to_string(home.join("logs/console.jsonl")).unwrap();
    assert!(log.contains("script export.py finished"), "{log}");
    for p in [&home, &v, &work] {
        let _ = std::fs::remove_dir_all(p);
    }
}

#[test]
fn with_an_instance_running_the_access_is_enforced_by_a_proxy() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return;
    }
    let (home, v) = (dir("sproxy-home"), dir("sproxy-vault"));
    vault(&v);
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::write(
        home.join("scripts/try_write.py"),
        "import studyfrog\nwith studyfrog.connect(start=False) as sf:\n    print(sf.state()['vault']['name'])\n    \
         try:\n        sf.create_item('note', 'x', stack='fr')\n        print('wrote')\n    \
         except studyfrog.StudyFrogError as e:\n        print('refused:', e)\n",
    )
    .unwrap();
    std::fs::write(
        home.join("scripts/slow.sh"),
        "#!/bin/sh\necho start\nsleep 30\n",
    )
    .unwrap();
    let perms = std::os::unix::fs::PermissionsExt::from_mode(0o755);
    std::fs::set_permissions(home.join("scripts/slow.sh"), perms).unwrap();
    let mut server = headless(&home, &v, &[]);
    let (code, stdout, _) = script(&home, &["run", "try_write.py", "--access", "read"]);
    assert_eq!(code, Some(0), "{stdout}");
    assert!(
        stdout.contains("refused: writing is switched off"),
        "{stdout}"
    );
    let (_, stdout, _) = script(&home, &["run", "try_write.py"]);
    assert!(
        stdout.contains("wrote"),
        "write access by default: {stdout}"
    );
    let started = std::time::Instant::now();
    let (code, stdout, stderr) = script(&home, &["run", "slow.sh", "--timeout", "1"]);
    assert_eq!(code, Some(1), "{stdout}{stderr}");
    assert!(stderr.contains("ran over its timeout"), "{stderr}");
    assert!(started.elapsed() < Duration::from_secs(20));
    let (code, _, stderr) = script(&home, &["run", "missing.py"]);
    assert_ne!(code, Some(0), "{stderr}");
    let (code, _, stderr) = script(&home, &["fly"]);
    assert_eq!(code, Some(2), "{stderr}");
    drop(server.stdin.take());
    assert!(server.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}
