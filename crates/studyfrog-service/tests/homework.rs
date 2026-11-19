#![allow(clippy::unwrap_used, clippy::expect_used)]
//! End-to-end tests for homework creation, validation, edits, toggles, and deletion.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use studyfrog_calendar::homework::{ChecklistItem, HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime};
use studyfrog_core::uid::Uid;
use studyfrog_service::create::IdSource;
use studyfrog_service::homework::{
    HomeworkDraft, create_homework, delete_homework, list_homework, save_homework, toggle_homework,
    toggle_homework_checklist,
};
use studyfrog_service::session::{ServiceError, open_vault};
use studyfrog_store::homework_toml::homework_toml_path;

static SEQ: AtomicU32 = AtomicU32::new(0);

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_vault(label: &str) -> Dir {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("studyfrog-hw-{label}-{}-{n}", std::process::id()));
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    Dir(dir)
}

fn draft(title: &str) -> HomeworkDraft {
    HomeworkDraft {
        title: title.to_owned(),
        subject: Some("Physics".to_owned()),
        stack_uid: Some(Uid(1)),
        timetable_id: Some(Uid(2)),
        due: LocalTime {
            date: Date {
                year: 2026,
                month: 11,
                day: 25,
            },
            minute: 1020,
        },
        start_date: Some(Date {
            year: 2026,
            month: 11,
            day: 18,
        }),
        estimated_minutes: 75,
        priority: HomeworkPriority::High,
        status: HomeworkStatus::NotStarted,
        checklist: vec![
            ChecklistItem {
                text: "Read section".to_owned(),
                done: false,
            },
            ChecklistItem {
                text: "Solve sheet".to_owned(),
                done: false,
            },
        ],
        notes: "Line 1\nLine 2".to_owned(),
        reminders: vec![1440, 60],
    }
}

fn ids(seed: u8) -> IdSource {
    IdSource {
        now_ms: 1_760_000_000_000 + u64::from(seed),
        random: [seed; 10],
    }
}

#[test]
fn fresh_vault_create_restart_edit_and_delete_work() {
    let d = temp_vault("crud");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    assert!(list_homework(&session).unwrap().is_empty());

    let list = create_homework(&mut session, draft("Lab Report"), ids(1)).unwrap();
    assert_eq!(list.len(), 1);
    let mut item = list[0].clone();
    assert_eq!(item.reminders, vec![60, 1440]);
    assert!(homework_toml_path(&d.0).exists());
    assert!(!d.0.join(".studyfrog/.homework.toml.tmp").exists());
    drop(session);

    let (mut session, _) = open_vault(path).unwrap();
    let reloaded = list_homework(&session).unwrap();
    assert_eq!(reloaded, vec![item.clone()]);

    item.title = "Lab Report Final".to_owned();
    let updated = save_homework(&mut session, item.clone()).unwrap();
    assert_eq!(updated[0].title, "Lab Report Final");

    let after_del = delete_homework(&mut session, item.id).unwrap();
    assert!(after_del.is_empty());
    assert!(matches!(
        delete_homework(&mut session, item.id),
        Err(ServiceError::Invalid(_))
    ));
}

#[test]
fn validation_rejections_leave_sqlite_and_toml_untouched() {
    let d = temp_vault("invalid");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    create_homework(&mut session, draft("Valid"), ids(2)).unwrap();
    let before_toml = fs::read_to_string(homework_toml_path(&d.0)).unwrap();

    let mut bad_title = draft(" ");
    bad_title.title = "\n".to_owned();
    assert!(matches!(
        create_homework(&mut session, bad_title, ids(3)),
        Err(ServiceError::Invalid(_))
    ));

    let mut bad_est = draft("Zero");
    bad_est.estimated_minutes = 0;
    assert!(matches!(
        create_homework(&mut session, bad_est, ids(4)),
        Err(ServiceError::Invalid(_))
    ));

    let mut bad_start = draft("Dates");
    bad_start.start_date = Some(Date {
        year: 2026,
        month: 12,
        day: 1,
    });
    assert!(matches!(
        create_homework(&mut session, bad_start, ids(5)),
        Err(ServiceError::Invalid(_))
    ));

    let mut too_many_rem = draft("Rem");
    too_many_rem.reminders = vec![1, 2, 3, 4, 5, 6];
    assert!(matches!(
        create_homework(&mut session, too_many_rem, ids(6)),
        Err(ServiceError::Invalid(_))
    ));

    assert_eq!(list_homework(&session).unwrap().len(), 1);
    assert_eq!(
        fs::read_to_string(homework_toml_path(&d.0)).unwrap(),
        before_toml
    );
}

#[test]
fn out_of_range_years_return_service_invalid() {
    let d = temp_vault("years");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    create_homework(&mut session, draft("Valid"), ids(10)).unwrap();
    let before_toml = fs::read_to_string(homework_toml_path(&d.0)).unwrap();

    let mut bad_year_due = draft("Year 10000");
    bad_year_due.due.date.year = 10_000;
    assert!(matches!(
        create_homework(&mut session, bad_year_due, ids(11)),
        Err(ServiceError::Invalid(_))
    ));

    let mut bad_year_item = list_homework(&session).unwrap()[0].clone();
    bad_year_item.due.date.year = 10_000;
    assert!(matches!(
        save_homework(&mut session, bad_year_item),
        Err(ServiceError::Invalid(_))
    ));

    let mut neg_year_start = draft("Neg start");
    neg_year_start.start_date = Some(Date {
        year: -1,
        month: 1,
        day: 1,
    });
    assert!(matches!(
        create_homework(&mut session, neg_year_start, ids(12)),
        Err(ServiceError::Invalid(_))
    ));

    assert_eq!(list_homework(&session).unwrap().len(), 1);
    assert_eq!(
        fs::read_to_string(homework_toml_path(&d.0)).unwrap(),
        before_toml
    );
}

#[test]
fn status_and_checklist_toggles_persist_and_reject_bad_targets() {
    let d = temp_vault("toggle");
    let path = d.0.to_str().unwrap();
    let (mut session, _) = open_vault(path).unwrap();
    let items = create_homework(&mut session, draft("Exercises"), ids(7)).unwrap();
    let id = items[0].id;

    let checked = toggle_homework_checklist(&mut session, id, 0).unwrap();
    assert!(checked[0].checklist[0].done);

    let completed = toggle_homework(&mut session, id).unwrap();
    assert_eq!(completed[0].status, HomeworkStatus::Completed);

    let reopened = toggle_homework(&mut session, id).unwrap();
    assert_eq!(reopened[0].status, HomeworkStatus::InProgress);

    assert!(matches!(
        toggle_homework_checklist(&mut session, id, 99),
        Err(ServiceError::Invalid(_))
    ));
    assert!(matches!(
        toggle_homework(&mut session, Uid(999)),
        Err(ServiceError::Invalid(_))
    ));
}
