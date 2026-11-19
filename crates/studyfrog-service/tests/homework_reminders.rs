#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Homework reminders through the app's reminder path: due at the lead time, shown once, and
//! still not shown again after the vault is reopened (the fired record is in SQLite).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use studyfrog_calendar::homework::{HomeworkPriority, HomeworkStatus};
use studyfrog_core::civil::{Date, LocalTime, date_to_days};
use studyfrog_core::time::UnixMs;
use studyfrog_service::create::IdSource;
use studyfrog_service::homework::{HomeworkDraft, create_homework};
use studyfrog_service::reminders::take_due_reminders;
use studyfrog_service::session::open_vault;

static SEQ: AtomicU32 = AtomicU32::new(0);

struct Dir(PathBuf);

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_vault(label: &str) -> Dir {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "studyfrog-hw-remind-{label}-{}-{n}",
        std::process::id()
    ));
    fs::create_dir_all(dir.join("s")).unwrap();
    fs::write(
        dir.join("s/_stack.md"),
        "---\nid: 00000000000000000000000001\nkind: stack\ntitle: S\n---\n",
    )
    .unwrap();
    Dir(dir)
}

/// Local 2026-11-28 at `minute_of_day`, as UTC milliseconds with a zero offset.
fn at(minute_of_day: i64) -> UnixMs {
    let day = date_to_days(Date {
        year: 2026,
        month: 11,
        day: 28,
    });
    UnixMs(day * 86_400_000 + minute_of_day * 60_000)
}

fn draft() -> HomeworkDraft {
    HomeworkDraft {
        title: "Essay".to_owned(),
        subject: None,
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

#[test]
fn homework_reminder_comes_once_and_stays_shown_after_reopen() {
    let dir = temp_vault("once");
    let path = dir.0.to_str().unwrap();
    {
        let (mut session, _) = open_vault(path).unwrap();
        let ids = IdSource {
            now_ms: 1_760_000_100_000,
            random: [7; 10],
        };
        create_homework(&mut session, draft(), ids).unwrap();
        assert!(
            take_due_reminders(&session, at(13 * 60 + 59), 0)
                .unwrap()
                .is_empty(),
            "one minute before the one-hour lead is too early"
        );
        let due = take_due_reminders(&session, at(14 * 60), 0).unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].title, "Essay");
        assert_eq!(due[0].minutes_before, 60);
        assert!(
            take_due_reminders(&session, at(14 * 60 + 5), 0)
                .unwrap()
                .is_empty(),
            "shown once in the same session"
        );
    }
    let (session, _) = open_vault(path).unwrap();
    assert!(
        take_due_reminders(&session, at(14 * 60 + 10), 0)
            .unwrap()
            .is_empty(),
        "still shown once after the vault is reopened"
    );
}
