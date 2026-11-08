#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! The example scripts in `examples/scripts/` (STORY-0134), each run with the real binary on a scratch
//! vault: the health check, the export, the glossary importer, the event-bound tagger and the sample
//! `scripts.toml`.

mod bin_common;

use bin_common::*;

fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/scripts")
}

fn example(name: &str) -> String {
    examples()
        .join(name)
        .canonicalize()
        .unwrap()
        .display()
        .to_string()
}

/// A home whose last vault is `v`, so a headless instance opens it.
fn home_for(label: &str, v: &Path) -> PathBuf {
    let home = dir(label);
    std::fs::create_dir_all(home.join("config")).unwrap();
    std::fs::write(
        home.join("config/app.toml"),
        format!("vault = \"{}\"\n", v.display()),
    )
    .unwrap();
    home
}

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

fn setup(label: &str) -> Option<(PathBuf, PathBuf)> {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return None;
    }
    let v = dir(&format!("{label}-vault"));
    vault(&v);
    Some((home_for(&format!("{label}-home"), &v), v))
}

#[test]
fn the_health_check_reports_and_can_save_its_note() {
    let Some((home, v)) = setup("ex-health") else {
        return;
    };
    let (code, out, err) = script(
        &home,
        &["run", &example("health_check.py"), "--access", "read"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(
        out.contains("items: 1 cards, 0 questions, 0 notes"),
        "{out}"
    );
    assert!(
        out.contains("never rehearsed: 1") && out.contains("empty stacks: none"),
        "{out}"
    );
    let (code, out, err) = script(
        &home,
        &["run", &example("health_check.py"), "--", "--note", "fr"],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert_eq!(
        std::fs::read_dir(v.join("fr")).unwrap().count(),
        3,
        "the report was saved as a note"
    );
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}

#[test]
fn the_export_writes_and_prunes() {
    let Some((home, v)) = setup("ex-export") else {
        return;
    };
    let backups = dir("ex-backups");
    for _ in 0..2 {
        let (code, out, err) = script(
            &home,
            &[
                "run",
                &example("export_vault.py"),
                "--",
                backups.to_str().unwrap(),
                "--keep",
                "1",
            ],
        );
        assert_eq!(code, Some(0), "{out}{err}");
        assert!(out.contains("exported: studyfrog"), "{out}");
    }
    assert_eq!(
        std::fs::read_dir(&backups).unwrap().count(),
        1,
        "older exports are pruned"
    );
    for p in [&home, &v, &backups] {
        let _ = std::fs::remove_dir_all(p);
    }
}

#[test]
fn the_glossary_importer_adds_new_terms_once() {
    let Some((home, v)) = setup("ex-gloss") else {
        return;
    };
    let glossary = example("glossary.txt");
    let (code, out, err) = script(
        &home,
        &[
            "run",
            &example("import_glossary.py"),
            "--",
            &glossary,
            "--stack",
            "fr",
        ],
    );
    assert_eq!(code, Some(0), "{out}{err}");
    assert!(out.contains("3 card(s) created, 0 already there"), "{out}");
    assert!(err.contains("line 5: no term and definition"), "{err}");
    let (_, out, _) = script(
        &home,
        &[
            "run",
            &example("import_glossary.py"),
            "--",
            &glossary,
            "--stack",
            "fr",
        ],
    );
    assert!(
        out.contains("0 card(s) created, 3 already there"),
        "a second run adds nothing: {out}"
    );
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}

#[test]
fn the_sample_configuration_is_valid_and_the_tagger_tags_a_new_item() {
    if !has_python() {
        eprintln!("SKIPPED: python3 is not installed");
        return;
    }
    let v = dir("ex-tag-vault");
    vault(&v);
    let home = home_for("ex-tag-home", &v);
    std::fs::create_dir_all(home.join("scripts")).unwrap();
    std::fs::copy(
        examples().join("scripts.toml"),
        home.join("scripts/scripts.toml"),
    )
    .unwrap();
    let (code, out, _) = script(&home, &["list"]);
    assert_eq!(code, Some(0), "the sample has no errors: {out}");
    assert!(
        out.contains("tag-new: tag_new_cards.py [on item_created; write; 60 s]"),
        "{out}"
    );
    std::fs::write(
        home.join("scripts/scripts.toml"),
        "[[job]]\nname = \"tag-new\"\nscript = \"tag_new_cards.py\"\non = [\"item_created\"]\naccess = \"write\"\n",
    )
    .unwrap();
    std::fs::copy(
        examples().join("tag_new_cards.py"),
        home.join("scripts/tag_new_cards.py"),
    )
    .unwrap();
    let mut server = headless(&home, &v, &[]);
    let mut stdin = server.stdin.take().unwrap();
    writeln!(
        stdin,
        "item create card Maus --text mouse --answer Maus --tags animals --stack fr"
    )
    .unwrap();
    let card = v.join("fr/maus.md");
    let mut text = String::new();
    for _ in 0..200 {
        text = std::fs::read_to_string(&card).unwrap_or_default();
        if text.contains("added-") {
            break;
        }
        pause(100);
    }
    assert!(
        text.contains("animals") && text.contains("added-"),
        "{text}"
    );
    drop(stdin);
    assert!(server.wait().unwrap().success());
    let _ = std::fs::remove_dir_all(&home);
    let _ = std::fs::remove_dir_all(&v);
}
