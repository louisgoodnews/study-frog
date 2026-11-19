#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Tests for external TOML sync, corrupt-TOML rejection, backup/restore parity, and vault isolation.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use studyfrog_calendar::homework::{HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_md::ulid::format_ulid;
use studyfrog_service::backup::{BackupOptions, create_backup};
use studyfrog_service::create::IdSource;
use studyfrog_service::homework::{HomeworkDraft, create_homework, list_homework, sync_homework};
use studyfrog_service::restore::restore_backup;
use studyfrog_service::session::{ServiceError, open_vault, poll_vault};
use studyfrog_store::error::StoreError;
use studyfrog_store::homework_toml::homework_toml_path;

static SEQ: AtomicU32 = AtomicU32::new(0);

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_dir(label: &str) -> Dir {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-hw-sync-{label}-{}-{n}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    Dir(dir)
}

fn temp_vault(label: &str) -> Dir {
    let d = temp_dir(label);
    fs::create_dir_all(d.0.join("s")).unwrap();
    fs::write(
        d.0.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    d
}

fn draft(title: &str) -> HomeworkDraft {
    HomeworkDraft {
        title: title.to_owned(),
        subject: Some("CS".to_owned()),
        stack_uid: None,
        timetable_id: None,
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 11,
                day: 28,
            },
            minute: 900,
        },
        start_date: None,
        estimated_minutes: 50,
        priority: HomeworkPriority::Medium,
        status: HomeworkStatus::NotStarted,
        checklist: vec![],
        notes: String::new(),
        reminders: vec![60],
    }
}

fn ids(seed: u8) -> IdSource {
    IdSource {
        now_ms: 1_760_000_100_000 + u64::from(seed),
        random: [seed; 10],
    }
}

#[test]
fn external_toml_sync_preserves_fired_history_and_isolates_vault() {
    let d = temp_vault("ext");
    let path = d.0.to_str().unwrap();
    let (mut session, info_before) = open_vault(path).unwrap();
    let created = create_homework(&mut session, draft("Original"), ids(1)).unwrap();
    let hw_id = created[0].id;
    session
        .conn
        .execute(
            "INSERT INTO homework_reminder_fired (homework_id, due_date, minutes_before) VALUES (?1, '2026-11-28', 60)",
            [format_ulid(hw_id)],
        )
        .unwrap();

    // Vault markdown scanning and polling ignore `.studyfrog/homework.toml`.
    assert!(poll_vault(&mut session, i64::MAX / 2).unwrap().is_none());

    let toml_path = homework_toml_path(&d.0);
    let edited = fs::read_to_string(&toml_path)
        .unwrap()
        .replace("Original", "Edited Externally");
    fs::write(&toml_path, edited).unwrap();

    let synced = sync_homework(&mut session).unwrap();
    assert_eq!(synced[0].title, "Edited Externally");
    let fired: i64 = session
        .conn
        .query_row("SELECT COUNT(*) FROM homework_reminder_fired", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(fired, 1);
    drop(session);

    let (_, info_after) = open_vault(path).unwrap();
    assert_eq!(info_before.overview, info_after.overview);
}

#[test]
fn corrupt_toml_errors_without_overwriting_and_backup_restore_survives() {
    let d = temp_vault("corrupt-backup");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    create_homework(&mut session, draft("Algorithms"), ids(2)).unwrap();

    let backup_dir = temp_dir("backup-out");
    let report = create_backup(
        &session,
        &backup_dir.0,
        BackupOptions {
            config_dir: None,
            include_keys: false,
            now_ms: 1_760_000_200_000,
            app_version: "1.0.0",
        },
    )
    .unwrap();
    drop(session);

    // Corrupt TOML refuses open_vault and preserves file on disk.
    let toml_path = homework_toml_path(&d.0);
    let broken = "[[homework]]\nnot_valid = [";
    fs::write(&toml_path, broken).unwrap();
    assert!(matches!(
        open_vault(path),
        Err(ServiceError::Store(StoreError::Config { .. }))
    ));
    assert_eq!(fs::read_to_string(&toml_path).unwrap(), broken);

    // Restore backup into clean directory -> TOML backfilled on open_vault.
    let restored = temp_dir("restored-vault");
    restore_backup(&report.path, &restored.0, None).unwrap();
    assert!(!homework_toml_path(&restored.0).exists());
    let (restored_session, _) = open_vault(restored.0.to_str().unwrap()).unwrap();
    let items = list_homework(&restored_session).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Algorithms");
    assert!(homework_toml_path(&restored.0).exists());
}

#[test]
fn case_variant_duplicate_ulids_fail_without_overwriting_disk_or_sqlite() {
    let d = temp_vault("dup-case");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    create_homework(&mut session, draft("Keep Existing"), ids(3)).unwrap();
    drop(session);

    let dup_toml = "\
[[homework]]
id = \"01K742SG0A1850M2GA1850M2GA\"
title = \"First\"
due_date = \"2026-11-28\"
due_time = \"09:00\"
estimated_minutes = 30
priority = \"medium\"
status = \"not_started\"

[[homework]]
id = \"01k742sg0a1850m2ga1850m2ga\"
title = \"Second\"
due_date = \"2026-11-28\"
due_time = \"10:00\"
estimated_minutes = 30
priority = \"medium\"
status = \"not_started\"
";
    let toml_path = homework_toml_path(&d.0);
    fs::write(&toml_path, dup_toml).unwrap();
    assert!(matches!(
        open_vault(path),
        Err(ServiceError::Store(StoreError::Config { .. }))
    ));
    assert_eq!(fs::read_to_string(&toml_path).unwrap(), dup_toml);

    // Original SQLite row remains intact if TOML is fixed back to empty or valid.
    fs::remove_file(&toml_path).unwrap();
    let (session_after, _) = open_vault(path).unwrap();
    let items = list_homework(&session_after).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, "Keep Existing");
}
