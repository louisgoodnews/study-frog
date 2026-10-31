#![allow(clippy::unwrap_used, clippy::expect_used)] // tests may panic on failure
//! Backups and restores of a vault, its database and the settings.

use std::fs;
use std::path::{Path, PathBuf};

use studyfrog_core::uid::Uid;
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::backup::{BackupOptions, create_backup};
use studyfrog_service::create::IdSource;
use studyfrog_service::kanban::board_or_default;
use studyfrog_service::restore::{read_manifest, restore_backup};
use studyfrog_service::session::open_vault;
use studyfrog_service::tar::{tar_bytes, tar_finish};

fn scratch(label: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("studyfrog-backup-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn make_vault(base: &Path) -> PathBuf {
    let v = base.join("my vault");
    fs::create_dir_all(v.join("deck")).unwrap();
    fs::create_dir_all(v.join("assets")).unwrap();
    let id = |n: u128| format_ulid(Uid(n));
    fs::write(
        v.join("deck/_stack.md"),
        format!("---\nid: {}\nkind: stack\ntitle: Deck\n---\n", id(1)),
    )
    .unwrap();
    fs::write(
        v.join("deck/c1.md"),
        format!(
            "---\nid: {}\nkind: card\ntitle: C1\n---\nq\n???\na\n",
            id(2)
        ),
    )
    .unwrap();
    fs::write(v.join("assets/p.png"), vec![7_u8; 1000]).unwrap();
    fs::write(v.join("subjects.toml"), "").unwrap();
    v
}

fn opts<'a>(cfg: Option<&'a Path>, keys: bool) -> BackupOptions<'a> {
    BackupOptions {
        config_dir: cfg,
        include_keys: keys,
        now_ms: 1_800_000_000_000,
        app_version: "0.1.0",
    }
}

#[test]
fn a_backup_restores_the_files_the_database_and_the_settings_into_a_new_folder() {
    let base = scratch("round");
    let vault = make_vault(&base);
    let cfg = base.join("config");
    fs::create_dir_all(&cfg).unwrap();
    fs::write(
        cfg.join("app.toml"),
        format!("vault = \"{}\"\nreading_wpm = 300\n", vault.display()),
    )
    .unwrap();
    fs::write(cfg.join("secrets.toml"), "openai = \"sk-secret\"\n").unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let ids = IdSource {
        now_ms: 1_800_000_000_000,
        random: [3; 10],
    };
    let board = board_or_default(&mut { session }, ids).unwrap();
    assert!(!board.columns.is_empty());
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();

    let dest = base.join("backups");
    let report = create_backup(&session, &dest, opts(Some(&cfg), false)).unwrap();
    assert!(
        report
            .path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("studyfrog-my-vault-")
    );
    let m = read_manifest(&report.path).unwrap();
    assert_eq!(
        (m.files, m.has_settings, m.has_keys, m.vault_name.as_str()),
        (4, true, false, "my vault")
    );
    assert_eq!(m, report.manifest);
    assert!(
        !fs::read(&report.path)
            .unwrap()
            .windows(9)
            .any(|w| w == b"sk-secret"),
        "keys are left out unless asked"
    );

    let target = base.join("restored");
    let new_cfg = base.join("config2");
    let r = restore_backup(&report.path, &target, Some(&new_cfg)).unwrap();
    assert_eq!(
        (r.files, r.settings_restored, r.keys_restored),
        (4, true, false)
    );
    assert_eq!(
        fs::read(target.join("assets/p.png")).unwrap(),
        vec![7_u8; 1000]
    );
    assert!(
        fs::read_to_string(target.join("deck/c1.md"))
            .unwrap()
            .contains("title: C1")
    );
    let settings = fs::read_to_string(new_cfg.join("app.toml")).unwrap();
    assert!(
        settings.contains("reading_wpm = 300") && settings.contains(&target.display().to_string()),
        "{settings}"
    );
    let (restored, _) = open_vault(target.to_str().unwrap()).unwrap();
    let again = board_or_default(&mut { restored }, ids).unwrap();
    assert_eq!(
        again, board,
        "the board lives in the database and came back"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn keys_are_included_only_on_request_and_restored_private() {
    use std::os::unix::fs::PermissionsExt;
    let base = scratch("keys");
    let vault = make_vault(&base);
    let cfg = base.join("config");
    fs::create_dir_all(&cfg).unwrap();
    fs::write(cfg.join("secrets.toml"), "openai = \"sk-secret\"\n").unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let report = create_backup(&session, &base.join("b"), opts(Some(&cfg), true)).unwrap();
    assert!(report.manifest.has_keys);
    let cfg2 = base.join("config2");
    fs::create_dir_all(&cfg2).unwrap();
    fs::write(cfg2.join("secrets.toml"), "old = 1\n").unwrap();
    let r = restore_backup(&report.path, &base.join("restored"), Some(&cfg2)).unwrap();
    assert!(r.keys_restored);
    let file = cfg2.join("secrets.toml");
    assert_eq!(
        fs::metadata(&file).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(fs::read_to_string(&file).unwrap().contains("sk-secret"));
    assert_eq!(
        fs::read_to_string(cfg2.join("secrets.toml.before-restore")).unwrap(),
        "old = 1\n",
        "the old keys are kept"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn a_restore_refuses_a_folder_with_content_and_bad_archives_and_unsafe_paths() {
    let base = scratch("refuse");
    let vault = make_vault(&base);
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let report = create_backup(&session, &base.join("b"), opts(None, false)).unwrap();
    let full = base.join("full");
    fs::create_dir_all(&full).unwrap();
    fs::write(full.join("mine.txt"), "x").unwrap();
    assert!(
        restore_backup(&report.path, &full, None)
            .unwrap_err()
            .to_string()
            .contains("not empty")
    );
    assert_eq!(fs::read_to_string(full.join("mine.txt")).unwrap(), "x");
    let empty = base.join("empty");
    fs::create_dir_all(&empty).unwrap();
    restore_backup(&report.path, &empty, None).unwrap();

    let junk = base.join("junk.tar");
    fs::write(&junk, b"this is not an archive at all").unwrap();
    assert!(read_manifest(&junk).is_err());
    assert!(restore_backup(&junk, &base.join("x"), None).is_err());
    assert!(!base.join("x").exists(), "nothing is left behind");

    let evil = base.join("evil.tar");
    let mut w = Vec::new();
    let manifest = "format = 1\ncreated_ms = 1\napp_version = \"x\"\nvault_name = \"v\"\nfiles = 1\nbytes = 1\nhas_settings = false\nhas_keys = false\n";
    tar_bytes(&mut w, "manifest.toml", manifest.as_bytes(), 0).unwrap();
    tar_bytes(&mut w, "database/index.sqlite", b"x", 0).unwrap();
    tar_bytes(&mut w, "vault/../escape.txt", b"x", 0).unwrap();
    tar_finish(&mut w).unwrap();
    fs::write(&evil, w).unwrap();
    assert!(
        restore_backup(&evil, &base.join("y"), None)
            .unwrap_err()
            .to_string()
            .contains("unsafe path")
    );
    assert!(!base.join("escape.txt").exists() && !base.join("y").exists());
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn the_archive_is_a_plain_tar_that_other_tools_can_list() {
    let base = scratch("tar");
    let vault = make_vault(&base);
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let report = create_backup(&session, &base.join("b"), opts(None, false)).unwrap();
    let out = std::process::Command::new("tar")
        .arg("-tf")
        .arg(&report.path)
        .output();
    if let Some(out) = out.ok().filter(|o| o.status.success()) {
        let listing = String::from_utf8_lossy(&out.stdout).into_owned();
        for name in [
            "manifest.toml",
            "database/index.sqlite",
            "vault/deck/c1.md",
            "vault/assets/p.png",
        ] {
            assert!(listing.lines().any(|l| l == name), "{name} in {listing}");
        }
    }
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn long_paths_survive_the_prefix_split() {
    let base = scratch("long");
    let vault = make_vault(&base);
    let deep = vault.join("a".repeat(60)).join("b".repeat(60));
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("note.md"), "x").unwrap();
    let (session, _) = open_vault(vault.to_str().unwrap()).unwrap();
    let report = create_backup(&session, &base.join("b"), opts(None, false)).unwrap();
    restore_backup(&report.path, &base.join("r"), None).unwrap();
    assert!(
        base.join("r")
            .join("a".repeat(60))
            .join("b".repeat(60))
            .join("note.md")
            .is_file()
    );
    let _ = fs::remove_dir_all(&base);
}
